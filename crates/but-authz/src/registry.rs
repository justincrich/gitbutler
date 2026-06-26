use std::{collections::BTreeMap, fs, io::Write, path::Path, time::Duration};

use anyhow::{Context, anyhow};
use gix::lock::acquire::Fail;
use serde::{Deserialize, Serialize};

use crate::PrincipalId;

const REGISTRY_LOCK_TIMEOUT: Duration = Duration::from_secs(10);

/// Agent identity used by the runtime process registry.
pub type AgentId = PrincipalId;

/// Runtime process identity key: process id plus process start time.
pub type ProcessKey = (u32, u64);

/// One runtime registration for an agent process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registration {
    pub agent_id: AgentId,
    pub registered_at: u64,
    pub expires_at: u64,
    pub registered_by: AgentId,
}

/// Runtime registry mapping process identity to registered agent identity.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    registrations: BTreeMap<ProcessKey, Registration>,
    loaded_base: Option<BTreeMap<ProcessKey, Registration>>,
}

impl PartialEq for Registry {
    fn eq(&self, other: &Self) -> bool {
        self.registrations == other.registrations
    }
}

impl Eq for Registry {}

impl Registry {
    /// Create an empty runtime registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Load a registry from a TOML file, treating a missing file as empty.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let registrations = load_registrations(path)?;
        Ok(Self::from_loaded(registrations))
    }

    /// Write the registry to a TOML file through a Git-style lock file.
    pub fn write(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let path = path.as_ref();
        self.write_inner(path)
            .with_context(|| format!("writing registry {}", path.display()))
    }

    /// Register or replace one process identity.
    pub fn register(
        &mut self,
        pid: u32,
        start_time: u64,
        agent_id: impl Into<AgentId>,
        ttl_seconds: u64,
        registered_by: impl Into<AgentId>,
    ) -> anyhow::Result<()> {
        let expires_at = start_time
            .checked_add(ttl_seconds)
            .ok_or_else(|| anyhow!("registry registration for pid {pid} overflows expires_at"))?;
        self.registrations.insert(
            (pid, start_time),
            Registration {
                agent_id: agent_id.into(),
                registered_at: start_time,
                expires_at,
                registered_by: registered_by.into(),
            },
        );
        Ok(())
    }

    /// Remove one process identity from the registry.
    pub fn unregister(&mut self, key: ProcessKey) -> Option<Registration> {
        self.registrations.remove(&key)
    }

    /// Resolve a process identity to an agent id.
    pub fn resolve(&self, key: ProcessKey) -> Option<AgentId> {
        self.registrations
            .get(&key)
            .map(|registration| registration.agent_id.clone())
    }

    /// Remove registrations whose expiry is before `now`.
    ///
    /// `expires_at` is the last valid second, so an entry survives when
    /// `expires_at == now` and is collected only once `expires_at < now`.
    pub fn gc(&mut self, now: u64) -> usize {
        let before = self.registrations.len();
        self.registrations
            .retain(|_, registration| registration.expires_at >= now);
        before - self.registrations.len()
    }

    /// Return the number of registrations.
    pub fn len(&self) -> usize {
        self.registrations.len()
    }

    /// Return whether the registry contains no registrations.
    pub fn is_empty(&self) -> bool {
        self.registrations.is_empty()
    }

    /// Return all registrations keyed by `(pid, start_time)`.
    pub fn registrations(&self) -> &BTreeMap<ProcessKey, Registration> {
        &self.registrations
    }

    fn write_inner(&self, path: &Path) -> anyhow::Result<()> {
        let mut lock = gix::lock::File::acquire_to_update_resource(
            path,
            Fail::AfterDurationWithBackoff(REGISTRY_LOCK_TIMEOUT),
            None,
        )
        .with_context(|| format!("locking registry {}", path.display()))?;
        let current = load_registrations(path)?;
        let merged = self.merged_registrations(current);
        let content = toml::to_string_pretty(&RegistryWire::from(&merged))
            .context("serializing registry TOML")?;

        lock.write_all(content.as_bytes()).with_context(|| {
            format!(
                "writing temporary locked registry {}",
                lock.lock_path().display()
            )
        })?;
        lock.flush().with_context(|| {
            format!(
                "flushing temporary locked registry {}",
                lock.lock_path().display()
            )
        })?;
        lock.with_mut(|file| file.sync_all()).with_context(|| {
            format!(
                "syncing temporary locked registry {}",
                lock.lock_path().display()
            )
        })?;
        lock.commit()
            .map_err(|error| error.error)
            .with_context(|| format!("committing registry {}", path.display()))?;

        Ok(())
    }

    fn merged_registrations(
        &self,
        mut current: BTreeMap<ProcessKey, Registration>,
    ) -> BTreeMap<ProcessKey, Registration> {
        let Some(loaded_base) = &self.loaded_base else {
            current.extend(self.registrations.clone());
            return current;
        };

        for key in loaded_base.keys() {
            if !self.registrations.contains_key(key) {
                current.remove(key);
            }
        }
        for (key, registration) in &self.registrations {
            if loaded_base.get(key) != Some(registration) {
                current.insert(*key, registration.clone());
            }
        }

        current
    }

    fn from_wire(wire: RegistryWire) -> anyhow::Result<Self> {
        let mut registrations = BTreeMap::new();
        for registration in wire.registration {
            let key = (registration.pid, registration.start_time);
            let parsed = Registration {
                agent_id: PrincipalId::new(registration.agent_id),
                registered_at: registration.registered_at,
                expires_at: registration.expires_at,
                registered_by: PrincipalId::new(registration.registered_by),
            };
            if registrations.insert(key, parsed).is_some() {
                return Err(anyhow!(
                    "duplicate registry registration for pid {} start_time {}",
                    key.0,
                    key.1
                ));
            }
        }
        Ok(Self::from_loaded(registrations))
    }

    fn from_loaded(registrations: BTreeMap<ProcessKey, Registration>) -> Self {
        Self {
            loaded_base: Some(registrations.clone()),
            registrations,
        }
    }
}

impl From<&str> for PrincipalId {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl From<String> for PrincipalId {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

fn load_registrations(path: &Path) -> anyhow::Result<BTreeMap<ProcessKey, Registration>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(BTreeMap::new());
        }
        Err(error) => {
            return Err(error).with_context(|| format!("reading registry {}", path.display()));
        }
    };

    let wire = toml::from_str::<RegistryWire>(&content)
        .with_context(|| format!("parsing registry {}", path.display()))?;
    Registry::from_wire(wire)
        .map(|registry| registry.registrations)
        .with_context(|| format!("loading registry {}", path.display()))
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryWire {
    #[serde(default)]
    registration: Vec<RegistrationWire>,
}

impl From<&BTreeMap<ProcessKey, Registration>> for RegistryWire {
    fn from(registrations: &BTreeMap<ProcessKey, Registration>) -> Self {
        Self {
            registration: registrations
                .iter()
                .map(|((pid, start_time), registration)| RegistrationWire {
                    pid: *pid,
                    start_time: *start_time,
                    agent_id: registration.agent_id.as_str().to_owned(),
                    registered_at: registration.registered_at,
                    expires_at: registration.expires_at,
                    registered_by: registration.registered_by.as_str().to_owned(),
                })
                .collect(),
        }
    }
}

impl From<&Registry> for RegistryWire {
    fn from(registry: &Registry) -> Self {
        Self::from(&registry.registrations)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistrationWire {
    pid: u32,
    start_time: u64,
    agent_id: String,
    registered_at: u64,
    expires_at: u64,
    registered_by: String,
}
