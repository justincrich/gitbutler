use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{
        REF_PIN_CAVEAT, group_add_member_with_repo, group_create_with_repo, group_delete_with_repo,
        group_grant_with_repo, group_list_with_repo, group_remove_member_with_repo,
        group_revoke_with_repo,
    },
};
use but_authz::{
    Authority, Denial, GroupName, PrincipalId, load_governance_config, permissions_path,
};

const MAIN_REF: &str = "refs/heads/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn group_ops_non_admin_denied_all_mutating_verbs() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let committed_before = committed_blob_text(&repo)?;

    let cases = [
        (
            "group_create",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_create_with_repo(&repo, MAIN_REF, "new-team", &["reviews:write"]),
            ),
        ),
        (
            "group_grant",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_grant_with_repo(&repo, MAIN_REF, "maintainers", &["comments:write"]),
            ),
        ),
        (
            "group_add_member",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_add_member_with_repo(&repo, MAIN_REF, "maintainers", "rust-implementer"),
            ),
        ),
        (
            "group_revoke",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_revoke_with_repo(&repo, MAIN_REF, "maintainers", &["merge"]),
            ),
        ),
        (
            "group_remove_member",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_remove_member_with_repo(&repo, MAIN_REF, "maintainers", "maint"),
            ),
        ),
    ];

    for (verb, result) in cases {
        let denial = structured_denial(result, verb)?;
        assert_eq!(
            denial.code,
            Denial::PERM_DENIED_CODE,
            "{verb} must return the stable perm.denied code"
        );
        assert!(
            denial.message.contains("administration:write"),
            "{verb} denial must name the missing administration:write authority"
        );
        assert!(
            !denial.remediation_hint.is_empty(),
            "{verb} denial must include an actionable remediation hint"
        );
        assert_eq!(
            worktree_permissions(&repo)?,
            committed_before,
            "denied {verb} must leave permissions.toml byte-for-byte unchanged"
        );
    }

    assert!(
        !worktree_permissions(&repo)?.contains("new-team"),
        "denied group_create must not leave a new-team group behind"
    );
    println!("all 4 mutating group verbs denied rust-reviewer with remediation hints");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_denials_include_remediation_hint() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let before = worktree_permissions(&repo)?;

    let cases = [
        (
            "group_create",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_create_with_repo(&repo, MAIN_REF, "new-team", &["reviews:write"]),
            ),
        ),
        (
            "group_grant",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_grant_with_repo(&repo, MAIN_REF, "maintainers", &["comments:write"]),
            ),
        ),
        (
            "group_add_member",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_add_member_with_repo(&repo, MAIN_REF, "maintainers", "rust-implementer"),
            ),
        ),
        (
            "group_revoke",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_revoke_with_repo(&repo, MAIN_REF, "maintainers", &["merge"]),
            ),
        ),
        (
            "group_remove_member",
            temp_env::with_vars(
                [
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                    ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
                    ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
                ],
                || group_remove_member_with_repo(&repo, MAIN_REF, "maintainers", "maint"),
            ),
        ),
    ];

    for (verb, result) in cases {
        let denial = structured_denial(result, verb)?;
        assert_eq!(
            denial.code,
            Denial::PERM_DENIED_CODE,
            "{verb} must return the stable perm.denied code"
        );
        assert!(
            denial.message.contains("administration:write"),
            "{verb} denial must name the missing administration:write authority"
        );
        assert!(
            !denial.remediation_hint.trim().is_empty(),
            "{verb} denial must include an actionable remediation hint"
        );
        assert_eq!(
            worktree_permissions(&repo)?,
            before,
            "denied {verb} must leave permissions.toml byte-for-byte unchanged"
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
fn group_remove_member_writes_worktree_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let remove = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_remove_member_with_repo(&repo, MAIN_REF, "maintainers", "rust-reviewer"),
    )?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let maintainers = group_block(&worktree_permissions, "maintainers")?;
    assert!(
        maintainers.contains("maint"),
        "remove-member must preserve other maintainers members"
    );
    assert!(
        !maintainers.contains("rust-reviewer"),
        "remove-member must remove the named member from the working-tree group"
    );
    assert!(
        format!("{remove:?}").contains(REF_PIN_CAVEAT),
        "remove-member result must include the ref-pin caveat"
    );

    let committed_after = load_governance_config(&repo, MAIN_REF)?;
    let committed_maintainers = committed_after
        .groups()
        .get(&GroupName::new("maintainers"))
        .ok_or_else(|| anyhow::anyhow!("committed maintainers group must still exist"))?;
    assert!(
        committed_maintainers
            .members()
            .contains(&PrincipalId::new("rust-reviewer")),
        "target-ref membership must remain unchanged until the working-tree edit is committed"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "group_remove_member must not commit or move refs/heads/main"
    );

    println!("remove-member removed rust-reviewer only from the working tree");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_grant_administration_write_delegates_admin_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let main_before = ref_id(&repo, MAIN_REF)?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_grant_with_repo(&repo, MAIN_REF, "maintainers", &["administration:write"]),
    )?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let maintainers = group_block(&worktree_permissions, "maintainers")?;
    assert!(
        maintainers.contains("administration:write"),
        "delegated admin grant must be written to the working-tree group"
    );

    let committed_after = load_governance_config(&repo, MAIN_REF)?;
    let committed_reviewers = committed_after
        .principal_authorities(&PrincipalId::new("rust-reviewer"))
        .ok_or_else(|| anyhow::anyhow!("rust-reviewer must exist in committed config"))?;
    assert!(
        !committed_reviewers.contains(Authority::AdministrationWrite),
        "working-tree delegated admin grant must not affect target-ref authority until committed"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "group_grant must not commit or move refs/heads/main"
    );

    println!("administration:write group grant was written but stayed inert");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_revoke_removes_direct_authority_and_preserves_members() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let outcome = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_revoke_with_repo(&repo, MAIN_REF, "maintainers", &["reviews:write"]),
    )?;

    let worktree_permissions = worktree_permissions(&repo)?;
    let maintainers = group_block(&worktree_permissions, "maintainers")?;
    assert!(
        !maintainers.contains("reviews:write"),
        "group_revoke must remove the requested direct authority from the group block"
    );
    assert!(
        maintainers.contains("merge"),
        "group_revoke must preserve unrelated direct group authorities"
    );
    assert!(
        maintainers.contains("maint") && maintainers.contains("rust-reviewer"),
        "group_revoke must preserve group members"
    );
    assert_eq!(
        outcome.authorities,
        vec!["reviews:write"],
        "group_revoke outcome must report the requested authority"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "group_revoke must not commit or move refs/heads/main"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn group_create_duplicate_errs_without_overwrite() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let before = worktree_permissions(&repo)?;

    let result = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_create_with_repo(&repo, MAIN_REF, "maintainers", &["reviews:write"]),
    );
    let denial = structured_denial(result, "duplicate group_create")?;
    assert_eq!(
        denial.code, "config.invalid",
        "duplicate group_create must return a stable config-invalid error"
    );
    assert!(
        !denial.remediation_hint.is_empty(),
        "duplicate group_create must include a remediation hint"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "duplicate group_create must not overwrite permissions.toml"
    );
    let maintainers = group_block(&before, "maintainers")?;
    assert!(
        maintainers.contains("merge") && maintainers.contains("rust-reviewer"),
        "duplicate group_create must preserve the existing group values"
    );
    assert_eq!(
        before.matches(r#"name = "maintainers""#).count(),
        1,
        "duplicate group_create must not append a second maintainers group"
    );

    println!("duplicate group_create returned config.invalid and left maintainers unchanged");
    Ok(())
}

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

    let add = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_add_member_with_repo(&repo, MAIN_REF, "code-reviewers", "rust-implementer"),
    )?;

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

    let create = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_create_with_repo(&repo, MAIN_REF, "release-captains", &["reviews:write"]),
    )?;
    assert_eq!(
        create.authorities,
        vec!["reviews:write"],
        "create outcome must report the create-time authority, not a later grant"
    );
    let after_create = worktree_permissions(&repo)?;
    assert!(
        group_block(&after_create, "release-captains")?.contains("reviews:write"),
        "group_create must write create-time authorities before any later group_grant"
    );
    assert!(
        !group_block(&after_create, "release-captains")?.contains("comments:write"),
        "test setup must prove comments:write arrives from group_grant, not group_create"
    );

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_grant_with_repo(&repo, MAIN_REF, "release-captains", &["comments:write"]),
    )?;

    let final_permissions = worktree_permissions(&repo)?;
    let release_captains = group_block(&final_permissions, "release-captains")?;
    assert!(
        release_captains.contains("reviews:write"),
        "group_create must keep the create-time authority in the created group block"
    );
    assert!(
        release_captains.contains("comments:write"),
        "group_grant must write the later authority into the created group block"
    );
    let duplicate_create = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_create_with_repo(&repo, MAIN_REF, "release-captains", &["statuses:write"]),
    );
    assert!(
        duplicate_create.is_err(),
        "creating an already-defined group must fail instead of silently succeeding"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        final_permissions,
        "duplicate group_create must not rewrite permissions.toml"
    );
    assert!(
        group_block(&final_permissions, "code-reviewers")?.contains(r#"role = "write""#),
        "group create/grant rewrite must preserve unrelated group role sugar by value"
    );
    assert!(
        principal_block(&final_permissions, "security-bot")?.contains("statuses:write"),
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

    let create_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_create_with_repo(
                &repo,
                MAIN_REF,
                "release-captains",
                &["reviews:write"],
            ))
        },
    )?;
    assert_perm_denied_administration_write(&create_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_create must not write permissions.toml"
    );

    let grant_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_grant_with_repo(
                &repo,
                MAIN_REF,
                "code-reviewers",
                &["administration:write"],
            ))
        },
    )?;
    assert_perm_denied_administration_write(&grant_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_grant must not write permissions.toml"
    );

    let add_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_add_member_with_repo(
                &repo,
                MAIN_REF,
                "code-reviewers",
                "rust-implementer",
            ))
        },
    )?;
    assert_perm_denied_administration_write(&add_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_add_member must not write permissions.toml"
    );

    let revoke_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_revoke_with_repo(
                &repo,
                MAIN_REF,
                "code-reviewers",
                &["reviews:write"],
            ))
        },
    )?;
    assert_perm_denied_administration_write(&revoke_error);
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied non-admin group_revoke must not write permissions.toml"
    );

    let remove_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_remove_member_with_repo(
                &repo,
                MAIN_REF,
                "code-reviewers",
                "rust-reviewer",
            ))
        },
    )?;
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
    let undefined = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_grant_with_repo(&repo, MAIN_REF, "undefined-group", &["reviews:write"]),
    );
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
    let bad_token = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_grant_with_repo(&repo, MAIN_REF, "code-reviewers", &["badtoken"]),
    );
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
    let unset_handle_error = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", None::<&str>),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            classified_error(group_grant_with_repo(
                &repo,
                MAIN_REF,
                "code-reviewers",
                &["reviews:write"],
            ))
        },
    )?;
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

    let list = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin-reader")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_list_with_repo(&repo, MAIN_REF),
    )?;
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

    let denied = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || classified_error(group_list_with_repo(&repo, MAIN_REF)),
    )?;
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

