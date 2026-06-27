use but_authz::{
    Authority, AuthoritySet, ConfigError, PrincipalId, governance_present, load_governance_config,
};

const TARGET_REF: &str = "refs/heads/main";

#[test]
fn config_loads_from_target_ref() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();

    let config = load_governance_config(&repo, TARGET_REF)?;
    let dev = config
        .principal_authorities(&PrincipalId::new("dev"))
        .ok_or_else(|| anyhow::anyhow!("dev principal must load from permissions.toml"))?;
    let ro = config
        .principal_authorities(&PrincipalId::new("ro"))
        .ok_or_else(|| anyhow::anyhow!("ro principal must load from permissions.toml"))?;
    let main = config
        .branch("main")
        .ok_or_else(|| anyhow::anyhow!("main branch protection must load from gates.toml"))?;

    assert!(
        dev.contains(Authority::ContentsWrite),
        "dev's committed permission set must include contents:write"
    );
    assert!(
        !ro.contains(Authority::ContentsWrite),
        "ro's committed permission set must not include contents:write"
    );
    assert!(
        main.protected(),
        "main branch must be protected by the target-ref gates.toml blob"
    );

    println!("dev effective set contains Authority::ContentsWrite");
    println!("ro effective set excludes Authority::ContentsWrite");
    println!("branch main protected == true");

    Ok(())
}

#[test]
fn agents_toml_parses_same_config() -> anyhow::Result<()> {
    let (repo, _tmp) = agents_governed_repo();

    let config = load_governance_config(&repo, TARGET_REF)?;
    let dev = config
        .principal_authorities(&PrincipalId::new("dev"))
        .ok_or_else(|| anyhow::anyhow!("dev agent must load from agents.toml"))?;
    let ro = config
        .principal_authorities(&PrincipalId::new("ro"))
        .ok_or_else(|| anyhow::anyhow!("ro agent must load from agents.toml"))?;
    let release_bot = config
        .principal_authorities(&PrincipalId::new("release-bot"))
        .ok_or_else(|| anyhow::anyhow!("release-bot agent must load from agents.toml"))?;
    let main = config
        .branch("main")
        .ok_or_else(|| anyhow::anyhow!("main branch protection must load from gates.toml"))?;
    let maintain = AuthoritySet::from_role("maintain")?;

    assert!(
        dev.contains(Authority::ContentsWrite),
        "dev's committed agents.toml grant must include contents:write"
    );
    assert!(
        !ro.contains(Authority::ContentsWrite),
        "ro's committed agents.toml grant must not include contents:write"
    );
    assert_eq!(
        release_bot, &maintain,
        "role=maintain in agents.toml must desugar like the legacy principal wire format"
    );
    assert!(
        release_bot.contains(Authority::Merge),
        "maintain role from agents.toml must include merge"
    );
    assert!(
        main.protected(),
        "main branch must be protected when agents.toml is paired with gates.toml"
    );

    println!("agents.toml dev effective set contains Authority::ContentsWrite");
    println!("agents.toml ro effective set excludes Authority::ContentsWrite");
    println!("agents.toml release-bot role=maintain contains Authority::Merge");
    println!("agents.toml branch main protected == true");

    Ok(())
}

#[test]
fn governance_present_agents_or_permissions() -> anyhow::Result<()> {
    let (agents_repo, _agents_tmp) = repo_with_agents_toml_only();
    let (permissions_repo, _permissions_tmp) = repo_with_permissions_toml_only();
    let (gates_repo, _gates_tmp) = repo_with_gates_toml_only();
    let (neither_repo, _neither_tmp) = but_testsupport::writable_scenario("governance-base");

    assert!(
        governance_present(&agents_repo, TARGET_REF)?,
        "committed .gitbutler/agents.toml alone must opt the ref into governance"
    );
    assert!(
        governance_present(&permissions_repo, TARGET_REF)?,
        "committed .gitbutler/permissions.toml alone must still opt the ref into governance"
    );
    assert!(
        governance_present(&gates_repo, TARGET_REF)?,
        "committed .gitbutler/gates.toml alone must continue to opt the ref into governance"
    );
    assert!(
        !governance_present(&neither_repo, TARGET_REF)?,
        "a resolvable ref with no governance files must remain ungoverned"
    );

    println!("governance_present agents.toml-only == true");
    println!("governance_present permissions.toml-only == true");
    println!("governance_present gates.toml-only == true");
    println!("governance_present neither == false");

    Ok(())
}

#[test]
fn both_files_prefers_agents_toml() -> anyhow::Result<()> {
    let (repo, _tmp) = repo_with_divergent_agents_and_permissions();

    let config = load_governance_config(&repo, TARGET_REF)?;
    let dev = config
        .principal_authorities(&PrincipalId::new("dev"))
        .ok_or_else(|| anyhow::anyhow!("dev agent must load from the preferred agents.toml"))?;

    assert!(
        dev.contains(Authority::ContentsWrite),
        "when both files exist, agents.toml must win over the divergent permissions.toml grant"
    );
    assert!(
        !dev.contains(Authority::ContentsRead),
        "when both files exist, permissions.toml must be ignored rather than merged"
    );

    println!("both files present: dev effective set contains Authority::ContentsWrite");

    Ok(())
}

