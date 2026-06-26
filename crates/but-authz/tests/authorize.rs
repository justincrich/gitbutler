use but_authz::{
    Authority, AuthoritySet, Denial, GovConfig, GroupName, Principal, PrincipalId, Registry,
    authorize, current_pid, effective_authority, load_governance_config, process_start_time,
    resolve_principal, resolve_principal_with_registry,
};
use std::ffi::OsString;
use std::sync::Mutex;

const TARGET_REF: &str = "refs/heads/main";

// These are the only process-environment mutations in this integration test
// binary, so a local lock keeps the IDENT-003 fallback cases serialized without
// adding crate-local env-mutation dependencies.
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
fn IDENT_003_registry_hit_resolves_registered_principal() -> anyhow::Result<()> {
    let config = ident_003_config()?;
    let pid = current_pid();
    let start_time = process_start_time(pid)?;
    let mut registry = Registry::empty();
    registry.register(pid, start_time, "rust-implementer", 60, "rust-implementer")?;

    let principal = resolve_principal_with_registry(Some(&registry), &config)?;

    assert_eq!(
        principal.id().as_str(),
        "rust-implementer",
        "registry hit for the current (pid, start_time) must resolve the registered principal"
    );

    Ok(())
}

#[test]
fn IDENT_003_registry_miss_allows_flagged_env_fallback() -> anyhow::Result<()> {
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_authz_env(Some("1"), Some("dev"), || {
        let config = ident_003_config()?;
        let registry = Registry::empty();

        let principal = resolve_principal_with_registry(Some(&registry), &config)?;

        assert_eq!(
            principal.id().as_str(),
            "dev",
            "registry miss with BUT_AUTHZ_ALLOW_ENV_HANDLE=1 must fall through to BUT_AGENT_HANDLE"
        );
        Ok(())
    })
}

#[test]
fn IDENT_003_registry_miss_without_env_flag_denies_unregistered() -> anyhow::Result<()> {
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_authz_env(None, None, || {
        let config = ident_003_config()?;
        let registry = Registry::empty();

        let denial =
            assert_no_principal_denied(resolve_principal_with_registry(Some(&registry), &config));

        assert_eq!(
            denial.code,
            Denial::PERM_DENIED_CODE,
            "unregistered process denials must use the stable perm.denied code"
        );
        assert!(
            denial.message.to_lowercase().contains("unregistered"),
            "unregistered denial message must name the unregistered process: {}",
            denial.message
        );
        assert!(
            denial
                .remediation_hint
                .to_lowercase()
                .contains("registration"),
            "unregistered denial remediation must point at registration: {}",
            denial.remediation_hint
        );
        Ok(())
    })
}

#[test]
fn IDENT_003_stale_registry_entry_denies_pid_reuse() -> anyhow::Result<()> {
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_authz_env(None, None, || {
        let config = ident_003_config()?;
        let pid = current_pid();
        let current_start_time = process_start_time(pid)?;
        let stale_start_time = if current_start_time == u64::MAX {
            current_start_time - 1
        } else {
            current_start_time + 1
        };
        let mut registry = Registry::empty();
        registry.register(
            pid,
            stale_start_time,
            "rust-implementer",
            60,
            "rust-implementer",
        )?;

        let denial =
            assert_no_principal_denied(resolve_principal_with_registry(Some(&registry), &config));

        assert_eq!(
            denial.code,
            Denial::PERM_DENIED_CODE,
            "stale registration denials must use the stable perm.denied code"
        );
        let message = denial.message.to_lowercase();
        assert!(
            message.contains("stale"),
            "stale registration denial message must name stale state: {}",
            denial.message
        );
        assert!(
            message.contains("pid reuse"),
            "stale registration denial message must name pid reuse: {}",
            denial.message
        );
        assert!(
            message.contains(&pid.to_string()) || message.contains("current pid"),
            "stale registration denial message must name the current pid: {}",
            denial.message
        );
        Ok(())
    })
}

#[test]
fn IDENT_003_none_registry_allows_flagged_env_fallback() -> anyhow::Result<()> {
    let _guard = ENV_LOCK.lock().expect("env lock must not be poisoned");
    with_authz_env(Some("1"), Some("dev"), || {
        let config = ident_003_config()?;

        let principal = resolve_principal_with_registry(None, &config)?;

        assert_eq!(
            principal.id().as_str(),
            "dev",
            "None registry with BUT_AUTHZ_ALLOW_ENV_HANDLE=1 must fall through to BUT_AGENT_HANDLE"
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

fn ident_003_config() -> anyhow::Result<GovConfig> {
    Ok(GovConfig::new(
        [
            (
                PrincipalId::new("rust-implementer"),
                AuthoritySet::parse(["contents:write"])?,
            ),
            (
                PrincipalId::new("dev"),
                AuthoritySet::parse(["contents:write"])?,
            ),
        ],
        [],
        [],
    ))
}

fn with_authz_env(
    allow_env_handle: Option<&str>,
    agent_handle: Option<&str>,
    test: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    let previous_allow = std::env::var_os("BUT_AUTHZ_ALLOW_ENV_HANDLE");
    let previous_agent = std::env::var_os("BUT_AGENT_HANDLE");

    set_env_var("BUT_AUTHZ_ALLOW_ENV_HANDLE", allow_env_handle);
    set_env_var("BUT_AGENT_HANDLE", agent_handle);
    let result = test();
    restore_env_var("BUT_AUTHZ_ALLOW_ENV_HANDLE", previous_allow);
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
