#![allow(non_snake_case)]

use std::{
    fs,
    sync::{Arc, Barrier},
    thread,
};

use anyhow::Context;
use but_authz::{PrincipalId, ProcessKey, Registry};
use but_testsupport::gix_testtools::tempfile::TempDir;

#[test]
fn IDENT_001_empty_registry_resolves_no_principal() {
    assert_eq!(Registry::empty().resolve((1, 1)), None);
}

#[test]
fn IDENT_001_register_resolve_round_trip_uses_pid_and_start_time() {
    let mut registry = Registry::empty();

    registry
        .register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")
        .expect("registering a live process identity should succeed");

    assert_eq!(
        registry.len(),
        1,
        "a successful registration must store exactly one process identity"
    );
    assert_eq!(
        registry.resolve((1234, 1_730_000_000)),
        Some(PrincipalId::new("rust-implementer")),
        "registered identity should resolve for the exact pid/start_time pair"
    );
}

#[test]
fn IDENT_001_same_pid_with_different_start_time_does_not_resolve() {
    let mut registry = Registry::empty();

    registry
        .register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")
        .expect("registering a live process identity should succeed");

    assert_eq!(
        registry.resolve((1234, 1_730_000_099)),
        None,
        "pid reuse must not resolve when the process start_time differs"
    );
    assert_eq!(
        registry.resolve((1234, 1_730_000_000)),
        Some(PrincipalId::new("rust-implementer")),
        "rejecting a reused pid with a different start_time must not remove the original entry"
    );
}

#[test]
fn IDENT_001_unregister_removes_exact_pid_start_time_pair() {
    let mut registry = Registry::empty();
    registry
        .register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")
        .expect("registering a live process identity should succeed");
    registry
        .register(1234, 1_730_000_099, "reviewer", 14_400, "operator")
        .expect("same pid with a distinct start_time should be a distinct registration");

    let removed = registry
        .unregister((1234, 1_730_000_000))
        .expect("unregister should remove the exact process identity");

    assert_eq!(removed.agent_id, PrincipalId::new("rust-implementer"));
    assert_eq!(
        registry.resolve((1234, 1_730_000_000)),
        None,
        "unregistered process identity must no longer resolve"
    );
    assert_eq!(
        registry.resolve((1234, 1_730_000_099)),
        Some(PrincipalId::new("reviewer")),
        "unregistering one start_time must not remove a reused pid with a different start_time"
    );
}

#[test]
fn IDENT_001_gc_keeps_expiry_boundary_and_drops_afterwards() {
    let mut registry = Registry::empty();
    registry
        .register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")
        .expect("registering a live process identity should succeed");

    assert_eq!(
        registry.gc(1_730_014_400),
        0,
        "expires_at is the last valid second, so gc at the boundary must keep the entry"
    );
    assert_eq!(
        registry.resolve((1234, 1_730_000_000)),
        Some(PrincipalId::new("rust-implementer")),
        "the entry must remain resolvable at its expiry boundary"
    );

    assert_eq!(
        registry.gc(1_730_014_401),
        1,
        "gc after the expiry boundary must drop the expired entry"
    );
    assert_eq!(
        registry.resolve((1234, 1_730_000_000)),
        None,
        "expired entries must not resolve after gc drops them"
    );
}

#[test]
fn IDENT_001_write_then_load_round_trips_parseable_toml() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");
    let mut registry = Registry::empty();
    registry.register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")?;

    registry.write(&path)?;

    let loaded = Registry::load(&path)?;
    assert_eq!(
        loaded, registry,
        "loading the registry TOML must preserve every registration field"
    );

    let content = fs::read_to_string(&path)?;
    let parsed = toml::from_str::<toml::Value>(&content)?;
    let registration = parsed
        .get("registration")
        .and_then(toml::Value::as_array)
        .and_then(|registrations| registrations.first())
        .and_then(toml::Value::as_table)
        .expect("registry TOML must contain a [[registration]] table");
    assert_eq!(registration["pid"].as_integer(), Some(1234));
    assert_eq!(registration["start_time"].as_integer(), Some(1_730_000_000));
    assert_eq!(registration["agent_id"].as_str(), Some("rust-implementer"));
    assert_eq!(
        registration["registered_at"].as_integer(),
        Some(1_730_000_000)
    );
    assert_eq!(registration["expires_at"].as_integer(), Some(1_730_014_400));
    assert_eq!(registration["registered_by"].as_str(), Some("operator"));

    Ok(())
}

