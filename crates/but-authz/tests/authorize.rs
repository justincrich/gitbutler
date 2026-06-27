#![allow(non_snake_case)]

use but_authz::{
    Authority, AuthoritySet, Denial, GovConfig, GroupName, Principal, PrincipalId, authorize,
    effective_authority, load_governance_config, resolve_principal, resolve_principal_from_env,
};
use std::ffi::OsString;
use std::sync::Mutex;

const TARGET_REF: &str = "refs/heads/main";

// `resolve_principal_from_env` reads the real process environment, so a local
// lock keeps the env-mutating positive case serialized within this binary.
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn authorize_held_vs_missing() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let dev = principal(&config, "dev")?;
    let ro = principal(&config, "ro")?;

    authorize(&dev, Authority::ContentsWrite, &config)?;
    println!("`authorize(dev, ContentsWrite, cfg)` returns `Ok(())`");

    let denial = assert_denied(authorize(&ro, Authority::ContentsWrite, &config));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "missing contents:write must use the stable perm.denied code"
    );
    assert!(
        denial.message.contains("contents:write"),
        "missing-permission denial must name contents:write in the message"
    );
    assert!(
        denial.remediation_hint.contains("reviewed merge"),
        "remediation hint must point at a reviewed merge path"
    );
    println!("`code == \"perm.denied\"`");
    println!("`message` contains \"contents:write\"");
    println!("`remediation_hint` contains \"reviewed merge\"");

    Ok(())
}

#[test]
fn resolve_no_handle_rejected() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;

    let unset = assert_no_principal_denied(resolve_principal(|_| None, &config));
    assert_eq!(
        unset.code,
        Denial::PERM_DENIED_CODE,
        "unset BUT_AGENT_HANDLE must be denied with perm.denied"
    );
    println!("`resolve_principal` returns `Err(Denial)`");
    println!("no principal resolved (`code == \"perm.denied\"`)");

    let empty = assert_no_principal_denied(resolve_principal(
        |key| (key == "BUT_AGENT_HANDLE").then(OsString::new),
        &config,
    ));
    assert_eq!(
        empty.code,
        Denial::PERM_DENIED_CODE,
        "empty BUT_AGENT_HANDLE must be denied with perm.denied"
    );
    println!("`resolve_principal` returns `Err(Denial)` for an empty-string handle");
    println!("`code == \"perm.denied\"` (no principal resolved)");

    Ok(())
}

#[test]
fn resolve_unknown_principal_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;

    let denial = assert_no_principal_denied(resolve_principal(
        |key| (key == "BUT_AGENT_HANDLE").then(|| OsString::from("ghost")),
        &config,
    ));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "unknown handles must fail closed with perm.denied"
    );
    assert!(
        denial.message.contains("ghost"),
        "unknown-principal denial must name the unresolved handle"
    );
    println!("`code == \"perm.denied\"`");
    println!("principal \"ghost\" not found");

    Ok(())
}

#[test]
fn effective_authority_union() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let reviewer = Principal::new(
        PrincipalId::new("reviewer"),
        AuthoritySet::empty(),
        [GroupName::new("code-reviewers")],
    );

    let held = effective_authority(&reviewer, &config);
    assert!(
        held.contains(Authority::ReviewsWrite),
        "reviewer must inherit reviews:write from code-reviewers"
    );
    authorize(&reviewer, Authority::ReviewsWrite, &config)?;
    println!("`authorize(reviewer, ReviewsWrite, cfg)` returns `Ok(())`");

    let denial = assert_denied(authorize(&reviewer, Authority::Merge, &config));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "reviewer must not inherit merge authority"
    );
    println!("`authorize(reviewer, Merge, cfg)` returns `code == \"perm.denied\"`");

    Ok(())
}

#[test]
fn authority_only_from_config() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let ro_with_claim = Principal::new(
        PrincipalId::new("ro"),
        AuthoritySet::parse(["contents:write"])?,
        std::iter::empty(),
    );

    let denial = assert_denied(authorize(&ro_with_claim, Authority::ContentsWrite, &config));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "caller-supplied authority claims must not widen ro beyond committed config"
    );
    assert!(
        denial.message.contains("contents:write"),
        "denial must name the rejected contents:write authority"
    );
    println!(
        "`authorize(ro, ContentsWrite, cfg)` returns `code == \"perm.denied\"` - authority sourced only from `cfg`"
    );

    Ok(())
}

#[test]
fn env_primary_handle_resolves_committed_principal() -> anyhow::Result<()> {
    // Env-primary identity: a `BUT_AGENT_HANDLE` naming a committed principal
    // resolves to that principal at the gate. This is the production path —
    // `resolve_principal_from_env` is what every governed `but-api` gate calls.
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_agent_handle(Some("rust-implementer"), || {
        let (repo, _tmp) = governed_repo();
        let config = load_governance_config(&repo, TARGET_REF)?;

        let principal = assert_principal_resolved(
            resolve_principal_from_env(&config),
            "a committed BUT_AGENT_HANDLE must resolve to its principal",
        );

        assert_eq!(
            principal.id().as_str(),
            "rust-implementer",
            "the resolved principal must be the one named by BUT_AGENT_HANDLE"
        );
        Ok(())
    })
}

#[test]
fn env_primary_unset_handle_denies() -> anyhow::Result<()> {
    // Fail closed: an unset `BUT_AGENT_HANDLE` resolves no principal and is
    // denied with the stable `perm.denied` code (`Denial::no_handle`).
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_agent_handle(None, || {
        let (repo, _tmp) = governed_repo();
        let config = load_governance_config(&repo, TARGET_REF)?;

        let denial = assert_no_principal_denied(resolve_principal_from_env(&config));
        assert_eq!(
            denial.code,
            Denial::PERM_DENIED_CODE,
            "an unset BUT_AGENT_HANDLE must fail closed with perm.denied"
        );
        Ok(())
    })
}

fn governed_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]

[[principal]]
id = "reviewer"
groups = ["code-reviewers"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["reviewer"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "principals"
"#,
        &repo,
    );
    (repo, tmp)
}

fn with_agent_handle(
    agent_handle: Option<&str>,
    test: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let previous_agent = std::env::var_os("BUT_AGENT_HANDLE");
    set_env_var("BUT_AGENT_HANDLE", agent_handle);
    let result = test();
    restore_env_var("BUT_AGENT_HANDLE", previous_agent);
    result
}

fn set_env_var(key: &str, value: Option<&str>) {
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

fn restore_env_var(key: &str, value: Option<OsString>) {
    unsafe {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }
}

fn principal(config: &GovConfig, id: &str) -> anyhow::Result<Principal> {
    let principal_id = PrincipalId::new(id);
    let authorities = config
        .principal_authorities(&principal_id)
        .ok_or_else(|| anyhow::anyhow!("principal {id} must be present in committed config"))?;
    Ok(Principal::new(
        principal_id,
        authorities.clone(),
        std::iter::empty(),
    ))
}

fn assert_denied(result: Result<(), Denial>) -> Denial {
    match result {
        Ok(()) => panic!("authorization must deny the missing permission"),
        Err(denial) => denial,
    }
}

fn assert_no_principal_denied(result: Result<Principal, Denial>) -> Denial {
    match result {
        Ok(principal) => panic!(
            "resolve_principal must not resolve default or anonymous principal `{}`",
            principal.id().as_str()
        ),
        Err(denial) => denial,
    }
}

fn assert_principal_resolved(result: Result<Principal, Denial>, context: &str) -> Principal {
    match result {
        Ok(principal) => principal,
        Err(denial) => panic!("{context}: {} {}", denial.code, denial.message),
    }
}

