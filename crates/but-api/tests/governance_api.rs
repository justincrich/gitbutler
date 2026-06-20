use but_api::legacy::governance::{
    REF_PIN_CAVEAT, governance_status_read, group_add_member, perm_grant,
};
use but_authz::Authority;
use serde_json::Value;

const MAIN_REF: &str = "refs/heads/main";
const TARGET_REF: &str = "refs/remotes/origin/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn governance_api_perm_grant_admin_lands_inert() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo();
    let ctx = context_for(&repo)?;
    let main_before = ref_id(&repo, MAIN_REF)?;

    let outcome = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant(
            &ctx,
            TARGET_REF.to_owned(),
            "rust-implementer".to_owned(),
            vec!["reviews:write".to_owned()],
        )
    })?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let rust_implementer = principal_block(&worktree_permissions, "rust-implementer")?;
    assert!(
        rust_implementer.contains("reviews:write"),
        "admin perm_grant must write reviews:write to rust-implementer's working-tree permissions"
    );

    let serialized = serde_json::to_value(outcome)?;
    assert_eq!(
        serialized.get("caveat").and_then(Value::as_str),
        Some(REF_PIN_CAVEAT),
        "GrantOutcome.caveat must survive serialization for API callers"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "perm_grant must leave refs/heads/main unmoved so the working-tree grant is inert"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_api_perm_grant_non_admin_denied_with_hint() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo();
    let ctx = context_for(&repo)?;
    let before = worktree_permissions_bytes(&repo)?;

    let result = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        perm_grant(
            &ctx,
            TARGET_REF.to_owned(),
            "rust-implementer".to_owned(),
            vec!["administration:write".to_owned()],
        )
    });

    let error = match result {
        Ok(_) => anyhow::bail!("non-admin self-grant through perm_grant must be denied"),
        Err(error) => error,
    };
    let error = json_error_value(error)?;
    assert_perm_denied_with_hint(&error, "administration:write");
    assert_eq!(
        worktree_permissions_bytes(&repo)?,
        before,
        "denied non-admin perm_grant must leave permissions.toml byte-for-byte unchanged"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_api_status_read_returns_own_effective_set() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo();
    let ctx = context_for(&repo)?;

    let effective = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        governance_status_read(&ctx)
    })?;

    assert!(
        effective.contains(Authority::ContentsWrite),
        "governance_status_read must include the caller's own contents:write authority"
    );
    assert!(
        effective.contains(Authority::PullRequestsWrite),
        "governance_status_read must include authority inherited from the caller's committed groups"
    );
    assert!(
        effective.len() >= 2,
        "status read must return the effective union, not only one side of own/group authority"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_api_group_add_member_non_admin_denied_with_hint() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo();
    let ctx = context_for(&repo)?;
    let before = worktree_permissions_bytes(&repo)?;

    let result = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-reviewer"), || {
        group_add_member(
            &ctx,
            TARGET_REF.to_owned(),
            "eng".to_owned(),
            "rust-reviewer".to_owned(),
        )
    });

    let error = match result {
        Ok(_) => anyhow::bail!("non-admin group_add_member must be denied"),
        Err(error) => error,
    };
    let error = json_error_value(error)?;
    assert_perm_denied_with_hint(&error, "administration:write");
    assert_eq!(
        worktree_permissions_bytes(&repo)?,
        before,
        "denied non-admin group_add_member must leave permissions.toml byte-for-byte unchanged"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn governance_api_status_read_is_self_scoped_no_foreign_principal() -> anyhow::Result<()> {
    let (repo, _tmp) = governance_api_repo();
    let ctx = context_for(&repo)?;

    let effective = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        governance_status_read(&ctx)
    })?;

    assert!(
        effective.contains(Authority::ContentsWrite),
        "self-scoped status must return rust-implementer's own effective set"
    );
    assert!(
        !effective.contains(Authority::AdministrationWrite),
        "self-scoped status must not leak admin's foreign administration:write authority"
    );
    assert_governance_status_read_has_no_foreign_principal_parameter()?;
    Ok(())
}

