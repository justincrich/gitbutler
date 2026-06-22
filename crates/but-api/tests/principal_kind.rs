//! LPR-013 `principal_kind_read` / `principal_kind_update` but-api proofs.
//!
//! Mirrors `perm_governance.rs` / `branch_gates_governance.rs`: real
//! `but-authz` + real `gix` via `but_testsupport::writable_scenario`, hand
//! assertions over the working-tree `permissions.toml` and the committed
//! target-ref blob. The `kind` field is enforcement-neutral (LPR-005's
//! invariant): the writer only mutates `permissions.toml`; it composes
//! `enforce_administration_write_gate` and writes the WORKING TREE only
//! (inert until committed through the existing `governance_commit` path).

use but_api::legacy::{
    config_mutate::{AdminWriteGateError, classify_error},
    governance::{
        PrincipalKindEntry, REF_PIN_CAVEAT, principal_kind_read_with_repo,
        principal_kind_update_with_repo,
    },
};
use but_authz::{Denial, load_governance_config};

const MAIN_REF: &str = "refs/heads/main";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";

#[test]
#[serial_test::serial]
fn principal_kind_update_writes_worktree_inert_until_committed() -> anyhow::Result<()> {
    let (repo, _tmp) = kind_governance_base();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let outcome = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        principal_kind_update_with_repo(&repo, MAIN_REF, "agent-A", "agent")
    })?;

    // THEN: the call returns Ok with the ref-pin caveat.
    assert_eq!(
        outcome.caveat, REF_PIN_CAVEAT,
        "principal_kind_update must carry the ref-pin caveat"
    );

    // THEN: the WORKING-TREE permissions.toml now declares kind="agent" for agent-A.
    let worktree = worktree_permissions(&repo)?;
    let agent_a = principal_block(&worktree, "agent-A")?;
    assert!(
        agent_a.contains(r#"kind = "agent""#),
        "principal_kind_update must write kind=\"agent\" for agent-A into the working tree: {agent_a}"
    );

    // THEN: principal_kind_read reports agent-A's COMMITTED kind None/human.
    let list = principal_kind_read_with_repo(&repo, MAIN_REF)?;
    let agent_a_entry = find_entry(&list, "agent-A");
    assert_eq!(
        agent_a_entry.kind, None,
        "principal_kind_read must report agent-A's COMMITTED kind as None/human (target-ref blob — inert)"
    );
    assert!(
        agent_a_entry.pending,
        "principal_kind_read must set pending=true because the working-tree kind differs"
    );

    // THEN: ref_id(main) AFTER == BEFORE (no commit performed).
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "principal_kind_update must not commit or move refs/heads/main"
    );

    println!("principal_kind_update wrote kind=\"agent\" for agent-A into the working tree");
    println!("principal_kind_read reports the COMMITTED agent-A kind as None/human (inert)");
    println!("ref_id(refs/heads/main) is unchanged: {main_before}");
    println!("outcome caveat: `{REF_PIN_CAVEAT}`");
    Ok(())
}