#[test]
#[serial_test::serial]
fn group_delete_removes_group_under_admin_write() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let outcome = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_delete_with_repo(&repo, MAIN_REF, "maintainers"),
    )?;

    let after = worktree_permissions(&repo)?;
    assert!(
        !after.contains(r#"name = "maintainers""#),
        "admin group_delete must remove the maintainers block from permissions.toml"
    );
    assert!(
        after.contains(r#"id = "admin""#),
        "admin group_delete must preserve the admin principal block"
    );
    assert!(
        format!("{outcome:?}").contains(REF_PIN_CAVEAT),
        "group_delete result must include the ref-pin caveat"
    );
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "group_delete must not commit or move refs/heads/main"
    );

    let committed_after = load_governance_config(&repo, MAIN_REF)?;
    assert!(
        committed_after
            .groups()
            .contains_key(&GroupName::new("maintainers")),
        "target-ref membership must remain unchanged until the working-tree delete is committed"
    );

    println!("admin group_delete removed maintainers from the working tree only");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_delete_non_admin_denied_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let before = worktree_permissions(&repo)?;

    let result = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-reviewer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_delete_with_repo(&repo, MAIN_REF, "maintainers"),
    );

    let denial = structured_denial(result, "group_delete")?;
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "group_delete must return the stable perm.denied code"
    );
    assert!(
        denial.message.contains("administration:write"),
        "group_delete denial must name the missing administration:write authority"
    );
    assert!(
        !denial.remediation_hint.is_empty(),
        "group_delete denial must include an actionable remediation hint"
    );
    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "denied group_delete must leave permissions.toml byte-for-byte unchanged"
    );

    println!("non-admin group_delete denied with remediation hint and wrote nothing");
    Ok(())
}

