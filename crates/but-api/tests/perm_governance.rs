use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{
        REF_PIN_CAVEAT, perm_grant_with_repo, perm_list_with_repo, perm_revoke_with_repo,
    },
};
use but_authz::{Authority, Denial, PrincipalId, load_governance_config};

const MAIN_REF: &str = "refs/heads/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn perm_grant_writes_worktree_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);
    let main_before = ref_id(&repo, MAIN_REF)?;

    let grant = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant_with_repo(&repo, MAIN_REF, "rust-implementer", &["reviews:write"])
    })?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let rust_implementer = principal_block(&worktree_permissions, "rust-implementer")?;
    assert!(
        rust_implementer.contains("reviews:write"),
        "admin grant must write reviews:write under rust-implementer in the working tree"
    );
    assert!(
        format!("{grant:?}").contains(REF_PIN_CAVEAT),
        "grant result must include the ref-pin caveat"
    );

    let committed = load_governance_config(&repo, MAIN_REF)?;
    let committed_rust_implementer = committed
        .principal_authorities(&PrincipalId::new("rust-implementer"))
        .expect("rust-implementer must exist in the committed target-ref config");
    assert!(
        committed_rust_implementer.contains(Authority::ContentsWrite),
        "committed target-ref config still grants the original contents:write authority"
    );
    assert!(
        !committed_rust_implementer.contains(Authority::ReviewsWrite),
        "working-tree grant must be inert until committed to the target ref"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "perm_grant must not commit or move refs/heads/main"
    );

    println!("working-tree permissions.toml gained reviews:write for rust-implementer");
    println!("target-ref effective authority remains unchanged until committed");
    println!("grant result included `{REF_PIN_CAVEAT}`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_grant_preserves_unrelated_entries_and_role_sugar() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(true);

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant_with_repo(&repo, MAIN_REF, "rust-implementer", &["merge"])
    })?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let rust_implementer = principal_block(&worktree_permissions, "rust-implementer")?;
    assert!(
        rust_implementer.contains(r#"role = "write""#),
        "read-modify-write must preserve rust-implementer's role=\"write\" sugar"
    );
    assert!(
        rust_implementer.contains("merge"),
        "grant must add merge without replacing the existing role sugar"
    );
    assert!(
        principal_block(&worktree_permissions, "security-bot")?.contains("statuses:write"),
        "unrelated security-bot principal must survive the write"
    );
    assert!(
        group_block(&worktree_permissions, "code-reviewers")?.contains("reviews:write"),
        "unrelated code-reviewers group and its reviews:write grant must survive the write"
    );

    println!("role sugar and unrelated entries survived the grant rewrite");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_first_grant_seeds_principal_and_authorizes_when_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_seed();
    let before = load_governance_config(&repo, MAIN_REF)?;
    assert!(
        before
            .principal_authorities(&PrincipalId::new("rust-implementer"))
            .is_none(),
        "seed fixture must start with no rust-implementer principal"
    );

    let grant = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant_with_repo(&repo, MAIN_REF, "rust-implementer", &["reviews:write"])
    })?;
    let worktree_permissions = worktree_permissions(&repo)?;
    assert!(
        principal_block(&worktree_permissions, "admin")?.contains("administration:write"),
        "seeding a new principal must preserve the existing admin block"
    );
    assert!(
        principal_block(&worktree_permissions, "rust-implementer")?.contains("reviews:write"),
        "first grant must register rust-implementer with reviews:write"
    );
    assert!(
        format!("{grant:?}").contains(REF_PIN_CAVEAT),
        "seeding grant result must include the ref-pin caveat"
    );

    but_testsupport::invoke_bash(
        r#"
git add .gitbutler/permissions.toml
git commit -m "seed rust-implementer"
"#,
        &repo,
    );

    let after_commit = load_governance_config(&repo, MAIN_REF)?;
    let rust_implementer = after_commit
        .principal_authorities(&PrincipalId::new("rust-implementer"))
        .expect("committed seeded principal must load from target-ref governance config");
    assert!(
        rust_implementer.contains(Authority::ReviewsWrite),
        "seeded principal must authorize for reviews:write once committed"
    );

    println!("first grant seeded rust-implementer and authorized after commit");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_revoke_removes_token_and_idempotent_noop() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);

    let revoke = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_revoke_with_repo(&repo, MAIN_REF, "rust-implementer", &["contents:write"])
    })?;
    let after_revoke = worktree_permissions(&repo)?;
    let rust_implementer = principal_block(&after_revoke, "rust-implementer")?;
    assert!(
        !rust_implementer.contains("contents:write"),
        "revoke must remove contents:write from rust-implementer"
    );
    assert!(
        after_revoke.contains(r#"id = "rust-implementer""#),
        "revoke must remove only the token, not the principal entry"
    );
    assert!(
        principal_block(&after_revoke, "security-bot")?.contains("statuses:write"),
        "unrelated security-bot principal must survive revoke"
    );
    assert!(
        group_block(&after_revoke, "code-reviewers")?.contains("reviews:write"),
        "unrelated code-reviewers group must survive revoke"
    );
    assert!(
        format!("{revoke:?}").contains(REF_PIN_CAVEAT),
        "revoke result must include the ref-pin caveat"
    );

    let before_noop = worktree_permissions(&repo)?;
    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_revoke_with_repo(&repo, MAIN_REF, "rust-implementer", &["merge"])
    })?;
    assert_eq!(
        worktree_permissions(&repo)?,
        before_noop,
        "revoking a token the principal does not hold must be an idempotent byte-stable no-op"
    );

    println!("revoke removed a held token and not-held revoke was byte-stable");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_grant_revoke_non_admin_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);
    let before = worktree_permissions(&repo)?;

    let grant_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(perm_grant_with_repo(
            &repo,
            MAIN_REF,
            "rust-implementer",
            &["administration:write"],
        ))
    })?;
    assert_perm_denied_administration_write(&grant_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin grant must not write permissions.toml"
    );

    let revoke_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(perm_revoke_with_repo(
            &repo,
            MAIN_REF,
            "admin",
            &["administration:write"],
        ))
    })?;
    assert_perm_denied_administration_write(&revoke_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin revoke must not write permissions.toml"
    );

    println!("non-admin grant and revoke denied with perm.denied and wrote nothing");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_grant_fail_closed_bad_token_and_unset_handle() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);
    let before_bad_token = worktree_permissions(&repo)?;
    let bad_token = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant_with_repo(&repo, MAIN_REF, "rust-implementer", &["badtoken"])
    });
    assert!(
        bad_token.is_err(),
        "unparseable authority token must return an error, not silently skip"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before_bad_token,
        "bad-token grant must leave permissions.toml byte-for-byte unchanged"
    );

    let before_unset_handle = worktree_permissions(&repo)?;
    let unset_handle_error = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        classified_error(perm_grant_with_repo(
            &repo,
            MAIN_REF,
            "rust-implementer",
            &["reviews:write"],
        ))
    })?;
    assert_eq!(
        unset_handle_error.code, "perm.denied",
        "unset BUT_AGENT_HANDLE must fail closed as perm.denied"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before_unset_handle,
        "unset-handle grant must leave permissions.toml byte-for-byte unchanged"
    );

    println!("bad token and unset handle both failed closed without writes");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_revoke_fail_closed_bad_token() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_remediation_base();
    let before = worktree_permissions(&repo)?;

    let error = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_revoke_with_repo(&repo, MAIN_REF, "rust-implementer", &["not a permission"])
    })
    .expect_err("bad permission token must reject before mutation");
    let invalid = error.downcast::<but_api::json::ConfigInvalid>()?;
    assert_eq!(
        invalid.code, "config.invalid",
        "bad permission token must classify as config.invalid"
    );
    assert!(
        invalid.message.contains("not a permission"),
        "invalid token message must include the rejected token, got: {}",
        invalid.message
    );
    assert!(
        !invalid.remediation_hint.trim().is_empty(),
        "invalid token must include remediation guidance"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "bad-token revoke must leave permissions.toml byte-for-byte unchanged"
    );

    println!(
        "seeded invalid-token response: code={}, message={}, remediation_hint={}",
        invalid.code, invalid.message, invalid.remediation_hint
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_revoke_fail_closed_unset_handle() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_remediation_base();
    let before = worktree_permissions(&repo)?;

    let denial = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        structured_denial(
            perm_revoke_with_repo(&repo, MAIN_REF, "rust-implementer", &["reviews:write"]),
            "unset-handle perm_revoke",
        )
    })?;
    assert_denial_payload(
        &denial,
        "BUT_AGENT_HANDLE",
        "unset-handle revoke must return a structured identity denial",
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "unset-handle revoke must leave permissions.toml byte-for-byte unchanged"
    );

    println!(
        "seeded revoke denial: code={}, message={}, remediation_hint={}",
        denial.code, denial.message, denial.remediation_hint
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_list_fail_closed_unset_handle() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_remediation_base();

    let denial = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        structured_denial(
            perm_list_with_repo(&repo, MAIN_REF, Some("rust-implementer")),
            "unset-handle perm_list",
        )
    })?;
    assert_denial_payload(
        &denial,
        "BUT_AGENT_HANDLE",
        "unset-handle list must return a structured identity denial",
    );
    assert!(
        !denial.message.contains("reviews:write")
            && !denial.remediation_hint.contains("reviews:write"),
        "unset-handle list denial must not reveal rust-implementer authorities: {denial:?}"
    );

    println!(
        "seeded list denial: code={}, message={}, remediation_hint={}",
        denial.code, denial.message, denial.remediation_hint
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_denials_include_remediation_hint() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_remediation_base();

    let grant = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        structured_denial(
            perm_grant_with_repo(&repo, MAIN_REF, "rust-reviewer", &["reviews:write"]),
            "perm_grant",
        )
    })?;
    let revoke = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        structured_denial(
            perm_revoke_with_repo(&repo, MAIN_REF, "admin", &["administration:write"]),
            "perm_revoke",
        )
    })?;
    let list = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        structured_denial(
            perm_list_with_repo(&repo, MAIN_REF, Some("rust-reviewer")),
            "perm_list",
        )
    })?;

    for (verb, denial) in [
        ("perm_grant", grant),
        ("perm_revoke", revoke),
        ("perm_list", list),
    ] {
        assert_denial_payload(
            &denial,
            "administration",
            &format!("{verb} denial must include code, message, and remediation_hint"),
        );
        println!(
            "seeded {verb} denial: code={}, message={}, remediation_hint={}",
            denial.code, denial.message, denial.remediation_hint
        );
    }

    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_list_cross_principal_scoping() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);

    let self_list = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        perm_list_with_repo(&repo, MAIN_REF, None)
    })?;
    let self_text = format!("{self_list:?}");
    assert!(
        self_text.contains("contents:write"),
        "self-read must show rust-implementer's actual contents:write authority"
    );

    let cross_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(perm_list_with_repo(&repo, MAIN_REF, Some("maint")))
    })?;
    assert_eq!(
        cross_error.code, "perm.denied",
        "resolved non-admin cross-principal list must be denied as the scope decision"
    );
    assert!(
        !cross_error.message.contains("merge"),
        "cross-principal denial must not leak maint's merge authority"
    );

    let admin_list = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin-reader"), || {
        perm_list_with_repo(&repo, MAIN_REF, Some("maint"))
    })?;
    assert!(
        format!("{admin_list:?}").contains("merge"),
        "administration:read holder must be allowed to list another principal"
    );

    println!("self-read allowed, cross-principal non-admin denied, admin-read allowed");
    Ok(())
}