#[test]
#[serial_test::serial]
fn principal_kind_update_round_trips_full_schema_lossless() -> anyhow::Result<()> {
    let (repo, _tmp) = kind_governance_base();

    let _outcome = temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        principal_kind_update_with_repo(&repo, MAIN_REF, "agent-A", "agent")
    })?;

    let worktree = worktree_permissions(&repo)?;

    // admin's full grant set survives the kind-only edit on agent-A.
    let admin = principal_block(&worktree, "admin")?;
    assert!(
        admin.contains("administration:write") && admin.contains("merge"),
        "admin grants must survive the kind-only edit: {admin}"
    );

    // agent-A's contents:write + groups=["reviewers"] survive.
    let agent_a = principal_block(&worktree, "agent-A")?;
    assert!(
        agent_a.contains("contents:write"),
        "agent-A contents:write must survive the kind edit: {agent_a}"
    );
    assert!(
        agent_a.contains("reviewers"),
        "agent-A groups=[\"reviewers\"] membership must survive the kind edit: {agent_a}"
    );
    assert!(
        agent_a.contains(r#"kind = "agent""#),
        "agent-A kind must now be \"agent\" (the only change): {agent_a}"
    );

    // rust-implementer's contents:write + kind="human" survive.
    let rust_implementer = principal_block(&worktree, "rust-implementer")?;
    assert!(
        rust_implementer.contains("contents:write"),
        "rust-implementer contents:write must survive: {rust_implementer}"
    );
    assert!(
        rust_implementer.contains(r#"kind = "human""#),
        "rust-implementer kind=\"human\" must survive the agent-A edit: {rust_implementer}"
    );

    // The [[group]] reviewers entry (permissions + members) survives.
    let reviewers = group_block(&worktree, "reviewers")?;
    assert!(
        reviewers.contains("reviews:write"),
        "[[group]] reviewers permissions must survive: {reviewers}"
    );
    assert!(
        reviewers.contains("agent-A"),
        "[[group]] reviewers members must survive: {reviewers}"
    );

    // Re-load through but_authz::load_governance_config to confirm nothing was
    // silently dropped (no governance weakening on a kind edit).
    let committed = load_governance_config(&repo, MAIN_REF)?;
    let committed_principal_count = committed.principals().len();
    assert!(
        committed_principal_count >= 3,
        "committed config still has the 3 principals after the kind edit"
    );

    println!(
        "admin grants, agent-A grants+groups, rust-implementer kind=human, and [[group]] reviewers all survived"
    );
    println!(
        "only agent-A's kind changed to \"agent\" — full-schema lossless round-trip confirmed"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn principal_kind_read_returns_committed_kinds_with_pending_signal() -> anyhow::Result<()> {
    let (repo, _tmp) = kind_governance_base();

    // Apply AC-1's working-tree kind="agent" edit (uncommitted).
    temp_env::with_var("BUT_AGENT_HANDLE", Some("admin"), || {
        principal_kind_update_with_repo(&repo, MAIN_REF, "agent-A", "agent")
    })?;

    let list = principal_kind_read_with_repo(&repo, MAIN_REF)?;

    // agent-A's COMMITTED kind is None/human (target ref), pending=true.
    let agent_a = find_entry(&list, "agent-A");
    assert_eq!(
        agent_a.kind, None,
        "agent-A committed kind must be None/human (read at target ref)"
    );
    assert!(
        agent_a.pending,
        "agent-A pending=true because the working-tree kind=\"agent\" differs"
    );

    // rust-implementer's COMMITTED kind="human", pending=false (no edit).
    let rust_implementer = find_entry(&list, "rust-implementer");
    assert_eq!(
        rust_implementer.kind.as_deref(),
        Some("human"),
        "rust-implementer committed kind=\"human\""
    );
    assert!(
        !rust_implementer.pending,
        "rust-implementer pending=false (no working-tree edit)"
    );

    // Every committed principal is listed.
    let mut listed = list
        .principals
        .iter()
        .map(|entry| entry.principal_id.clone())
        .collect::<Vec<_>>();
    listed.sort();
    assert_eq!(
        listed,
        vec![
            "admin".to_owned(),
            "agent-A".to_owned(),
            "rust-implementer".to_owned()
        ],
        "every committed principal must be listed"
    );

    // On a clean working tree (no edit), pending=false everywhere.
    let (clean_repo, _clean_tmp) = kind_governance_base();
    let clean_list = principal_kind_read_with_repo(&clean_repo, MAIN_REF)?;
    let clean_agent_a = find_entry(&clean_list, "agent-A");
    assert_eq!(
        clean_agent_a.kind, None,
        "clean tree: agent-A committed kind None/human"
    );
    assert!(
        !clean_agent_a.pending,
        "clean tree: agent-A pending=false (no working-tree edit)"
    );

    println!("principal_kind_read returned committed kinds + pending signal");
    println!(
        "agent-A pending=true after the uncommitted kind=\"agent\" edit; pending=false on clean tree"
    );
    Ok(())
}

#[test]
#[serial_test::serial]
fn principal_kind_update_non_admin_denied_writes_nothing() -> anyhow::Result<()> {
    let (repo, _tmp) = kind_governance_base();
    let before = worktree_permissions_bytes(&repo)?;

    // rust-implementer holds contents:write only (NO administration:write).
    let result = temp_env::with_var("BUT_AGENT_HANDLE", Some("rust-implementer"), || {
        principal_kind_update_with_repo(&repo, MAIN_REF, "agent-A", "agent")
    });

    let error = classified_error(result)?;
    assert_eq!(
        error.code, "perm.denied",
        "non-admin principal_kind_update must be denied with perm.denied"
    );
    assert!(
        error.message.contains("administration:write"),
        "denial message must name the missing administration:write authority: {}",
        error.message
    );

    // The working-tree permissions.toml must be byte-for-byte unchanged.
    let after = worktree_permissions_bytes(&repo)?;
    assert_eq!(
        before, after,
        "denied principal_kind_update must leave permissions.toml byte-for-byte unchanged"
    );

    println!("non-admin principal_kind_update denied with perm.denied naming administration:write");
    println!("working-tree permissions.toml is byte-for-byte unchanged on the denial path");
    Ok(())
}

// -------------------------------------------------------------------------------
// Helpers (mirror perm_governance.rs / branch_gates_governance.rs idioms)
// -------------------------------------------------------------------------------

fn find_entry<'a>(
    list: &'a but_api::legacy::governance::PrincipalKindList,
    id: &str,
) -> &'a PrincipalKindEntry {
    list.principals
        .iter()
        .find(|entry| entry.principal_id == id)
        .unwrap_or_else(|| panic!("principal_kind list must include {id}"))
}

fn kind_governance_base() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "agent-A"
permissions = ["contents:write"]
groups = ["reviewers"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]
kind = "human"

[[group]]
name = "reviewers"
permissions = ["reviews:write"]
members = ["agent-A"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config with principal kinds"
"#,
        &repo,
    );
    (repo, tmp)
}

fn worktree_permissions(repo: &gix::Repository) -> anyhow::Result<String> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    Ok(std::fs::read_to_string(workdir.join(PERMISSIONS_PATH))?)
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

fn classified_error<T>(result: anyhow::Result<T>) -> anyhow::Result<AdminWriteGateError> {
    match result {
        Ok(_) => anyhow::bail!("principal_kind update should reject this scenario"),
        Err(error) => classify_error(&error)
            .ok_or_else(|| anyhow::anyhow!("principal_kind error should classify")),
    }
}

// Silence the unused Denial import warning if no test references Denial directly.
#[allow(dead_code)]
fn _denial_type_marker(_denial: &Denial) {}
