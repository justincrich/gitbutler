use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{
        BranchGatesOutcome, BranchProtectionInput, branch_gates_read_with_repo,
        branch_gates_update_with_repo,
    },
};
use but_authz::Denial;

const MAIN_REF: &str = "refs/heads/main";
const GATES_PATH: &str = ".gitbutler/gates.toml";

#[test]
#[serial_test::serial]
fn branch_gates_read_returns_working_tree_gates_under_admin_read() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_base();

    let outcome = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin-reader")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || branch_gates_read_with_repo(&repo, MAIN_REF),
    )?;

    assert_gates_entry(&outcome, "main", true);
    assert_gates_entry(&outcome, "release", false);

    println!(
        "branch_gates_read returned {} entries under administration:read",
        outcome.branches.len()
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_read_non_admin_denied() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_base();
    let before = worktree_gates_bytes(&repo)?;

    let result = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || branch_gates_read_with_repo(&repo, MAIN_REF),
    );

    let denial = structured_denial(result, "branch_gates_read non-admin")?;
    assert_eq!(
        denial.code,
        Denial::PERM_DENIED_CODE,
        "non-admin branch_gates_read must return perm.denied"
    );
    assert!(
        denial.message.contains("administration:read"),
        "branch_gates_read denial must name the missing administration:read authority"
    );
    assert_eq!(
        worktree_gates_bytes(&repo)?,
        before,
        "denied branch_gates_read must leave gates.toml byte-for-byte unchanged"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_upserts_under_admin_write() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_base();

    let updated = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            branch_gates_update_with_repo(
                &repo,
                MAIN_REF,
                "main",
                BranchProtectionInput { protected: false },
            )
        },
    )?;
    assert_gates_entry(&updated, "main", false);
    assert_gates_entry(&updated, "release", false);
    let after_update = worktree_gates(&repo)?;
    assert!(
        after_update.contains(r#"name = "main""#) && after_update.contains("protected = false"),
        "branch_gates_update must write protected=false for main to gates.toml"
    );

    let inserted = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("admin")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            branch_gates_update_with_repo(
                &repo,
                MAIN_REF,
                "featureBranch",
                BranchProtectionInput { protected: true },
            )
        },
    )?;
    assert_gates_entry(&inserted, "featureBranch", true);
    let after_insert = worktree_gates(&repo)?;
    assert!(
        after_insert.contains(r#"name = "featureBranch""#)
            && after_insert.contains("protected = true"),
        "branch_gates_update must insert a new gates.toml entry for featureBranch"
    );

    println!("branch_gates_update upserted existing and inserted new entries");
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_non_admin_denied_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_base();
    let before = worktree_gates_bytes(&repo)?;

    let result = temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("rust-implementer")),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
        ],
        || {
            branch_gates_update_with_repo(
                &repo,
                MAIN_REF,
                "main",
                BranchProtectionInput { protected: false },
            )
        },
    );

    let error = classified_error(result)?;
    assert_eq!(
        error.code, "perm.denied",
        "non-admin branch_gates_update must be denied with perm.denied"
    );
    assert!(
        error.message.contains("administration:write"),
        "denied branch_gates_update must name the missing administration:write authority"
    );
    assert_eq!(
        worktree_gates_bytes(&repo)?,
        before,
        "denied branch_gates_update must leave gates.toml byte-for-byte unchanged"
    );
    Ok(())
}

fn assert_gates_entry(outcome: &BranchGatesOutcome, name: &str, protected: bool) {
    let entry = outcome
        .branches
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("branch_gates outcome must include {name}"));
    assert_eq!(
        entry.protected, protected,
        "branch_gates entry {name} must have protected={protected}"
    );
}

fn branch_gates_base() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
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
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "release"
protected = false
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config with branch gates"
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
        Ok(_) => anyhow::bail!("branch_gates update should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("branch_gates error should classify")),
    }
}

fn worktree_gates(repo: &gix::Repository) -> anyhow::Result<String> {
    Ok(String::from_utf8(worktree_gates_bytes(repo)?)?)
}

fn worktree_gates_bytes(repo: &gix::Repository) -> anyhow::Result<Vec<u8>> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(std::fs::read(workdir.join(GATES_PATH))?)
}
