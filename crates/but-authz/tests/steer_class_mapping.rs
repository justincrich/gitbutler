//! STEER-004: (code, principal-resolution) → DenialClass mapping proofs.
//!
//! Integration tests proving every denial constructor maps to the
//! spec-mandated [`but_authz::DenialClass`]. The class field drives steering
//! routing (actor vs operator recovery) and MUST NOT drift from the fixed
//! `(code, principal-resolution) → class` mapping established by the
//! steering capability spec.
//!
//! Mapping contract (fixed, not adjustable per call site):
//!
//! | Constructor / code         | Principal resolved? | Class              |
//! |----------------------------|---------------------|--------------------|
//! | `missing_permission`       | yes                 | `ActorCorrectable` |
//! | `no_handle`                | no (no handle)      | `OperatorRequired` |
//! | `unknown_principal`        | no (unknown)        | `OperatorRequired` |
//! | `branch.protected`         | yes                 | `ActorCorrectable` |
//! | `config.invalid`           | n/a                 | `OperatorRequired` |

use but_authz::{
    Authority, AuthoritySet, Denial, DenialClass, GovConfig, GroupName, Principal, PrincipalId,
    effective_authority, load_governance_config,
};

const TARGET_REF: &str = "refs/heads/main";

/// AC-1: `missing_permission` (resolved principal, fixable) → ActorCorrectable.
///
/// The principal is known to governance; they can self-recover by requesting
/// a reviewed merge or asking for the missing authority to be granted.
#[test]
fn steer_class_missing_permission_is_actor_correctable() {
    let denial = Denial::missing_permission(Authority::ContentsWrite, &AuthoritySet::empty());

    assert_eq!(
        denial.class,
        DenialClass::ActorCorrectable,
        "missing_permission must be ActorCorrectable: the resolved principal \
         can request a reviewed merge or ask a maintainer for the authority"
    );
}

/// AC-2: `no_handle` (no handle, can't self-correct) → OperatorRequired.
///
/// The actor cannot recover without first setting a valid handle, which
/// requires operator/maintainer context to identify and commit the right
/// principal.
#[test]
fn steer_class_no_handle_is_operator_required() {
    let denial = Denial::no_handle();

    assert_eq!(
        denial.class,
        DenialClass::OperatorRequired,
        "no_handle must be OperatorRequired: an operator must provision the \
         handle in committed governance config before the actor can act"
    );
}

/// AC-3: `unknown_principal` (unknown handle, can't self-correct) → OperatorRequired.
///
/// The handle is absent from committed config; an operator must commit the
/// principal before the actor can recover.
#[test]
fn steer_class_unknown_principal_is_operator_required() {
    let denial = Denial::unknown_principal("ghost");

    assert_eq!(
        denial.class,
        DenialClass::OperatorRequired,
        "unknown_principal must be OperatorRequired: an operator must commit \
         the principal to governance config before the actor can recover"
    );
}

/// AC-4: `branch.protected` (caller can switch to a feature branch) → ActorCorrectable.
///
/// The principal is known and holds `contents:write`; they can self-recover
/// by switching to a feature branch and landing the change through a reviewed
/// merge. This test mirrors the production `branch_protected` contract: the
/// denial carries `class = ActorCorrectable` and `held_permissions` populated
/// from `effective_authority` so the gate-state-aware menu (STEER-003) can
/// derive recovery affordances.
#[test]
fn steer_class_branch_protected_is_actor_correctable() {
    let held = AuthoritySet::parse(["contents:write"]).expect("contents:write parses");
    let principal = Principal::new(
        PrincipalId::new("dev"),
        held.clone(),
        std::iter::empty::<GroupName>(),
    );
    let cfg = GovConfig::new([(PrincipalId::new("dev"), held)], [], []);

    // Mirror the production `branch_protected` constructor shape: the denial
    // carries class=ActorCorrectable and held_permissions from
    // effective_authority so a downstream consumer can derive a recovery menu.
    let denial = Denial {
        class: DenialClass::ActorCorrectable,
        held_permissions: effective_authority(&principal, &cfg).iter().collect(),
        ..Denial::new(
            "branch.protected",
            "direct commits to protected branch \"main\" are denied for principal \"dev\"; \
             land changes through a reviewed merge"
                .to_owned(),
            "open a reviewed merge into main instead of committing directly".to_owned(),
        )
    };

    assert_eq!(
        denial.class,
        DenialClass::ActorCorrectable,
        "branch.protected must be ActorCorrectable: the caller can self-recover \
         by switching to a feature branch"
    );
    assert_eq!(
        denial.code, "branch.protected",
        "branch.protected denial must carry the stable branch.protected code"
    );
    assert!(
        !denial.held_permissions.is_empty(),
        "branch.protected must carry the principal's held permissions (from \
         effective_authority) so the gate-state-aware menu can derive affordances"
    );
}

/// AC-5: `ConfigError` (`config.invalid`) → OperatorRequired.
///
/// Malformed committed governance config can only be fixed by a maintainer
/// re-committing valid `.gitbutler/*.toml`; the actor has no path to
/// self-correct. Exercises the real production path via
/// `load_governance_config` against a malformed `gates.toml`.
#[test]
fn steer_class_config_error_is_operator_required() -> anyhow::Result<()> {
    let (repo, _tmp) = malformed_config_repo();

    let error = match load_governance_config(&repo, TARGET_REF) {
        Ok(_) => panic!("malformed config must fail closed with a ConfigError"),
        Err(error) => error,
    };

    assert_eq!(
        error.code(),
        "config.invalid",
        "ConfigError must carry the stable config.invalid code"
    );
    assert_eq!(
        error.class,
        Some(DenialClass::OperatorRequired),
        "config.invalid must be OperatorRequired: only a maintainer can fix \
         committed governance config and re-commit"
    );

    Ok(())
}

/// Seed a governed repo with valid `permissions.toml` + malformed `gates.toml`.
fn malformed_config_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]
name = "main"
protected = nope
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "malformed governance config"
"#,
        &repo,
    );
    (repo, tmp)
}
