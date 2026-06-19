use but_authz::{
    Authority, AuthoritySet, Denial, GroupName, Principal, PrincipalId, authorize,
    effective_authority, load_governance_config,
};

const TARGET_REF: &str = "refs/heads/main";

#[test]
fn group_union_authorizes_review_denies_merge() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let reviewer = bare_principal("reviewer-only");

    authorize(&reviewer, Authority::ReviewsWrite, &config)?;
    println!("`authorize(reviewer-only, ReviewsWrite, cfg)` returns `Ok(())`");

    let denial = assert_denied(authorize(&reviewer, Authority::Merge, &config));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "unsourced merge must fail closed with perm.denied"
    );
    assert!(
        denial.message.contains("merge"),
        "missing-permission denial must name merge in the message"
    );
    println!("`authorize(reviewer-only, Merge, cfg)` returns `code == \"perm.denied\"`");
    println!("`Merge denial.message` contains \"merge\"");

    Ok(())
}

#[test]
fn union_paths_stay_equal() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let reviewer_only = bare_principal("reviewer-only");
    let reviewer_byref = Principal::new(
        PrincipalId::new("reviewer-byref"),
        AuthoritySet::empty(),
        [GroupName::new("code-reviewers")],
    );
    let ro = bare_principal("ro");

    assert_effective_equals_loaded(&reviewer_only, &config);
    assert_effective_equals_loaded(&reviewer_byref, &config);
    assert_effective_equals_loaded(&ro, &config);

    assert_reviews_only(
        config
            .principal_authorities(reviewer_only.id())
            .ok_or_else(|| anyhow::anyhow!("reviewer-only must be folded into principal set"))?,
        "reviewer-only must resolve solely through group members=[...]",
    );
    assert_reviews_only(
        config
            .principal_authorities(reviewer_byref.id())
            .ok_or_else(|| anyhow::anyhow!("reviewer-byref must be folded into principal set"))?,
        "reviewer-byref must resolve solely through principal groups=[...]",
    );
    assert!(
        !config
            .principal_authorities(ro.id())
            .ok_or_else(|| anyhow::anyhow!("ro must be present in committed config"))?
            .contains(Authority::ReviewsWrite),
        "read-only control must not inherit reviews:write"
    );
    println!("effective_authority(reviewer-only) == principal_authorities(reviewer-only)");
    println!("effective_authority(reviewer-byref) == principal_authorities(reviewer-byref)");
    println!("both reviewer paths contain exactly reviews:write; ro excludes reviews:write");

    Ok(())
}

#[test]
fn delegated_admin_ceiling() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let delegate = bare_principal("delegate");
    let reviewer = bare_principal("reviewer-only");

    authorize(&delegate, Authority::AdministrationWrite, &config)?;
    println!("`authorize(delegate, AdministrationWrite, cfg)` returns `Ok(())`");

    let denial = assert_denied(authorize(
        &reviewer,
        Authority::AdministrationWrite,
        &config,
    ));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "non-admin reviewer must not receive delegated admin authority"
    );
    assert!(
        denial.message.contains("administration:write"),
        "missing-permission denial must name administration:write in the message"
    );
    println!(
        "`authorize(reviewer-only, AdministrationWrite, cfg)` returns `code == \"perm.denied\"`"
    );

    Ok(())
}

#[test]
fn claims_do_not_widen_union_even_with_group_backing() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let config = load_governance_config(&repo, TARGET_REF)?;
    let fabricated_merge = Principal::new(
        PrincipalId::new("reviewer-only"),
        AuthoritySet::parse(["merge"])?,
        [GroupName::new("code-reviewers")],
    );
    let fabricated_admin = Principal::new(
        PrincipalId::new("reviewer-only"),
        AuthoritySet::parse(["administration:write"])?,
        std::iter::empty(),
    );

    let merge_denial = assert_denied(authorize(&fabricated_merge, Authority::Merge, &config));
    assert_eq!(
        merge_denial.code,
        Denial::PERM_DENIED_CODE,
        "caller-supplied merge claim must not widen committed group-backed authority"
    );
    assert!(
        merge_denial.message.contains("merge"),
        "missing-permission denial must name merge in the message"
    );

    let admin_denial = assert_denied(authorize(
        &fabricated_admin,
        Authority::AdministrationWrite,
        &config,
    ));
    assert_eq!(
        admin_denial.code,
        Denial::PERM_DENIED_CODE,
        "caller-supplied admin claim must not widen the cfg lookup for reviewer-only"
    );
    println!("fabricated merge claim for reviewer-only is denied with perm.denied");
    println!("fabricated administration:write claim for reviewer-only is denied with perm.denied");

    Ok(())
}

fn governed_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    if std::env::var_os("AUTHZ_EMPTY_START").is_some() {
        return (repo, tmp);
    }

    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "ro"
permissions = ["contents:read"]

[[principal]]
id = "reviewer-byref"
groups = ["code-reviewers"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["reviewer-only"]

[[group]]
name = "config-admins"
permissions = ["administration:write"]
members = ["delegate"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "group union config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn bare_principal(id: &str) -> Principal {
    Principal::new(
        PrincipalId::new(id),
        AuthoritySet::empty(),
        std::iter::empty(),
    )
}

fn assert_effective_equals_loaded(principal: &Principal, config: &but_authz::GovConfig) {
    let expected = config
        .principal_authorities(principal.id())
        .cloned()
        .unwrap_or_else(AuthoritySet::empty);
    assert_eq!(
        effective_authority(principal, config),
        expected,
        "effective authority must be the already-folded principal_authorities set"
    );
}

fn assert_reviews_only(authorities: &AuthoritySet, message: &str) {
    assert_eq!(authorities.len(), 1, "{message}");
    assert!(
        authorities.contains(Authority::ReviewsWrite),
        "{message}: reviews:write must be present"
    );
}

fn assert_denied(result: Result<(), Denial>) -> Denial {
    match result {
        Ok(()) => panic!("authorization must deny the missing permission"),
        Err(denial) => denial,
    }
}
