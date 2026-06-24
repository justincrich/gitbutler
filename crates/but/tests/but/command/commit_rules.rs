//! LPR-REM-004 CLI integration proofs — `but commit` fires `Trigger::Commit`
//! rules through the production path.
//!
//! These are the only tests that prove the wiring: `crates/but-rules` has
//! `process_commit_rules` and 4 passing engine tests at
//! `crates/but-rules/tests/review_requested_hook.rs`, but without the wiring in
//! `crates/but/src/command/legacy/commit.rs` the function is never called from
//! `but commit`. These tests seed a `Trigger::Commit` rule into the project's
//! on-disk DB (same fixture shape as the engine tests), run the real `but
//! commit` binary, and assert on the persisted `local_review_assignments` row.
//!
//! Together they prove:
//! * **AC-1** — the rule engine actually fires in the production commit path.
//! * **AC-2** — rule evaluation is non-fatal: the commit lands even when the
//!   engine runs, and the row it writes never enters the commit/merge gate
//!   (proven statically by `cargo test -p but-authz invariant_build_gates`).
//! * **AC-3** — idempotency: the backend upsert keys on
//!   `(target, reviewer_principal)`, so multiple commits to the same branch
//!   with the same watched principal produce exactly one assignment row, not
//!   one row per commit.

use crate::utils::Sandbox;

const WATCHED_PRINCIPAL: &str = "dev";
const REVIEWER: &str = "rev2";
const BRANCH: &str = "A";
/// Full ref name — `target_branch.reference.to_string()` in `commit.rs`
/// produces `refs/heads/<name>` via `gix::refs::FullName`'s `Display` impl, so
/// the persisted assignment row's `target` column is the full ref.
const TARGET_REF: &str = "refs/heads/A";

/// AC-1 + AC-2 [PRIMARY]: a `Trigger::Commit` + `Action::RequestReview` rule,
/// seeded into the project DB, fires when `but commit` runs as the watched
/// principal. The commit MUST succeed (non-fatal — the engine is a post-commit
/// drive-metadata write), and exactly one `pending` `local_review_assignments`
/// row MUST be written for the configured reviewer.
#[test]
fn commit_trigger_rule_fires_review_assignment() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&[BRANCH])?;
    seed_commit_review_rule(&env)?;

    // Create a worktree change and commit it as the watched principal. The
    // commit MUST succeed — rule evaluation is post-commit and non-fatal.
    env.file("ordinary.txt", "ordinary");
    env.but("commit -m ordinary")
        .env("BUT_AGENT_HANDLE", WATCHED_PRINCIPAL)
        .assert()
        .success();

    // The engine outcome — a `pending` assignment row for the configured
    // reviewer on the commit's branch ref.
    let rows = local_review_assignments(&env, TARGET_REF)?;
    assert_eq!(
        rows.len(),
        1,
        "exactly one pending assignment must be written — the rule fired in the production commit path"
    );
    let assignment = &rows[0];
    assert_eq!(
        assignment.target, TARGET_REF,
        "assignment target must be the branch ref the commit landed on (CommitContext.target)"
    );
    assert_eq!(
        assignment.reviewer_principal, REVIEWER,
        "reviewer must come from the rule's Action::RequestReview payload"
    );
    assert_eq!(
        assignment.state,
        but_authz::AssignmentState::Pending.name(),
        "the auto-opened assignment is `pending` (drive-only, never a gate)"
    );

    Ok(())
}

/// AC-1 negative control: the SAME commit by an UNWATCHED principal opens NO
/// assignment. This rules out an "every-commit" hook — the rule's
/// `Filter::ClaudeCodeSessionId` must actually scope the firing.
#[test]
fn commit_trigger_rule_skipped_for_unwatched_principal() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&[BRANCH])?;
    seed_commit_review_rule(&env)?;

    env.file("ordinary.txt", "ordinary");
    // `other` is NOT the watched principal — the filter must not match.
    env.but("commit -m ordinary")
        .env("BUT_AGENT_HANDLE", "other")
        .assert()
        .success();

    let rows = local_review_assignments(&env, TARGET_REF)?;
    assert!(
        rows.is_empty(),
        "an unwatched-principal commit MUST NOT open an assignment — the rule is filter-scoped"
    );

    Ok(())
}