#[test]
fn permissions_only_deprecation_warning() -> anyhow::Result<()> {
    if std::env::var_os("BUT_AUTHZ_DEPRECATION_WARNING_CHILD").is_some() {
        let (repo, _tmp) = governed_repo();

        let config = load_governance_config(&repo, TARGET_REF)?;
        let dev = config
            .principal_authorities(&PrincipalId::new("dev"))
            .ok_or_else(|| anyhow::anyhow!("dev principal must load from permissions.toml"))?;
        assert!(
            dev.contains(Authority::ContentsWrite),
            "legacy permissions.toml must still load while warning about migration"
        );

        return Ok(());
    }

    let output = std::process::Command::new(std::env::current_exe()?)
        .arg("--exact")
        .arg("permissions_only_deprecation_warning")
        .arg("--nocapture")
        .env("BUT_AUTHZ_DEPRECATION_WARNING_CHILD", "1")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "deprecation-warning child test must pass\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );

    let matching_lines = stderr
        .lines()
        .filter(|line| line.contains("permissions.toml") || line.contains("but agent migrate"))
        .collect::<Vec<_>>();
    assert_eq!(
        matching_lines.len(),
        1,
        "legacy-only load must emit exactly one deprecation warning line naming permissions.toml and but agent migrate; stderr was:\n{stderr}"
    );
    let line = matching_lines[0];
    assert!(
        line.contains("permissions.toml") && line.contains("but agent migrate"),
        "deprecation warning must name permissions.toml and the but agent migrate remediation"
    );

    println!("captured deprecation warning: {line}");
    println!("permissions.toml-only load emits exactly one but agent migrate warning");

    Ok(())
}

#[test]
fn byte_equivalent_across_formats() -> anyhow::Result<()> {
    let (permissions_repo, _permissions_tmp) = governed_repo();
    let (agents_repo, _agents_tmp) = agents_governed_repo();

    let permissions_config = load_governance_config(&permissions_repo, TARGET_REF)?;
    let agents_config = load_governance_config(&agents_repo, TARGET_REF)?;

    assert_eq!(
        permissions_config, agents_config,
        "agents.toml and permissions.toml parse to byte-equivalent GovConfig; only the table header differs"
    );

    println!("byte-equivalent agents.toml and permissions.toml produce equal GovConfig values");

    Ok(())
}

#[test]
fn config_ignores_working_tree_edit() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();

    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false
EOF
"#,
        &repo,
    );

    let config = load_governance_config(&repo, TARGET_REF)?;
    let main = config
        .branch("main")
        .ok_or_else(|| anyhow::anyhow!("main branch protection must load from gates.toml"))?;
    assert!(
        main.protected(),
        "uncommitted gates.toml edits must not affect target-ref config"
    );
    println!("after working-tree edit, branch main protected == true");

    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "ro"
permissions = ["contents:read", "contents:write"]
EOF
"#,
        &repo,
    );

    let config = load_governance_config(&repo, TARGET_REF)?;
    let ro = config
        .principal_authorities(&PrincipalId::new("ro"))
        .ok_or_else(|| anyhow::anyhow!("ro principal must load from permissions.toml"))?;
    assert!(
        !ro.contains(Authority::ContentsWrite),
        "uncommitted permissions.toml edits must not widen ro's target-ref set"
    );
    println!(
        "after the uncommitted permissions.toml edit, ro effective set excludes Authority::ContentsWrite"
    );

    Ok(())
}

#[test]
fn config_malformed_fails_closed() {
    let (repo, _tmp) = governed_repo();
    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]
name = "main"
protected = nope
EOF
git add .gitbutler/gates.toml
git commit -m "malformed gates"
"#,
        &repo,
    );

    assert_config_invalid(load_governance_config(&repo, TARGET_REF));
    println!("malformed gates.toml returns Err(ConfigError)");
    println!("error code == \"config.invalid\"");

    let (repo, _tmp) = governed_repo();
    but_testsupport::invoke_bash(
        r#"
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:bogus"]
EOF
git add .gitbutler/permissions.toml
git commit -m "malformed permissions"
"#,
        &repo,
    );

    assert_config_invalid(load_governance_config(&repo, TARGET_REF));
    println!("malformed permissions.toml returns Err(ConfigError)");
    println!("error code == \"config.invalid\"");
}

#[test]
fn config_role_entry_desugars() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();

    let config = load_governance_config(&repo, TARGET_REF)?;
    let release_bot = config
        .principal_authorities(&PrincipalId::new("release-bot"))
        .ok_or_else(|| anyhow::anyhow!("release-bot principal must load from permissions.toml"))?;
    let maintain = AuthoritySet::from_role("maintain")?;

    assert_eq!(
        release_bot, &maintain,
        "role=maintain must desugar to the same AuthoritySet as from_role"
    );
    assert!(
        release_bot.contains(Authority::Merge),
        "maintain role must include merge"
    );
    assert!(
        release_bot.contains(Authority::AdministrationRead),
        "maintain role must include administration:read"
    );
    assert!(
        !release_bot.contains(Authority::AdministrationWrite),
        "maintain role must not include administration:write"
    );

    println!("release-bot set contains Authority::Merge");
    println!("release-bot set contains Authority::AdministrationRead");
    println!("release-bot set excludes Authority::AdministrationWrite");

    Ok(())
}

