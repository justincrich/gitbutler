use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};

use crate::PrincipalId;

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
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Registry {
    registrations: BTreeMap<ProcessKey, Registration>,
}

impl Registry {
    /// Create an empty runtime registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Load a registry from a TOML file, treating a missing file as empty.
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self::empty());
            }
            Err(error) => {
                return Err(error).with_context(|| format!("reading registry {}", path.display()));
            }
        };

        let wire = toml::from_str::<RegistryWire>(&content)
            .with_context(|| format!("parsing registry {}", path.display()))?;
        Self::from_wire(wire).with_context(|| format!("loading registry {}", path.display()))
    }

    /// Write the registry to a TOML file via same-directory temp file + rename.
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
        let temp_path = temp_path_for(path)?;
        let content = toml::to_string_pretty(&RegistryWire::from(self))
            .context("serializing registry TOML")?;

        let write_result = (|| -> anyhow::Result<()> {
            let mut file = File::create(&temp_path)
                .with_context(|| format!("creating temporary registry {}", temp_path.display()))?;
            file.write_all(content.as_bytes())
                .with_context(|| format!("writing temporary registry {}", temp_path.display()))?;
            file.sync_all()
                .with_context(|| format!("syncing temporary registry {}", temp_path.display()))?;
            drop(file);
            fs::rename(&temp_path, path).with_context(|| {
                format!(
                    "renaming temporary registry {} to {}",
                    temp_path.display(),
                    path.display()
                )
            })?;
            Ok(())
        })();

        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }

        write_result
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
        Ok(Self { registrations })
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

fn temp_path_for(path: &Path) -> anyhow::Result<PathBuf> {
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow!("registry path {} has no file name", path.display()))?;
    let temp_file_name = format!("{}.tmp.{}", file_name.to_string_lossy(), std::process::id());
    Ok(path.with_file_name(temp_file_name))
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RegistryWire {
    #[serde(default)]
    registration: Vec<RegistrationWire>,
}

impl From<&Registry> for RegistryWire {
    fn from(registry: &Registry) -> Self {
        Self {
            registration: registry
                .registrations
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