#[test]
fn governance_api_invariant_build_gate_covers_governance_boundary() -> anyhow::Result<()> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../but-authz/tests/invariant_build_gates.rs"),
    )?;
    assert!(
        source.contains(r#"const GOVERNANCE: &str = "crates/but-api/src/legacy/governance.rs";"#),
        "invariant_build_gates must name the governance boundary path explicitly"
    );
    let paths_start = source
        .find("const ENFORCEMENT_PATHS")
        .ok_or_else(|| anyhow::anyhow!("ENFORCEMENT_PATHS must exist in invariant_build_gates"))?;
    let paths_end = source[paths_start..]
        .find("];")
        .map(|offset| paths_start + offset)
        .ok_or_else(|| anyhow::anyhow!("ENFORCEMENT_PATHS must have a closing bracket"))?;
    assert!(
        source[paths_start..paths_end].contains("GOVERNANCE"),
        "ENFORCEMENT_PATHS must non-vacuously include governance.rs"
    );
    Ok(())
}

fn governance_api_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["reviews:write"]

[[group]]
name = "eng"
permissions = ["pull_requests:write"]
members = ["rust-implementer"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance api config"
git update-ref refs/remotes/origin/main refs/heads/main
"#,
        &repo,
    );
    (repo, tmp)
}

fn context_for(repo: &gix::Repository) -> anyhow::Result<but_ctx::Context> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    let mut project_meta = ctx.project_meta()?;
    project_meta.target_ref = Some(TARGET_REF.try_into()?);
    project_meta.target_commit_id = Some(ref_id(repo, TARGET_REF)?);
    ctx.set_project_meta(project_meta)?;
    Ok(ctx)
}

fn json_error_value(error: anyhow::Error) -> anyhow::Result<Value> {
    Ok(serde_json::to_value(but_api::json::Error::from(error))?)
}

fn assert_perm_denied_with_hint(error: &Value, missing: &str) {
    assert_eq!(
        error.get("code").and_then(Value::as_str),
        Some("perm.denied"),
        "governance API denial must serialize as code perm.denied"
    );
    assert!(
        error
            .get("message")
            .and_then(Value::as_str)
            .is_some_and(|message| message.contains(missing)),
        "serialized denial message must name the missing {missing} authority"
    );
    assert!(
        error
            .get("remediation_hint")
            .and_then(Value::as_str)
            .is_some_and(|hint| !hint.is_empty() && hint.contains(missing)),
        "serialized denial must include a non-empty remediation_hint naming {missing}"
    );
}

fn assert_governance_status_read_has_no_foreign_principal_parameter() -> anyhow::Result<()> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/legacy/governance.rs"),
    )?;
    let signature_start = source
        .find("fn governance_status_read")
        .ok_or_else(|| anyhow::anyhow!("governance_status_read must be defined"))?;
    let params_start = source[signature_start..]
        .find('(')
        .map(|offset| signature_start + offset)
        .ok_or_else(|| anyhow::anyhow!("governance_status_read must have parameters"))?;
    let params_end = source[params_start..]
        .find(')')
        .map(|offset| params_start + offset)
        .ok_or_else(|| anyhow::anyhow!("governance_status_read parameter list must close"))?;
    let params = &source[params_start + 1..params_end];
    assert!(
        params.contains("Context"),
        "governance_status_read must be scoped by Context"
    );
    for forbidden in ["principal", "handle", "subject"] {
        assert!(
            !params.contains(forbidden),
            "governance_status_read must not expose a foreign {forbidden} parameter"
        );
    }
    Ok(())
}

fn worktree_permissions(repo: &gix::Repository) -> anyhow::Result<String> {
    Ok(String::from_utf8(worktree_permissions_bytes(repo)?)?)
}

fn worktree_permissions_bytes(repo: &gix::Repository) -> anyhow::Result<Vec<u8>> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(std::fs::read(workdir.join(PERMISSIONS_PATH))?)
}

fn principal_block<'a>(toml: &'a str, id: &str) -> anyhow::Result<&'a str> {
    named_block(toml, "[[principal]]", "id", id)
}

fn named_block<'a>(toml: &'a str, header: &str, key: &str, value: &str) -> anyhow::Result<&'a str> {
    let marker = format!(r#"{key} = "{value}""#);
    toml.split(header)
        .skip(1)
        .find(|block| block.contains(&marker))
        .ok_or_else(|| anyhow::anyhow!("expected {header} block with {marker}"))
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo.find_reference(ref_name)?;
    Ok(reference.peel_to_commit()?.id)
}
