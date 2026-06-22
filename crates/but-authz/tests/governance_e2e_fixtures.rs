use std::{ffi::OsString, path::Path};

use but_authz::{
    Authority, Denial, GroupName, Principal, PrincipalId, authorize, load_governance_config,
    resolve_principal,
};

const TARGET_REF: &str = "refs/heads/master";
const ADMIN_HANDLE: &str = "admin";
const NONADMIN_HANDLE: &str = "dev";
const SEED_SCRIPT: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../e2e/playwright/scripts/seed-governance.sh"
);

#[test]
fn governance_e2e_seed_commits_target_ref_and_clean_tree() -> anyhow::Result<()> {
    let (repo, _tmp) = seeded_governance_repo()?;

    let permissions = git_stdout(
        repo.workdir().expect("seeded repo is non-bare"),
        ["cat-file", "-p", "master:.gitbutler/permissions.toml"],
    )?;
    assert!(
        permissions.contains(r#"id = "admin""#),
        "committed permissions.toml must include the admin principal"
    );
    assert!(
        permissions.contains(r#"permissions = ["administration:write", "merge"]"#),
        "admin's committed principal entry must carry administration:write and merge"
    );
    assert!(
        permissions.contains(r#"id = "dev""#),
        "committed permissions.toml must include the non-admin principal"
    );
    assert!(
        permissions.contains(r#"permissions = ["contents:write"]"#),
        "dev's committed principal entry must carry contents:write only"
    );
    assert!(
        permissions.contains(r#"name = "maintainers""#)
            && permissions.contains(r#"members = ["admin"]"#),
        "committed permissions.toml must include a maintainers group with a member"
    );
    assert!(
        permissions.contains(r#"name = "code-reviewers""#)
            && permissions.contains(r#"members = ["dev"]"#),
        "committed permissions.toml must include a code-reviewers group with a member"
    );

    let gates = git_stdout(
        repo.workdir().expect("seeded repo is non-bare"),
        ["cat-file", "-p", "master:.gitbutler/gates.toml"],
    )?;
    assert_eq!(
        gates, "[[branch]]\nname = \"master\"\nprotected = true\n",
        "gates.toml must use the locked minimal live schema only"
    );

    let status = git_stdout(
        repo.workdir().expect("seeded repo is non-bare"),
        ["status", "--porcelain"],
    )?;
    assert!(
        status.is_empty(),
        "seed script must leave no uncommitted .gitbutler config behind"
    );

    println!("master:.gitbutler/permissions.toml contains admin and dev principals");
    println!("master:.gitbutler/gates.toml contains only [[branch]] master protected=true");
    println!("git status --porcelain is empty after seeding");

    Ok(())
}

#[test]
fn governance_e2e_admin_authorizes() -> anyhow::Result<()> {
    let (repo, _tmp) = seeded_governance_repo()?;
    let config = load_governance_config(&repo, TARGET_REF)?;
    let admin = principal_from_handle(&config, ADMIN_HANDLE)?;

    assert_eq!(
        admin.id(),
        &PrincipalId::new(ADMIN_HANDLE),
        "injected BUT_AGENT_HANDLE lookup must resolve the seeded admin principal"
    );
    assert!(
        admin.groups().contains(&GroupName::new("maintainers")),
        "admin must resolve with its committed maintainers group membership"
    );
    authorize(&admin, Authority::AdministrationWrite, &config)?;

    println!("resolve_principal(injected BUT_AGENT_HANDLE=admin) returned the admin principal");
    println!("authorize(admin, AdministrationWrite, cfg) returned Ok(())");

    Ok(())
}

#[test]
fn governance_e2e_dev_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = seeded_governance_repo()?;
    let config = load_governance_config(&repo, TARGET_REF)?;
    let dev = principal_from_handle(&config, NONADMIN_HANDLE)?;

    assert_eq!(
        dev.id(),
        &PrincipalId::new(NONADMIN_HANDLE),
        "injected BUT_AGENT_HANDLE lookup must resolve the seeded non-admin principal"
    );
    assert!(
        dev.groups().contains(&GroupName::new("code-reviewers")),
        "dev must resolve with its committed code-reviewers group membership"
    );

    let denial = assert_denied(authorize(&dev, Authority::AdministrationWrite, &config));
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "missing administration:write must use the stable perm.denied code"
    );
    assert!(
        denial.message.contains("administration:write"),
        "missing-permission denial must name administration:write"
    );

    println!("resolve_principal(injected BUT_AGENT_HANDLE=dev) returned the dev principal");
    println!("authorize(dev, AdministrationWrite, cfg) returned code == \"perm.denied\"");
    println!("denial message contains \"administration:write\"");

    Ok(())
}

fn seeded_governance_repo() -> anyhow::Result<(gix::Repository, impl std::fmt::Debug)> {
    let (_fixture_repo, tmp) = but_testsupport::writable_scenario("governance-base");
    let seed_dir = tmp.path().join("seed-governance-workdir");
    let output = std::process::Command::new("bash")
        .arg(SEED_SCRIPT)
        .arg(&seed_dir)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "seed-governance.sh failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo = but_testsupport::open_repo(&seed_dir)?;
    Ok((repo, tmp))
}

fn principal_from_handle(config: &but_authz::GovConfig, handle: &str) -> anyhow::Result<Principal> {
    resolve_principal(
        |key| (key == "BUT_AGENT_HANDLE").then(|| OsString::from(handle)),
        config,
    )
    .map_err(anyhow::Error::new)
}

fn git_stdout<const N: usize>(dir: &Path, args: [&str; N]) -> anyhow::Result<String> {
    let output = but_testsupport::git_at_dir(dir).args(args).output()?;
    anyhow::ensure!(
        output.status.success(),
        "git command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(String::from_utf8(output.stdout)?)
}

fn assert_denied(result: Result<(), Denial>) -> Denial {
    match result {
        Ok(()) => panic!("authorization must deny the missing administration:write permission"),
        Err(denial) => denial,
    }
}
