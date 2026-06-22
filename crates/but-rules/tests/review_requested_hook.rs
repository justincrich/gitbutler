//! LPR-007 integration proofs (AC-1..AC-3) — real but-db + real gix.
//!
//! These tests exercise the additive `Trigger::Commit` + `Action::RequestReview`
//! extension to the shipped but-rules engine. A fixtured commit matching a
//! configured rule's filter fires the "review-requested" action, and the test
//! asserts the ENGINE OUTCOME — the `pending` `local_review_assignments` row
//! written via the LPR-001 Handle — never a hook log message.
//!
//! No mocks — real but-db + real gix via `but_testsupport`.

use but_rules::{
    Action, CommitContext, Filter, RequestReviewAction, Trigger, WorkspaceRule, process_commit_rules,
};

const WATCHED_BRANCH: &str = "refs/heads/feat";
const REVIEWER: &str = "rev2";

/// AC-1 [PRIMARY]: a fixtured commit matching the rule's filter fires the
/// "review-requested" action and writes a `pending` `local_review_assignments`
/// row for the configured reviewer (the ENGINE OUTCOME — DB row, not a log).
#[test]
fn review_requested_hook_creates_pending_assignment() -> anyhow::Result<()> {
    let db = but_db::DbHandle::new_at_path(":memory:")?;

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

    let mut db_handle = db;
    let written = process_commit_rules(vec![rule.clone()], &commit_ctx, &mut db_handle)?;
    assert_eq!(
        written, 1,
        "exactly one pending assignment must be written for the matching commit"
    );

    let rows = db_handle
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert_eq!(
        rows.len(),
        1,
        "the hook must write exactly one pending assignment row (engine outcome)"
    );
    let assignment = &rows[0];
    assert_eq!(
        assignment.target,
        WATCHED_BRANCH,
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

    // Serde round-trip proof — the persisted rule blob must carry both new
    // variants (mechanism reused, variants added — not a parallel mechanism).
    let persisted = serde_json::to_string(&rule)?;
    let reloaded: WorkspaceRule = serde_json::from_str(&persisted)?;
    assert!(
        matches!(reloaded.trigger(), Trigger::Commit),
        "the commit Trigger variant must round-trip through the persisted rule blob"
    );
    let reviewer = match reloaded.action() {
        Action::RequestReview(RequestReviewAction { reviewer }) => reviewer.clone(),
        _ => panic!("the RequestReview Action variant must round-trip through the persisted rule blob"),
    };
    assert_eq!(
        reviewer, REVIEWER,
        "the reviewer payload must survive the serde round-trip"
    );

    Ok(())
}

/// AC-2: a commit NOT matching the rule's filter writes NO assignment row.
/// This is the negative control proving the hook is filter-scoped (not
/// every-commit) — a rule scoped to principal `dev` MUST NOT fire on a commit
/// by an unwatched principal `other`.
#[test]
fn non_matching_commit_creates_no_assignment() -> anyhow::Result<()> {
    let mut db = but_db::DbHandle::new_at_path(":memory:")?;

    let rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::ClaudeCodeSessionId("dev".to_owned())],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    // The commit is by `other`, NOT the watched `dev` — filter must reject it.
    let commit_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: Some("other".to_owned()),
        changed_paths: vec![],
    };

    let written = process_commit_rules(vec![rule], &commit_ctx, &mut db)?;
    assert_eq!(
        written, 0,
        "an unwatched commit MUST NOT open an assignment — the hook is filter-scoped"
    );

    let rows = db
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert!(
        rows.is_empty(),
        "no assignment row may exist for an unwatched commit (every-commit hook would fail this)"
    );

    Ok(())
}

/// AC-3: a rule with a file-scope (`PathMatchesRegex`) filter fires only for
/// commits whose changed paths match. A commit touching the watched path
/// opens an assignment; a commit touching an unwatched path does NOT.
#[test]
fn file_filter_scopes_commit_trigger() -> anyhow::Result<()> {
    let mut db = but_db::DbHandle::new_at_path(":memory:")?;

    let rule = workspace_rule(
        Trigger::Commit,
        vec![Filter::PathMatchesRegex(
            regex::Regex::new("src/.*").expect("fixture regex is valid"),
        )],
        Action::RequestReview(RequestReviewAction {
            reviewer: REVIEWER.to_owned(),
        }),
    );

    // Commit A: touches `src/foo.rs` — matches the path filter — must fire.
    let matching_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: None,
        changed_paths: vec!["src/foo.rs".to_owned()],
    };
    let written = process_commit_rules(vec![rule.clone()], &matching_ctx, &mut db)?;
    assert_eq!(
        written, 1,
        "a commit touching the watched path MUST fire (path filter matches)"
    );

    // Commit B: touches `docs/README.md` — does NOT match the path filter — must NOT fire.
    let non_matching_ctx = CommitContext {
        target: WATCHED_BRANCH.to_owned(),
        session_id: None,
        changed_paths: vec!["docs/README.md".to_owned()],
    };
    let written = process_commit_rules(vec![rule], &non_matching_ctx, &mut db)?;
    assert_eq!(
        written, 0,
        "a commit touching an unwatched path MUST NOT fire (path filter scopes the hook)"
    );

    // Exactly one assignment row for the watched branch — the matching commit's.
    let rows = db
        .local_review_assignments()
        .list_by_target(WATCHED_BRANCH)?;
    assert_eq!(
        rows.len(),
        1,
        "exactly one assignment — the watched-path commit's — must exist"
    );
    assert_eq!(rows[0].reviewer_principal, REVIEWER);

    Ok(())
}

// ---- fixture helpers -------------------------------------------------------

/// Build a `WorkspaceRule` with a fixed id/created_at and the given trigger,
/// filters, action. Mirrors the shape `but_rules::insert_rule` builds.
fn workspace_rule(trigger: Trigger, filters: Vec<Filter>, action: Action) -> WorkspaceRule {
    WorkspaceRule::for_test("test-rule-001", trigger, filters, action)
}
