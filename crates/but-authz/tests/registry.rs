#![allow(non_snake_case)]

use std::fs;

use but_authz::{PrincipalId, Registry};
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
