use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{BranchGatesOutcome, BranchProtectionInput, REF_PIN_CAVEAT},
    governance::{branch_gates_read_with_repo, branch_gates_update_with_repo},
};
use serde::Deserialize;

const MAIN_REF: &str = "refs/heads/main";
const GATES_PATH: &str = ".gitbutler/gates.toml";

#[test]
#[serial_test::serial]
fn branch_gates_update_writes_worktree_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();
    let before_ref = ref_id(&repo)?;

    let update = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "main",
            BranchProtectionInput {
                protected: true,
                min_approvals: Some(3),
                require_distinct_from_author: Some(true),
                require_approval_from_group: Some(vec![
                    "code-reviewers".to_owned(),
                    "maintainers".to_owned(),
                ]),
            },
        )
    })?;

    assert!(
        update.caveat.contains(REF_PIN_CAVEAT),
        "branch_gates_update must return the inert-until-committed caveat"
    );
    assert_eq!(
        ref_id(&repo)?,
        before_ref,
        "branch_gates_update must not commit or move refs/heads/main"
    );
    assert_eq!(
        gate(&worktree_gates(&repo)?, "main")?.min_approvals,
        3,
        "admin update must write the edited min_approvals to the working tree"
    );

    let read = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_read_with_repo(&repo, MAIN_REF)
    })?;
    let main = branch(&read, "main");
    assert_eq!(
        main.min_approvals, 2,
        "branch_gates_read must report committed target-ref gates.toml, not the uncommitted edit"
    );
    assert!(
        main.pending,
        "branch_gates_read must flag a pending working-tree diff for main"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_unprotect_preserves_gate_requirement() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "main",
            BranchProtectionInput {
                protected: false,
                min_approvals: None,
                require_distinct_from_author: None,
                require_approval_from_group: None,
            },
        )
    })?;

    let gates = worktree_gates(&repo)?;
    assert!(
        !branch_wire(&gates, "main")?.protected,
        "unprotecting main must land protected=false in the working tree"
    );
    assert_full_main_requirement(&gates)?;
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_round_trips_full_gate_schema_lossless() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "main",
            BranchProtectionInput {
                protected: true,
                min_approvals: None,
                require_distinct_from_author: None,
                require_approval_from_group: None,
            },
        )
    })?;

    let gates = worktree_gates(&repo)?;
    assert_eq!(
        gates.branch.len(),
        2,
        "protection-only writes must preserve every unrelated [[branch]] entry"
    );
    assert_eq!(
        gates.gate.len(),
        2,
        "protection-only writes must preserve every [[gate]] review requirement"
    );
    assert_full_main_requirement(&gates)?;
    let release = gate(&gates, "release")?;
    assert_eq!(
        release.min_approvals, 1,
        "release min_approvals must survive an unrelated main edit"
    );
    assert_eq!(
        release.require_approval_from_group,
        vec!["maintainers"],
        "release required groups must survive an unrelated main edit"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_non_admin_denied_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();
    let before = worktree_gates_bytes(&repo)?;

    let error = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        classified_error(branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "main",
            BranchProtectionInput {
                protected: false,
                min_approvals: Some(0),
                require_distinct_from_author: Some(false),
                require_approval_from_group: Some(Vec::new()),
            },
        ))
    })?;

    assert_eq!(
        error.code, "perm.denied",
        "non-admin branch_gates_update must return perm.denied"
    );
    assert!(
        error.message.contains("administration:write"),
        "denied branch_gates_update must name administration:write"
    );
    assert_eq!(
        worktree_gates_bytes(&repo)?,
        before,
        "denied branch_gates_update must not alter gates.toml"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_read_returns_committed_set_with_pending_signal() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    let clean = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_read_with_repo(&repo, MAIN_REF)
    })?;
    let clean_main = branch(&clean, "main");
    assert_eq!(
        clean_main.min_approvals, 2,
        "clean read must include the committed main review requirement"
    );
    assert!(
        !clean_main.pending,
        "clean working tree must not report a pending gates.toml edit"
    );

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "main",
            BranchProtectionInput {
                protected: true,
                min_approvals: Some(3),
                require_distinct_from_author: None,
                require_approval_from_group: None,
            },
        )
    })?;

    let pending = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_read_with_repo(&repo, MAIN_REF)
    })?;
    let pending_main = branch(&pending, "main");
    assert_eq!(
        pending_main.min_approvals, 2,
        "pending read must still expose the committed target-ref gate requirement"
    );
    assert!(
        pending_main.pending,
        "uncommitted working-tree gates.toml changes must set pending=true"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_sets_distinct_and_required_groups() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "release",
            BranchProtectionInput {
                protected: true,
                min_approvals: Some(2),
                require_distinct_from_author: Some(true),
                require_approval_from_group: Some(vec![
                    "maintainers".to_owned(),
                    "code-reviewers".to_owned(),
                ]),
            },
        )
    })?;

    let gates = worktree_gates(&repo)?;
    let release = gate(&gates, "release")?;
    assert!(
        release.require_distinct_from_author,
        "branch_gates_update must set require_distinct_from_author=true"
    );
    assert_eq!(
        release.require_approval_from_group,
        vec!["maintainers", "code-reviewers"],
        "branch_gates_update must set the complete required group list"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_update_appends_new_branch_and_creates_absent_file() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &repo,
            MAIN_REF,
            "feature/x",
            BranchProtectionInput {
                protected: true,
                min_approvals: Some(1),
                require_distinct_from_author: Some(false),
                require_approval_from_group: Some(vec!["maintainers".to_owned()]),
            },
        )
    })?;

    let populated = worktree_gates(&repo)?;
    assert!(
        branch_wire(&populated, "feature/x")?.protected,
        "new branch updates must append a [[branch]] entry"
    );
    assert_eq!(
        gate(&populated, "feature/x")?.require_approval_from_group,
        vec!["maintainers"],
        "new branch updates must append the corresponding [[gate]] entry"
    );
    assert_full_main_requirement(&populated)?;

    let (empty_repo, _empty_tmp) = branch_gates_without_gates_file();
    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_update_with_repo(
            &empty_repo,
            MAIN_REF,
            "feature/x",
            BranchProtectionInput {
                protected: true,
                min_approvals: Some(1),
                require_distinct_from_author: Some(false),
                require_approval_from_group: Some(vec!["maintainers".to_owned()]),
            },
        )
    })?;

    let created = worktree_gates(&empty_repo)?;
    assert_eq!(
        created.branch.len(),
        1,
        "absent gates.toml create path must seed exactly one [[branch]] entry"
    );
    assert_eq!(
        created.gate.len(),
        1,
        "absent gates.toml create path must seed exactly one [[gate]] entry"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn branch_gates_read_requires_administration_read() -> anyhow::Result<()> {
    let (repo, _tmp) = branch_gates_contract_base();

    let admin = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        branch_gates_read_with_repo(&repo, MAIN_REF)
    })?;
    assert_eq!(
        branch(&admin, "main").min_approvals,
        2,
        "admin branch_gates_read must disclose the committed main gate"
    );

    let denial = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        match branch_gates_read_with_repo(&repo, MAIN_REF) {
            Ok(_) => anyhow::bail!("branch_gates_read should reject callers without admin read"),
            Err(error) => error.downcast::<but_authz::Denial>().map_err(|error| {
                anyhow::anyhow!("branch_gates_read should return Denial: {error}")
            }),
        }
    })?;
    assert_eq!(
        denial.code,
        but_authz::Denial::PERM_DENIED_CODE,
        "branch_gates_read without administration:read must return perm.denied"
    );
    assert!(
        denial.message.contains("administration:read"),
        "branch_gates_read denial must name administration:read"
    );
    Ok(())
}

