//! LPR-007 integration proofs (AC-1..AC-4) — real but-db.
//!
//! These tests exercise the additive `Trigger::Commit` + `Action::RequestReview`
//! extension to the shipped but-rules engine. A fixtured commit matching a
//! configured rule's filter fires the "review-requested" action, and the test
//! asserts the ENGINE OUTCOME — the `pending` `local_review_assignments` row
//! written via the LPR-001 Handle — never a hook log message.
//!
//! The auto-opened assignment is drive-only — it blocks no commit and gates no
//! merge (AC-4). The static side of that guarantee (the commit/merge gates
//! never read `local_review_assignments`) is proven by
//! `cargo test -p but-authz invariant_build_gates`; this file proves the
//! dynamic side — the hook writes the row but never returns a denial, never
//! touches `local_review_verdicts` (the merge gate's data source), and never
//! holds a repo guard.

use but_rules::{
    Action, CommitContext, Filter, RequestReviewAction, Trigger, WorkspaceRule,
    process_commit_rules,
};

const WATCHED_BRANCH: &str = "refs/heads/feat";
const UNWATCHED_BRANCH: &str = "refs/heads/docs";
const REVIEWER: &str = "rev2";

/// AC-1 [PRIMARY]: a fixtured commit matching the rule's filter fires the
/// "review-requested" action and writes a `pending` `local_review_assignments`
/// row for the configured reviewer (the ENGINE OUTCOME — DB row, not a log).
#[test]
fn review_requested_hook_creates_pending_assignment() -> anyhow::Result<()> {
    let mut db = but_db::DbHandle::new_at_path(":memory:")?;

    // Configure a commit-trigger + RequestReview rule scoped to the watched
    // principal (`dev`). The rule has no path filter — every commit by `dev`
    // opens an assignment.
    let rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::ClaudeCodeSessionId("dev".to_owned())],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    let commit_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: Some("dev".to_owned()),
        changed_paths: vec![],
    };

    let written = process_commit_rules(vec![rule], &commit_ctx, &mut db)?;
    assert_eq!(
        written, 1,
        "exactly one pending assignment must be written for the matching commit"
    );

    let rows = db
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert_eq!(
        rows.len(),
        1,
        "the hook must write exactly one pending assignment row (engine outcome)"
    );
    let assignment = &rows[0];
    assert_eq!(
        assignment.target, WATCHED_BRANCH,
        "assignment target must be the watched branch (commit context target)"
    );
    assert_eq!(
        assignment.reviewer_principal, REVIEWER,
        "assignment reviewer must be the configured reviewer principal from the Action"
    );
    assert_eq!(
        assignment.state,
        but_authz::AssignmentState::Pending.name(),
        "assignment state must be `pending` via AssignmentState::Pending.name() (reuse LPR-002 literal)"
    );

    Ok(())
}

/// AC-2: a commit Trigger -> review-requested Action rule round-trips through
/// the engine's `workspace_rules` persistence. The serde shape of the two new
/// variants matches the sibling variants (`#[serde(rename_all = "camelCase",
/// tag = "type", content = "subject")]`), so the persisted blob round-trips —
/// the engine mechanism is reused, not a parallel mechanism.
#[test]
fn commit_trigger_review_action_variants_persist() -> anyhow::Result<()> {
    let rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::ClaudeCodeSessionId("dev".to_owned())],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    // Round-trip through the serde shape used by `workspace_rules`. The crate
    // serializes `WorkspaceRule` as JSON via `DbHandle::workspace_rules_mut()`;
    // this test exercises the same JSON round-trip to prove the two new
    // variants survive the persisted blob (no side table, no parallel
    // mechanism).
    let persisted = serde_json::to_string(&rule)?;
    let reloaded: WorkspaceRule = serde_json::from_str(&persisted)?;

    assert!(
        matches!(reloaded.trigger(), Trigger::Commit),
        "the commit Trigger variant must round-trip through the persisted rule blob"
    );
    let reviewer = match reloaded.action() {
        Action::RequestReview(RequestReviewAction { reviewer }) => reviewer.clone(),
        _ => panic!(
            "the RequestReview Action variant must round-trip through the persisted rule blob"
        ),
    };
    assert_eq!(
        reviewer, REVIEWER,
        "the reviewer payload must survive the serde round-trip"
    );

    // The full filter set round-trips too — the new variants sit beside the
    // existing siblings without disturbing them.
    assert_eq!(
        reloaded.session_id().as_deref(),
        Some("dev"),
        "the ClaudeCodeSessionId filter (the principal axis) must round-trip"
    );

    Ok(())
}