#[test]
fn IDENT_004_registry_write_then_load_round_trips_three_distinct_entries_exact_fields()
-> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");
    let mut registry = Registry::empty();
    let expected = [
        ExpectedRegistration::new(1234, 900, "rust-implementer", 100, "operator-a"),
        ExpectedRegistration::new(2345, 1_730_000_000, "rust-reviewer", 14_400, "operator-b"),
        ExpectedRegistration::new(3456, 1_730_001_000, "release-agent", 86_400, "operator-c"),
    ];

    for entry in &expected {
        registry.register(
            entry.key.0,
            entry.key.1,
            entry.agent_id.clone(),
            entry.ttl_seconds,
            entry.registered_by.clone(),
        )?;
    }

    registry.write(&path)?;
    let loaded = Registry::load(&path)?;

    assert_eq!(
        loaded, registry,
        "loading the registry TOML must preserve the exact three-entry registry"
    );
    assert_eq!(
        loaded.len(),
        expected.len(),
        "the loaded registry must contain all three distinct registrations"
    );
    for entry in &expected {
        assert_registration_fields(&loaded, entry);
    }

    Ok(())
}

#[test]
fn IDENT_004_gc_keeps_expiry_boundary_and_drops_afterwards() -> anyhow::Result<()> {
    let mut registry = Registry::empty();
    let entry = ExpectedRegistration::new(55, 900, "ttl-agent", 100, "operator");
    registry.register(
        entry.key.0,
        entry.key.1,
        entry.agent_id.clone(),
        entry.ttl_seconds,
        entry.registered_by.clone(),
    )?;

    assert_registration_fields(&registry, &entry);
    assert_eq!(
        registry.gc(999),
        0,
        "gc before expires_at=1000 must keep the registration"
    );
    assert_eq!(
        registry.resolve(entry.key),
        Some(PrincipalId::new("ttl-agent")),
        "the registration must remain resolvable before the expiry boundary"
    );
    assert_eq!(
        registry.gc(1000),
        0,
        "gc at expires_at=1000 must keep the registration"
    );
    assert_eq!(
        registry.resolve(entry.key),
        Some(PrincipalId::new("ttl-agent")),
        "the registration must remain resolvable at the expiry boundary"
    );
    assert_eq!(
        registry.gc(1001),
        1,
        "gc after expires_at=1000 must drop the expired registration"
    );
    assert_eq!(
        registry.resolve(entry.key),
        None,
        "the registration must no longer resolve after gc drops it"
    );

    Ok(())
}

#[test]
fn IDENT_004_concurrent_registry_writes_preserve_all_distinct_entries_and_parse()
-> anyhow::Result<()> {
    const WRITER_COUNT: u32 = 8;

    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");
    Registry::empty().write(&path)?;

    let expected = (0..WRITER_COUNT)
        .map(|index| {
            ExpectedRegistration::new(
                20_000 + index,
                1_800_000_000 + u64::from(index),
                format!("agent-{index}"),
                3_600 + u64::from(index),
                format!("operator-{index}"),
            )
        })
        .collect::<Vec<_>>();
    let start_barrier = Arc::new(Barrier::new(WRITER_COUNT as usize));
    let loaded_barrier = Arc::new(Barrier::new(WRITER_COUNT as usize));
    let handles = expected
        .iter()
        .cloned()
        .map(|entry| {
            let path = path.clone();
            let start_barrier = Arc::clone(&start_barrier);
            let loaded_barrier = Arc::clone(&loaded_barrier);
            thread::spawn(move || -> anyhow::Result<()> {
                start_barrier.wait();
                let loaded = Registry::load(&path);
                loaded_barrier.wait();

                let mut registry = loaded?;
                registry.register(
                    entry.key.0,
                    entry.key.1,
                    entry.agent_id.clone(),
                    entry.ttl_seconds,
                    entry.registered_by.clone(),
                )?;
                registry.write(&path)?;
                Ok(())
            })
        })
        .collect::<Vec<_>>();

    let mut writer_errors = Vec::new();
    for handle in handles {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => writer_errors.push(format!("{error:#}")),
            Err(payload) => writer_errors.push(thread_panic_message(payload)),
        }
    }

    let content = fs::read_to_string(&path).with_context(|| {
        format!(
            "reading registry after concurrent writes: {}",
            path.display()
        )
    })?;
    toml::from_str::<toml::Value>(&content).with_context(|| {
        format!(
            "parsing registry after concurrent writes: {}",
            path.display()
        )
    })?;

    let loaded = Registry::load(&path)?;
    let missing = expected
        .iter()
        .filter(|entry| !registration_matches(&loaded, entry))
        .map(ExpectedRegistration::description)
        .collect::<Vec<_>>();
    let loaded_keys = loaded.registrations().keys().copied().collect::<Vec<_>>();

    assert!(
        writer_errors.is_empty() && missing.is_empty(),
        "concurrent registry writers must all succeed and preserve every distinct registration; \
         writer_errors={writer_errors:?}; missing={missing:?}; loaded_keys={loaded_keys:?}"
    );

    Ok(())
}

