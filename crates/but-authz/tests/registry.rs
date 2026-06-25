use but_authz::{PrincipalId, Registry};

#[test]
fn IDENT_001_empty_registry_resolves_no_principal() {
    assert_eq!(Registry::empty().resolve((1, 1)), None);
}

#[test]
fn IDENT_001_register_resolve_round_trip_uses_pid_and_start_time() {
    let mut registry = Registry::empty();

    registry
        .register(
            1234,
            1_730_000_000,
            PrincipalId::new("rust-implementer"),
            14_400,
            PrincipalId::new("operator"),
        )
        .expect("registering a live process identity should succeed");

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
        .register(
            1234,
            1_730_000_000,
            PrincipalId::new("rust-implementer"),
            14_400,
            PrincipalId::new("operator"),
        )
        .expect("registering a live process identity should succeed");

    assert_eq!(
        registry.resolve((1234, 1_730_000_099)),
        None,
        "pid reuse must not resolve when the process start_time differs"
    );
}