/// AC-3 idempotency: TWO commits to the same branch by the same watched
/// principal produce exactly ONE assignment row, not two. The backend's UNIQUE
/// index on `(target, reviewer_principal)` plus the
/// `ON CONFLICT … DO UPDATE` upsert means re-evaluation on the same
/// `(target, reviewer)` updates the existing row in place rather than
/// duplicating. This is what makes `--amend` and re-commit safe.
#[test]
fn commit_trigger_rule_idempotent_across_commits() -> anyhow::Result<()> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.setup_metadata(&[BRANCH])?;
    seed_commit_review_rule(&env)?;

    // First watched commit — opens the assignment.
    env.file("first.txt", "first");
    env.but("commit -m first")
        .env("BUT_AGENT_HANDLE", WATCHED_PRINCIPAL)
        .assert()
        .success();

    let after_first = local_review_assignments(&env, TARGET_REF)?;
    assert_eq!(
        after_first.len(),
        1,
        "the first watched commit opens exactly one assignment"
    );

    // Second watched commit on the SAME branch — the upsert must NOT duplicate
    // the row. `(target, reviewer_principal)` is constant across both commits
    // (same branch, same reviewer), so the second fire updates the existing
    // row in place.
    env.file("second.txt", "second");
    env.but("commit -m second")
        .env("BUT_AGENT_HANDLE", WATCHED_PRINCIPAL)
        .assert()
        .success();

    let after_second = local_review_assignments(&env, TARGET_REF)?;
    assert_eq!(
        after_second.len(),
        1,
        "the second watched commit on the same branch MUST NOT duplicate the assignment row — \
         the backend upsert is idempotent per (target, reviewer_principal), so re-running on \
         the same commit (e.g. via --amend) cannot duplicate artifacts either"
    );
    assert_eq!(
        after_second[0].reviewer_principal, REVIEWER,
        "the surviving row is still the configured reviewer's assignment"
    );

    Ok(())
}

// ---- fixture helpers --------------------------------------------------------

/// Seed a `Trigger::Commit` + `Action::RequestReview` rule scoped to the
/// watched principal into the project's on-disk DB. Mirrors the engine-test
/// fixture shape at `crates/but-rules/tests/review_requested_hook.rs` but
/// writes through the real project DB so the `but commit` subprocess can read
/// it back.
fn seed_commit_review_rule(env: &Sandbox) -> anyhow::Result<()> {
    let rule = but_rules::WorkspaceRule::for_test(
        "test-rule-lpr-rem-004",
        but_rules::Trigger::Commit,
        vec![but_rules::Filter::ClaudeCodeSessionId(
            WATCHED_PRINCIPAL.to_owned(),
        )],
        but_rules::Action::RequestReview(but_rules::RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );
    // Convert to the `but_db` row shape and persist. Scoped so the Context
    // (and its SQLite connection) is dropped before the `but commit`
    // subprocess opens the same DB file.
    {
        let ctx = env.context()?;
        let db_rule: but_db::WorkspaceRule = rule.try_into()?;
        ctx.db
            .get_cache_mut()?
            .workspace_rules_mut()
            .insert(db_rule)?;
    }
    Ok(())
}

/// Read the `local_review_assignments` rows for `target` from the project's
/// on-disk DB. Opens a fresh Context so we observe the post-commit state the
/// subprocess wrote.
fn local_review_assignments(
    env: &Sandbox,
    target: &str,
) -> anyhow::Result<Vec<but_db::LocalReviewAssignment>> {
    let ctx = env.context()?;
    let db = ctx.db.get_cache()?;
    Ok(db.local_review_assignments().list_by_target(target)?)
}