#[test]
fn IDENT_004_unregister_write_removes_exact_key_without_resurrecting_via_merge()
-> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");
    let original = ExpectedRegistration::new(30_000, 1_900_000_000, "original", 600, "operator-a");
    let kept = ExpectedRegistration::new(30_001, 1_900_000_001, "kept", 600, "operator-a");
    let concurrent =
        ExpectedRegistration::new(30_002, 1_900_000_002, "concurrent", 600, "operator-b");

    let mut initial = Registry::empty();
    for entry in [&original, &kept] {
        initial.register(
            entry.key.0,
            entry.key.1,
            entry.agent_id.clone(),
            entry.ttl_seconds,
            entry.registered_by.clone(),
        )?;
    }
    initial.write(&path)?;

    let mut stale = Registry::load(&path)?;
    let removed = stale
        .unregister(original.key)
        .expect("the stale registry must remove the exact original key");
    assert_eq!(
        removed.agent_id.as_str(),
        original.agent_id.as_str(),
        "unregister must return the removed original registration"
    );

    let mut concurrent_writer = Registry::load(&path)?;
    concurrent_writer.register(
        concurrent.key.0,
        concurrent.key.1,
        concurrent.agent_id.clone(),
        concurrent.ttl_seconds,
        concurrent.registered_by.clone(),
    )?;
    concurrent_writer.write(&path)?;

    stale.write(&path)?;
    let loaded = Registry::load(&path)?;

    assert_eq!(
        loaded.resolve(original.key),
        None,
        "three-way merge must not resurrect a key removed from the loaded base"
    );
    assert!(
        registration_matches(&loaded, &kept),
        "writing the stale unregister must keep unchanged base entries"
    );
    assert!(
        registration_matches(&loaded, &concurrent),
        "writing the stale unregister must preserve concurrent additions outside its base deletion"
    );

    Ok(())
}

#[test]
fn IDENT_004_stale_add_write_does_not_resurrect_concurrently_unregistered_base_key()
-> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");
    let removed = ExpectedRegistration::new(31_000, 1_900_001_000, "removed", 600, "operator-a");
    let kept = ExpectedRegistration::new(31_001, 1_900_001_001, "kept", 600, "operator-a");
    let added = ExpectedRegistration::new(31_002, 1_900_001_002, "added", 600, "operator-b");

    let mut initial = Registry::empty();
    for entry in [&removed, &kept] {
        initial.register(
            entry.key.0,
            entry.key.1,
            entry.agent_id.clone(),
            entry.ttl_seconds,
            entry.registered_by.clone(),
        )?;
    }
    initial.write(&path)?;

    let mut stale_add_writer = Registry::load(&path)?;
    stale_add_writer.register(
        added.key.0,
        added.key.1,
        added.agent_id.clone(),
        added.ttl_seconds,
        added.registered_by.clone(),
    )?;

    let mut unregister_writer = Registry::load(&path)?;
    unregister_writer
        .unregister(removed.key)
        .expect("the concurrent writer must remove the base key");
    unregister_writer.write(&path)?;

    stale_add_writer.write(&path)?;
    let loaded = Registry::load(&path)?;

    assert_eq!(
        loaded.resolve(removed.key),
        None,
        "a stale add-only write must not reapply an unchanged base entry removed by another writer"
    );
    assert!(
        registration_matches(&loaded, &kept),
        "a stale add-only write must leave unrelated base entries intact"
    );
    assert!(
        registration_matches(&loaded, &added),
        "a stale add-only write must still persist its local addition"
    );

    Ok(())
}

