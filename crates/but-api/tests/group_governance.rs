use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{
        REF_PIN_CAVEAT, group_add_member, group_create, group_grant, group_list,
        group_remove_member,
    },
};
use but_authz::{Authority, GroupName, PrincipalId, load_governance_config};

const MAIN_REF: &str = "refs/heads/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn group_add_member_writes_worktree_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = group_governance_base(true);
    let main_before = ref_id(&repo, MAIN_REF)?;
    let committed_before = load_governance_config(&repo, MAIN_REF)?;
    let committed_reviewers = committed_before
        .groups()
        .get(&GroupName::new("code-reviewers"))
        .expect("committed code-reviewers group must exist before mutation");
    assert!(
        !committed_reviewers
            .members()
            .contains(&PrincipalId::new("rust-implementer")),
        "fixture must not start with rust-implementer as a committed group member"
    );

    let add = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        group_add_member(&repo, MAIN_REF, "code-reviewers", "rust-implementer")
    })?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let reviewers = group_block(&worktree_permissions, "code-reviewers")?;
    assert!(
        reviewers.contains("rust-implementer"),
        "admin add-member must write membership into the working-tree group block"
    );
    assert!(
        format!("{add:?}").contains(REF_PIN_CAVEAT),
        "add-member result must include the ref-pin caveat"
    );

    let committed_after = load_governance_config(&repo, MAIN_REF)?;
    let committed_reviewers_after = committed_after
        .groups()
        .get(&GroupName::new("code-reviewers"))
        .expect("committed code-reviewers group must still exist after mutation");
    assert!(
        !committed_reviewers_after
            .members()
            .contains(&PrincipalId::new("rust-implementer")),
        "working-tree membership must be inert until committed to the target ref"
    );
    let committed_rust_implementer = committed_after
        .principal_authorities(&PrincipalId::new("rust-implementer"))
        .expect("rust-implementer must exist in committed target-ref config");
    assert!(
        committed_rust_implementer.contains(Authority::ContentsWrite),
        "committed direct authority must remain visible"
    );
    assert!(
        !committed_rust_implementer.contains(Authority::ReviewsWrite),
        "working-tree group membership must not change target-ref effective authority"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "group_add_member must not commit or move refs/heads/main"
    );

    println!("working-tree group membership changed while target-ref effective set stayed inert");
    println!("add-member result included `{REF_PIN_CAVEAT}`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_create_grant_writes_worktree() -> anyhow::Result<()> {
    let (repo, _tmp) = group_governance_base(true);

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        group_create(&repo, MAIN_REF, "release-captains")
    })?;
    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        group_grant(
            &repo,
            MAIN_REF,
            "release-captains",
            &["administration:write"],
        )
    })?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let release_captains = group_block(&worktree_permissions, "release-captains")?;
    assert!(
        release_captains.contains("administration:write"),
        "group_grant must write the new authority into the created group block"
    );
    assert!(
        group_block(&worktree_permissions, "code-reviewers")?.contains(r#"role = "write""#),
        "group create/grant rewrite must preserve unrelated group role sugar by value"
    );
    assert!(
        principal_block(&worktree_permissions, "security-bot")?.contains("statuses:write"),
        "group create/grant rewrite must preserve unrelated principal entries"
    );

    println!("group create and grant wrote a new group block while preserving unrelated TOML");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_ops_non_admin_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = group_governance_base(false);
    let before = worktree_permissions(&repo)?;

    let create_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(group_create(&repo, MAIN_REF, "release-captains"))
    })?;
    assert_perm_denied_administration_write(&create_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_create must not write permissions.toml"
    );

    let grant_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(group_grant(
            &repo,
            MAIN_REF,
            "code-reviewers",
            &["administration:write"],
        ))
    })?;
    assert_perm_denied_administration_write(&grant_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_grant must not write permissions.toml"
    );

    let add_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(group_add_member(
            &repo,
            MAIN_REF,
            "code-reviewers",
            "rust-implementer",
        ))
    })?;
    assert_perm_denied_administration_write(&add_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_add_member must not write permissions.toml"
    );

    let remove_error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(group_remove_member(
            &repo,
            MAIN_REF,
            "code-reviewers",
            "rust-reviewer",
        ))
    })?;
    assert_perm_denied_administration_write(&remove_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_remove_member must not write permissions.toml"
    );

    println!("non-admin group mutations denied with perm.denied and wrote nothing");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_grant_fail_closed_undefined_group_bad_token_and_unset_handle() -> anyhow::Result<()> {
    let (repo, _tmp) = group_governance_base(false);
    let before_undefined = worktree_permissions(&repo)?;
    let undefined = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        group_grant(&repo, MAIN_REF, "undefined-group", &["reviews:write"])
    });
    assert!(
        undefined.is_err(),
        "granting an undefined group must fail instead of auto-creating it"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before_undefined,
        "undefined-group grant must leave permissions.toml byte-for-byte unchanged"
    );

    let before_bad_token = worktree_permissions(&repo)?;
    let bad_token = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        group_grant(&repo, MAIN_REF, "code-reviewers", &["badtoken"])
    });
    assert!(
        bad_token.is_err(),
        "unparseable authority token must return an error"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before_bad_token,
        "bad-token group grant must leave permissions.toml byte-for-byte unchanged"
    );

    let before_unset_handle = worktree_permissions(&repo)?;
    let unset_handle_error = temp_env::with_var("BUT_AGENT_HANDLE", None::<&str>, || {
        classified_error(group_grant(
            &repo,
            MAIN_REF,
            "code-reviewers",
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
        "unset-handle group grant must leave permissions.toml byte-for-byte unchanged"
    );

    println!("undefined group, bad token, and unset handle failed closed without writes");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_list_under_admin_read() -> anyhow::Result<()> {
    let (repo, _tmp) = group_governance_base(true);

    let list = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin-reader"), || {
        group_list(&repo, MAIN_REF)
    })?;
    let list_text = format!("{list:?}");
    assert!(
        list_text.contains("code-reviewers"),
        "administration:read caller must see group names"
    );
    assert!(
        list_text.contains("reviews:write") && list_text.contains("contents:write"),
        "administration:read caller must see group grants, including role sugar grants"
    );
    assert!(
        list_text.contains("rust-reviewer"),
        "administration:read caller must see group members"
    );

    let denied = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(group_list(&repo, MAIN_REF))
    })?;
    assert_eq!(
        denied.code, "perm.denied",
        "caller without administration:read must be denied"
    );
    assert!(
        denied.message.contains("administration:read"),
        "group_list denial must name the missing administration:read authority"
    );

    println!("group list showed groups/grants/members under administration:read and denied others");
    Ok(())
}

fn group_governance_base(role_sugar_group: bool) -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    let code_reviewers_line = if role_sugar_group {
        r#"role = "write""#
    } else {
        r#"permissions = ["reviews:write"]"#
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
permissions = ["contents:write"]

[[principal]]
id = "security-bot"
permissions = ["statuses:write"]

[[group]]
name = "code-reviewers"
{code_reviewers_line}
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

fn classified_error<T>(result: anyhow::Result<T>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(_) => anyhow::bail!("governance group operation should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("governance group error should classify")),
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

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo.find_reference(ref_name)?;
    Ok(reference.peel_to_commit()?.id)
}