#[test]
fn config_loads_from_target_not_head() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    but_testsupport::invoke_bash(
        r#"
git checkout -b feat
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = false
EOF
git add .gitbutler/gates.toml
git commit -m "unprotect on feature"
"#,
        &repo,
    );

    let config = load_governance_config(&repo, TARGET_REF)?;
    let main = config
        .branch("main")
        .ok_or_else(|| anyhow::anyhow!("main branch protection must load from gates.toml"))?;
    assert!(
        main.protected(),
        "target ref must govern even when HEAD points at a feature branch"
    );

    println!("branch main protected == true (read from refs/heads/main, not the feature head)");

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
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]

[[principal]]
id = "release-bot"
role = "maintain"
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

#[test]
fn gates_wire_accepts_full_gate_array() -> anyhow::Result<()> {
    let (repo, _tmp) = repo_with_full_gates();

    // negative_control: #[serde(deny_unknown_fields)] must NOT reject the [[gate]] table.
    // If it did, this would return Err(ConfigError) and the ? would propagate the failure.
    let config = load_governance_config(&repo, TARGET_REF)?;

    // negative_control: the loader must NOT silently drop branch protections when [[gate]] is present.
    let main = config
        .branch("main")
        .ok_or_else(|| anyhow::anyhow!("main branch protection must survive [[gate]] parsing"))?;
    assert!(
        main.protected(),
        "branch protection for main must remain protected=true alongside a [[gate]] array"
    );

    // negative_control: the config must NOT be empty — the seeded branch protection is present,
    // ruling out a stub loader that returns an empty GovConfig.
    assert!(
        !config.branches().is_empty(),
        "GovConfig must carry the seeded branch protection, not an empty map"
    );

    println!("gates.toml with [[branch]] + [[gate]] parsed without config.invalid");
    println!("branch main protected == true (not dropped by [[gate]] table)");
    println!("GovConfig branches non-empty (not a stub empty config)");

    Ok(())
}

#[test]
fn gates_path_returns_canonical() {
    // negative_control: gates_path must return exactly this literal — any other path fails.
    assert_eq!(
        but_authz::gates_path(),
        ".gitbutler/gates.toml",
        "gates_path must return the canonical governance gates path literal"
    );

    // negative_control: must not be an empty stub.
    assert!(
        !but_authz::gates_path().is_empty(),
        "gates_path must not return an empty string"
    );

    // negative_control: calling but_authz::gates_path() at all proves it is re-exported
    // from the crate root — if it were not, this test would not compile.
    println!("but_authz::gates_path() == \".gitbutler/gates.toml\"");
    println!("gates_path is re-exported and callable as but_authz::gates_path()");
}

fn repo_with_full_gates() -> (gix::Repository, impl std::fmt::Debug) {
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
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 2
require_distinct_from_author = true
require_approval_from_group = ["code-reviewers"]
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "seed gates with full [[gate]] array"
"#,
        &repo,
    );
    (repo, tmp)
}

fn agents_governed_repo() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/agents.toml <<'EOF'
[[agent]]
id = "dev"
permissions = ["contents:write"]

[[agent]]
id = "ro"
permissions = ["contents:read"]

[[agent]]
id = "release-bot"
role = "maintain"
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/agents.toml .gitbutler/gates.toml
git commit -m "agents governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn repo_with_agents_toml_only() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/agents.toml <<'EOF'
[[agent]]
id = "dev"
permissions = ["contents:write"]
EOF

git add .gitbutler/agents.toml
git commit -m "agents governance opt in"
"#,
        &repo,
    );
    (repo, tmp)
}

fn repo_with_permissions_toml_only() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]
EOF

git add .gitbutler/permissions.toml
git commit -m "permissions governance opt in"
"#,
        &repo,
    );
    (repo, tmp)
}

fn repo_with_gates_toml_only() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/gates.toml
git commit -m "gates governance opt in"
"#,
        &repo,
    );
    (repo, tmp)
}

fn repo_with_divergent_agents_and_permissions() -> (gix::Repository, impl std::fmt::Debug) {
    let (repo, tmp) = but_testsupport::writable_scenario("governance-base");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/agents.toml <<'EOF'
[[agent]]
id = "dev"
permissions = ["contents:write"]
EOF

cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/agents.toml .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "divergent agents and permissions governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn assert_config_invalid(result: Result<but_authz::GovConfig, ConfigError>) {
    let Err(error) = result else {
        panic!("malformed target-ref config must fail closed");
    };
    assert_eq!(
        error.code(),
        "config.invalid",
        "malformed config must have the stable config.invalid classification"
    );
}