#[test]
fn IDENT_001_load_missing_file_returns_empty_registry() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents.toml");

    let registry = Registry::load(&path)?;

    assert!(
        registry.is_empty(),
        "missing registry files must be treated as an empty registry"
    );

    Ok(())
}

#[derive(Debug, Clone)]
struct ExpectedRegistration {
    key: ProcessKey,
    agent_id: String,
    ttl_seconds: u64,
    expires_at: u64,
    registered_by: String,
}

impl ExpectedRegistration {
    fn new(
        pid: u32,
        start_time: u64,
        agent_id: impl Into<String>,
        ttl_seconds: u64,
        registered_by: impl Into<String>,
    ) -> Self {
        Self {
            key: (pid, start_time),
            agent_id: agent_id.into(),
            ttl_seconds,
            expires_at: start_time + ttl_seconds,
            registered_by: registered_by.into(),
        }
    }

    fn description(&self) -> String {
        format!(
            "pid={} start_time={} agent_id={} expires_at={} registered_by={}",
            self.key.0, self.key.1, self.agent_id, self.expires_at, self.registered_by
        )
    }
}

fn assert_registration_fields(registry: &Registry, expected: &ExpectedRegistration) {
    let registration = registry
        .registrations()
        .get(&expected.key)
        .unwrap_or_else(|| panic!("registration must exist for {:?}", expected.key));

    assert_eq!(
        registration.agent_id.as_str(),
        expected.agent_id.as_str(),
        "agent_id must round-trip exactly for {:?}",
        expected.key
    );
    assert_eq!(
        registration.registered_at, expected.key.1,
        "registered_at must round-trip exactly for {:?}",
        expected.key
    );
    assert_eq!(
        registration.expires_at, expected.expires_at,
        "expires_at must round-trip exactly for {:?}",
        expected.key
    );
    assert_eq!(
        registration.registered_by.as_str(),
        expected.registered_by.as_str(),
        "registered_by must round-trip exactly for {:?}",
        expected.key
    );
}

fn registration_matches(registry: &Registry, expected: &ExpectedRegistration) -> bool {
    registry
        .registrations()
        .get(&expected.key)
        .is_some_and(|registration| {
            registration.agent_id.as_str() == expected.agent_id.as_str()
                && registration.registered_at == expected.key.1
                && registration.expires_at == expected.expires_at
                && registration.registered_by.as_str() == expected.registered_by.as_str()
        })
}

fn thread_panic_message(payload: Box<dyn std::any::Any + Send + 'static>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        format!("writer thread panicked: {message}")
    } else if let Some(message) = payload.downcast_ref::<String>() {
        format!("writer thread panicked: {message}")
    } else {
        "writer thread panicked with non-string payload".to_owned()
    }
}

#[cfg(unix)]
#[test]
fn H1a_write_creates_runtime_registry_file_mode_0600() -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let tmp = TempDir::new()?;
    let path = tmp.path().join("agents-runtime.toml");
    let mut registry = Registry::empty();
    registry.register(1234, 1_730_000_000, "rust-implementer", 14_400, "operator")?;

    registry.write(&path)?;

    let mode = fs::metadata(&path)?.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o600,
        "the runtime registry is the spoofing trust root; it must be created owner-only (0600), got {mode:o}"
    );

    // Re-writing an existing registry must preserve the 0600 mode across the
    // atomic rename, not inherit the default umask of the replacement temp file.
    registry.register(2345, 1_730_000_100, "rust-reviewer", 14_400, "operator")?;
    registry.write(&path)?;
    let mode_after = fs::metadata(&path)?.permissions().mode() & 0o777;
    assert_eq!(
        mode_after, 0o600,
        "re-writing the registry must keep it owner-only (0600), got {mode_after:o}"
    );

    Ok(())
}

#[test]
fn IDENT_001_write_missing_parent_returns_error_naming_path() -> anyhow::Result<()> {
    let tmp = TempDir::new()?;
    let path = tmp.path().join("missing").join("agents.toml");

    let error = Registry::empty()
        .write(&path)
        .expect_err("writing to a missing parent directory must fail");

    assert!(
        error.to_string().contains(&path.display().to_string()),
        "write errors must name the target path: {error:#}"
    );
    assert!(
        !path.exists(),
        "failed atomic write must not leave a partial target registry file"
    );

    Ok(())
}