#[test]
#[serial_test::serial]
fn group_delete_missing_group_is_idempotent_noop() -> anyhow::Result<()> {
    let (repo, _tmp) = group_contract_base();
    let before = worktree_permissions(&repo)?;

    let outcome = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || group_delete_with_repo(&repo, MAIN_REF, "never-existed"),
    )?;

    assert_eq!(
        worktree_permissions(&repo)?,
        before,
        "deleting an undefined group must be an idempotent byte-stable no-op"
    );
    assert!(
        format!("{outcome:?}").contains(REF_PIN_CAVEAT),
        "idempotent group_delete must still return the ref-pin caveat"
    );

    println!("deleting an undefined group was byte-stable");
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

fn group_contract_base() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "rust-reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "rust-implementer"
role = "write"

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge", "reviews:write"]
members = ["maint", "rust-reviewer"]
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
        Err(error) => error.downcast::<Denial>().map_err(|error| {
            anyhow::anyhow!("{scenario} should return a structured error: {error}")
        }),
    }
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

fn committed_blob_text(repo: &gix::Repository) -> anyhow::Result<String> {
    let mut reference = repo.find_reference(MAIN_REF)?;
    let commit = reference.peel_to_commit()?;
    let tree = commit.tree()?;
    let entry = tree
        .lookup_entry_by_path(std::path::Path::new(permissions_path()))?
        .ok_or_else(|| anyhow::anyhow!("{} must exist at {MAIN_REF}", permissions_path()))?;
    let blob = repo.find_blob(entry.id())?;
    Ok(std::str::from_utf8(&blob.data)?.to_owned())
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