#[test]
#[serial_test::serial]
fn perm_list_pending_marks_uncommitted_grant() -> anyhow::Result<()> {
    let (repo, _tmp) = perm_governance_base(false);

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_grant_with_repo(&repo, MAIN_REF, "rust-implementer", &["reviews:write"])
    })?;
    let list = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        perm_list_with_repo(&repo, MAIN_REF, Some("rust-implementer"))
    })?;
    let list_text = format!("{list:?}");

    assert!(
        list_text.contains("contents:write"),
        "list must include committed contents:write"
    );
    assert!(
        list_text.contains("reviews:write") && list_text.contains("PENDING"),
        "list must mark the uncommitted reviews:write working-tree grant as PENDING"
    );
    assert!(
        !authority_line(&list_text, "contents:write").contains("PENDING"),
        "committed contents:write must not be marked PENDING"
    );

    println!("perm_list rendered committed authority plus pending working-tree grant");
    Ok(())
}

fn perm_governance_base(role_sugar: bool) -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let rust_implementer_line = if role_sugar {
        r#"role = "write""#
    } else {
        r#"permissions = ["contents:write"]"#
    };
    but_testsupport::invoke_bash(
        &format!(
            r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "admin-reader"
permissions = ["administration:read"]

[[principal]]
id = "rust-implementer"
{rust_implementer_line}

[[principal]]
id = "security-bot"
permissions = ["statuses:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["rust-reviewer"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
"#
        ),
        &repo,
    );
    (repo, tmp)
}