fn branch<'a>(
    outcome: &'a BranchGatesOutcome,
    name: &str,
) -> &'a but_api::legacy::governance::BranchGateEntry {
    outcome
        .branches
        .iter()
        .find(|branch| branch.name == name)
        .unwrap_or_else(|| panic!("branch_gates_read must include {name}"))
}

fn assert_full_main_requirement(gates: &GatesWire) -> anyhow::Result<()> {
    let main = gate(gates, "main")?;
    assert_eq!(
        main.min_approvals, 2,
        "main min_approvals must survive branch gate edits"
    );
    assert!(
        main.require_distinct_from_author,
        "main distinct-from-author requirement must survive branch gate edits"
    );
    assert_eq!(
        main.require_approval_from_group,
        vec!["code-reviewers", "maintainers"],
        "main required approval groups must survive branch gate edits"
    );
    Ok(())
}

fn branch_gates_contract_base() -> (gix::Repository, tempfile::TempDir) {
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

[[group]]
name = "code-reviewers"
permissions = ["reviews:write"]
members = ["admin"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["admin"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "release"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 2
require_distinct_from_author = true
require_approval_from_group = ["code-reviewers", "maintainers"]

[[gate]]
branch = "release"
type = "review"
min_approvals = 1
require_approval_from_group = ["maintainers"]
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config with full branch gates"
"#,
        &repo,
    );
    (repo, tmp)
}

fn branch_gates_without_gates_file() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]
EOF

git add .gitbutler/permissions.toml
git commit -m "governance config without gates file"
"#,
        &repo,
    );
    (repo, tmp)
}

fn classified_error<T>(result: anyhow::Result<T>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(_) => anyhow::bail!("branch_gates_update should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("branch_gates_update error should classify")),
    }
}

fn worktree_gates(repo: &gix::Repository) -> anyhow::Result<GatesWire> {
    let text = String::from_utf8(worktree_gates_bytes(repo)?)?;
    Ok(toml::from_str(&text)?)
}

fn worktree_gates_bytes(repo: &gix::Repository) -> anyhow::Result<Vec<u8>> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(std::fs::read(workdir.join(GATES_PATH))?)
}

fn branch_wire<'a>(gates: &'a GatesWire, name: &str) -> anyhow::Result<&'a BranchWire> {
    gates
        .branch
        .iter()
        .find(|branch| branch.name == name)
        .ok_or_else(|| anyhow::anyhow!("gates.toml must include [[branch]] {name}"))
}

fn gate<'a>(gates: &'a GatesWire, branch: &str) -> anyhow::Result<&'a GateWire> {
    gates
        .gate
        .iter()
        .find(|gate| gate.branch == branch)
        .ok_or_else(|| anyhow::anyhow!("gates.toml must include [[gate]] for {branch}"))
}

fn ref_id(repo: &gix::Repository) -> anyhow::Result<gix::ObjectId> {
    let mut reference = repo.find_reference(MAIN_REF)?;
    Ok(reference.peel_to_id()?.detach())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesWire {
    #[serde(default)]
    branch: Vec<BranchWire>,
    #[serde(default)]
    gate: Vec<GateWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BranchWire {
    name: String,
    protected: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GateWire {
    branch: String,
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    min_approvals: usize,
    #[serde(default)]
    require_approval_from_group: Vec<String>,
    #[serde(default)]
    require_distinct_from_author: bool,
}
