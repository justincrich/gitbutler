use std::ffi::OsString;

use but_authz::{
    Authority, Denial, GroupName, Principal, PrincipalId, authorize, effective_authority,
    load_governance_config, resolve_principal,
};

const TARGET_REF: &str = "refs/heads/main";

#[test]
fn self_grant_admin_inert_until_landed() -> anyhow::Result<()> {
    let (repo, _tmp) = self_grant_admin_repo();
    let main_before = ref_id(&repo, TARGET_REF)?;
    let config = load_governance_config(&repo, TARGET_REF)?;
    let feat_author = principal(&config, "feat-author")?;

    let denial = assert_denied(authorize(
        &feat_author,
        Authority::AdministrationWrite,
        &config,
    ));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "feature-head administration:write self-grant must be inert before landing"
    );
    assert!(
        denial.message.contains("administration:write"),
        "before-landing denial must name administration:write"
    );

    land_admin_write(&repo);
    let main_after = ref_id(&repo, TARGET_REF)?;
    assert_ne!(
        main_before, main_after,
        "landing administration:write grant must advance refs/heads/main"
    );

    let config = load_governance_config(&repo, TARGET_REF)?;
    let feat_author = principal(&config, "feat-author")?;
    authorize(&feat_author, Authority::AdministrationWrite, &config)?;

    println!("before landing: administration:write self-grant returns perm.denied");
    println!("after landing: administration:write authorizes from refs/heads/main");

    Ok(())
}

#[test]
fn membership_read_only_from_target_ref() -> anyhow::Result<()> {
    let (repo, _tmp) = self_escalation_membership_repo();
    but_testsupport::invoke_bash(
        r#"
git checkout feat
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint", "feat-author"]
EOF
"#,
        &repo,
    );

    let config = load_governance_config(&repo, TARGET_REF)?;
    let maintainers = config
        .groups()
        .get(&GroupName::new("maintainers"))
        .ok_or_else(|| anyhow::anyhow!("target-ref maintainers group must exist"))?;
    assert!(
        maintainers.members().contains(&PrincipalId::new("maint")),
        "target-ref maintainers group must retain committed maint member"
    );
    assert!(
        !maintainers
            .members()
            .contains(&PrincipalId::new("feat-author")),
        "target-ref maintainers group must ignore feature-head and working-tree feat-author"
    );

    let feat_author = principal(&config, "feat-author")?;
    let held = effective_authority(&feat_author, &config);
    assert!(
        !held.contains(Authority::Merge),
        "feat-author must not receive merge from feature-head or working-tree maintainers edits"
    );

    println!("target-ref maintainers include maint");
    println!("target-ref maintainers exclude feat-author while HEAD and working tree include it");
    println!("feat-author effective target-ref authority excludes merge");

    Ok(())
}

fn self_grant_admin_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "target governance excludes admin grant"
git checkout -b feat-admin
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write", "administration:write"]
EOF
git add .gitbutler/permissions.toml
git commit -m "self grant administration write on feature head"
"#,
        &repo,
    );
    // FIX-GRPS-002-AC3-TEETH: leave HEAD on feat-admin so the negative control
    // has teeth. A HEAD-peel mutation in load_governance_config would read the
    // feat-admin tree (which carries the self-grant) and authorize when it
    // should deny. Only the target-ref peel (`refs/heads/main`) reads the
    // pre-grant config and correctly denies.
    (repo, tmp)
}

fn self_escalation_membership_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "target governance excludes feat author from maintainers"
git checkout -b feat
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maint", "feat-author"]
EOF
git add .gitbutler/permissions.toml
git commit -m "feature head self adds maintainers membership"
"#,
        &repo,
    );
    (repo, tmp)
}

fn land_admin_write(repo: &gix::Repository) {
    but_testsupport::invoke_bash(
        r#"
git checkout main
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "feat-author"
permissions = ["contents:write", "administration:write"]
EOF
git add .gitbutler/permissions.toml
git commit -m "land administration write grant"
"#,
        repo,
    );
}

fn principal(config: &but_authz::GovConfig, id: &str) -> anyhow::Result<Principal> {
    resolve_principal(
        |key| (key == "BUT_AGENT_HANDLE").then(|| OsString::from(id)),
        config,
    )
    .map_err(anyhow::Error::new)
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}

fn assert_denied(result: Result<(), Denial>) -> Denial {
    match result {
        Ok(()) => panic!("authorization must deny the missing target-ref permission"),
        Err(denial) => denial,
    }
}
