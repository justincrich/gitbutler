//! STEER-003 — gate-state-aware `authorized_actions` proof against real git.
//!
//! This integration test proves the C5 subtraction (AC-2) against a real git
//! repository: a `branch.protected` denial where the caller HOLDS
//! `contents:write` but the target ref (main) is protected. The menu must
//! offer a feature-branch commit affordance AND must NOT offer the
//! protected-ref commit that just failed.
//!
//! The test loads the same `GovConfig` the commit gate loads, constructs the
//! denied-route context, and calls `but_authz::authorized_actions` directly
//! (STEER-004 wires the menu into the denial carriers; STEER-003 provides the
//! derivation function).
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-003-gate-state-aware-authorized-actions.md` for the task contract.

use but_authz::{
    DenialPredicate, DeniedRoute, Principal, PrincipalId, Route, authorized_actions,
    load_governance_config,
};

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";

/// AC-2 / TC-2, TC-3 — a `branch.protected` menu contains a feature-branch
/// commit affordance on a different unprotected ref, and EXCLUDES the
/// protected-ref commit just denied.
///
/// GIVEN a `branch.protected` denial where the caller HOLDS `contents:write`
/// but the target ref (main) is protected, WHEN `authorized_actions` is
/// derived with the failed (commit route, branch-protected predicate, main
/// ref) subtracted, THEN the menu offers a commit to an unprotected FEATURE
/// branch + review affordances, and EXCLUDES the protected-ref commit.
#[test]
#[serial_test::serial]
fn steer_branch_protected_menu_feature_not_protected() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();

    // Load the SAME GovConfig the commit gate loads at refs/heads/main.
    let cfg = load_governance_config(&repo, MAIN_REF)?;

    // Resolve the dev principal (holds contents:write per the fixture).
    let dev_id = PrincipalId::new("dev");
    let dev_authorities = cfg
        .principal_authorities(&dev_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("dev principal must be in the governed config"))?;
    let principal = Principal::new(dev_id, dev_authorities, []);

    // Prove the fixture: dev holds contents:write.
    assert!(
        principal
            .authorities()
            .contains(but_authz::Authority::ContentsWrite),
        "fixture: dev must hold contents:write"
    );

    // Prove the fixture: main is protected in the committed config.
    assert!(
        cfg.branch("main").is_some_and(|b| b.protected()),
        "fixture: main must be protected in the committed config"
    );

    // Construct the denied-route context: Commit route, BranchProtected
    // predicate (the C5 gate-state-aware case).
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);

    // Derive the menu using the cfg the gate already loaded.
    let actions = authorized_actions(&principal, &denied, &cfg);

    assert!(
        !actions.is_empty(),
        "branch.protected menu must be non-empty (at minimum discovery + feature commit)"
    );

    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();

    // Must observe: a commit affordance on an unprotected FEATURE ref.
    assert!(
        commands.contains(&"but commit"),
        "branch.protected menu MUST include `but commit` (the feature-branch affordance — \
         contents:write is held; the C5 subtraction is about the REF not the authority)"
    );

    // The `but commit` entry's effect MUST name an unprotected feature branch,
    // NOT the protected ref (the C5 subtraction — succeeding context).
    let commit_action = actions
        .iter()
        .find(|a| a.command == "but commit")
        .expect("`but commit` entry must be present");
    assert!(
        commit_action.effect.to_lowercase().contains("unprotected"),
        "`but commit` effect must name an UNPROTECTED feature branch (the C5 succeeding context), \
         got: {:?}",
        commit_action.effect
    );

    // Must NOT observe: a commit-to-protected-`main` affordance. The catalog
    // entry's effect must NOT name the protected ref.
    assert!(
        !commit_action
            .effect
            .to_lowercase()
            .contains("protected ref"),
        "`but commit` effect must NOT name the protected ref (the denied context): {:?}",
        commit_action.effect
    );

    // Discovery is appended.
    assert!(
        commands.contains(&"but perm list"),
        "branch.protected menu must include the discovery affordance"
    );

    println!("AC-2: branch.protected menu for dev (contents:write) on main:");
    for action in &actions {
        println!("  - {} → {}", action.command, action.effect);
    }

    Ok(())
}

/// AC-2 edge case — the feature branch `feat` is NOT protected in the
/// fixture, confirming the C5 succeeding context (commit to `feat` would
/// succeed).
#[test]
#[serial_test::serial]
fn steer_branch_protected_menu_feature_branch_is_unprotected() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let cfg = load_governance_config(&repo, FEAT_REF)?;

    // The feature branch must NOT be protected in the committed config —
    // this is the C5 succeeding context the menu's `but commit` affordance
    // points to.
    assert!(
        cfg.branch("feat").is_some_and(|b| !b.protected()),
        "fixture: feat must NOT be protected — it's the C5 succeeding-context ref"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Fixture — mirrors crates/but-api/tests/commit_gate.rs::governed_repo
// ---------------------------------------------------------------------------

fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
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
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "feat"
protected = false
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
echo feat-base >feat-base.txt
git add feat-base.txt
git commit -m "feat base"
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}