/// AC-3: the hook is filter-scoped (not every-commit). A commit matching the
/// watched principal opens an assignment; a commit by an unwatched principal
/// does NOT. A second axis (path filter) is exercised to prove the filter is
/// generic — not a hardcoded principal check.
#[test]
fn review_requested_hook_scoped_by_filter() -> anyhow::Result<()> {
    let mut db = but_db::DbHandle::new_at_path(":memory:")?;

    // --- Axis 1: principal filter ---------------------------------------
    let principal_rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::ClaudeCodeSessionId("dev".to_owned())],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    // Watched commit — by `dev` on `feat`. MUST open an assignment.
    let watched_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: Some("dev".to_owned()),
        changed_paths: vec![],
    };
    let written = process_commit_rules(vec![principal_rule.clone()], &watched_ctx, &mut db)?;
    assert_eq!(
        written, 1,
        "watched principal commit MUST open an assignment (filter-scoped)"
    );

    // Unwatched commit — by `other` on `docs`. MUST NOT open an assignment.
    let unwatched_ctx = CommitContext {
        target: UNWATCHED_BRANCH.to_owned(),
        session_id: Some("other".to_owned()),
        changed_paths: vec![],
    };
    let written = process_commit_rules(vec![principal_rule], &unwatched_ctx, &mut db)?;
    assert_eq!(
        written, 0,
        "unwatched principal commit MUST NOT open an assignment (an every-commit hook would fail this)"
    );

    // Exactly one assignment row for the watched branch; ZERO for the unwatched.
    let watched_rows = db
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert_eq!(
        watched_rows.len(),
        1,
        "exactly one assignment — the watched principal commit's — must exist"
    );
    let unwatched_rows = db
        .local_review_assignments()
        .list_by_target(UNWATCHED_BRANCH)?;
    assert!(
        unwatched_rows.is_empty(),
        "NO assignment row may exist for an unwatched commit"
    );

    // --- Axis 2: path filter --------------------------------------------
    // A second rule scoped by `PathMatchesRegex` proves the filter mechanism
    // is the generic engine one — not a hardcoded session-id check.
    let path_rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::PathMatchesRegex(
            regex::Regex::new("src/.*").expect("fixture regex is valid"),
        )],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    // Commit on `feat` touching `src/foo.rs` — matches the path filter.
    let matching_ctx = CommitContext {
        target: "refs/heads/feat-src".to_owned(),
        session_id: None,
        changed_paths: vec!["src/foo.rs".to_owned()],
    };
    let written = process_commit_rules(vec![path_rule.clone()], &matching_ctx, &mut db)?;
    assert_eq!(
        written, 1,
        "a commit touching the watched path MUST fire (path filter matches)"
    );

    // Commit on `feat` touching `docs/README.md` — does NOT match.
    let non_matching_ctx = CommitContext {
        target: "refs/heads/feat-docs".to_owned(),
        session_id: None,
        changed_paths: vec!["docs/README.md".to_owned()],
    };
    let written = process_commit_rules(vec![path_rule], &non_matching_ctx, &mut db)?;
    assert_eq!(
        written, 0,
        "a commit touching an unwatched path MUST NOT fire (path filter scopes the hook)"
    );

    Ok(())
}

/// AC-4: the auto-opened assignment is drive-only — it blocks no commit and
/// gates no merge. The hook fires post-commit (the commit already landed by
/// construction); the row it writes is presentation metadata that the merge
/// gate never reads. The static side of this guarantee — that
/// `enforce_commit_gate` and `enforce_merge_gate` never reference
/// `local_review_assignments` — is proven by `cargo test -p but-authz
/// invariant_build_gates`. This test proves the dynamic side at the but-rules
/// layer:
///
///   1. The hook returns `Ok(n)` — never `Err` — so it cannot block the
///      commit (which has already landed; the hook is post-commit).
///   2. The hook acquires NO repo guard (`process_commit_rules` takes only
///      `&mut DbHandle`, not `&mut RepoExclusive`), so it cannot deadlock the
///      merge gate's own guard acquisition.
///   3. The hook writes ONLY `local_review_assignments` — the merge gate's
///      data source (`local_review_verdicts`) is untouched. Even with the
///      auto-assignment present, the gate's verdict-at-head truth is empty
///      unless an explicit approval was recorded.
#[test]
fn auto_assignment_blocks_no_commit_no_merge() -> anyhow::Result<()> {
    let mut db = but_db::DbHandle::new_at_path(":memory:")?;

    let rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::ClaudeCodeSessionId("dev".to_owned())],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );
    let commit_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: Some("dev".to_owned()),
        changed_paths: vec![],
    };

    // (1) Hook fires — Ok(1), never Err. The commit that triggered it has
    // already landed (post-commit hook) — by construction it is not blocked.
    let written = process_commit_rules(vec![rule], &commit_ctx, &mut db)?;
    assert_eq!(
        written, 1,
        "the hook fires and returns Ok — it never blocks the commit"
    );

    // (2) The auto-opened row is the same inert drive row UC-LPR-01 defines:
    // a pending presentation row, no gate semantics.
    let rows = db
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].state,
        but_authz::AssignmentState::Pending.name(),
        "the auto-opened row is the same inert pending row UC-LPR-01 defines — never a gate"
    );

    // (3) The merge gate reads `local_review_verdicts` (verdict-at-head), NOT
    // `local_review_assignments`. Prove the tables are independent: the hook
    // fires and the assignments table has a row, but the verdicts table is
    // still empty — the gate's data source is structurally unaffected.
    let verdicts = db.local_review_verdicts().list_by_target(WATCHED_BRANCH)?;
    assert!(
        verdicts.is_empty(),
        "the hook writes local_review_assignments only — the merge gate's \
         verdict-at-head source (local_review_verdicts) is structurally untouched, \
         so the open auto-assignment cannot gate a verdict-satisfied merge"
    );

    Ok(())
}

// ----fixture helpers --------------------------------------------------------

/// Build a `WorkspaceRule` with a fixed id/created_at and the given trigger,
/// filters, action. Mirrors the shape `but_rules::insert_rule` builds.
fn workspace_rule(trigger: Trigger, filters: Vec<Filter>, action: Action) -> WorkspaceRule {
    WorkspaceRule::for_test("test-rule-001", trigger, filters, action)
}