fn perm_governance_seed() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config (admin only)"
"#,
        &repo,
    );
    (repo, tmp)
}

fn perm_remediation_base() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]

[[principal]]
id = "rust-implementer"
permissions = ["reviews:write"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["rust-reviewer"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn structured_denial<T>(result: anyhow::Result<T>, scenario: &str) -> anyhow::Result<Denial> {
    match result {
        Ok(_) => anyhow::bail!("{scenario} should reject this scenario"),
        Err(error) => error
            .downcast::<Denial>()
            .map_err(|error| anyhow::anyhow!("{scenario} should return Denial: {error}")),
    }
}

fn classified_error<T>(result: anyhow::Result<T>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(_) => anyhow::bail!("governance permission operation should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("governance permission error should classify")),
    }
}

fn assert_perm_denied_administration_write(error: &AdminWriteGateError) {
    assert_eq!(
        error.code, "perm.denied",
        "non-admin governance writes must be denied with perm.denied"
    );
    assert!(
        error.message.contains("administration:write"),
        "denial message must name the missing administration:write authority"
    );
}

fn assert_denial_payload(denial: &Denial, expected_message: &str, reason: &str) {
    assert_eq!(denial.code, Denial::PERM_DENIED_CODE, "{reason}");
    assert!(
        denial.message.contains(expected_message),
        "{reason}; message must contain {expected_message:?}, got: {}",
        denial.message
    );
    assert!(
        !denial.remediation_hint.trim().is_empty(),
        "{reason}; remediation_hint must be non-empty"
    );
}

fn worktree_permissions(repo: &gix::Repository) -> anyhow::Result<String> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(std::fs::read_to_string(workdir.join(PERMISSIONS_PATH))?)
}

fn principal_block<'a>(toml: &'a str, id: &str) -> anyhow::Result<&'a str> {
    named_block(toml, "[[principal]]", "id", id)
}

fn group_block<'a>(toml: &'a str, name: &str) -> anyhow::Result<&'a str> {
    named_block(toml, "[[group]]", "name", name)
}

fn named_block<'a>(toml: &'a str, header: &str, key: &str, value: &str) -> anyhow::Result<&'a str> {
    let marker = format!(r#"{key} = "{value}""#);
    toml.split(header)
        .skip(1)
        .find(|block| block.contains(&marker))
        .ok_or_else(|| anyhow::anyhow!("expected {header} block with {marker}"))
}

fn authority_line<'a>(text: &'a str, authority: &str) -> &'a str {
    text.lines()
        .find(|line| line.contains(authority))
        .unwrap_or_default()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo.find_reference(ref_name)?;
    Ok(reference.peel_to_commit()?.id)
}
