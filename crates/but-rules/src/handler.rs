use std::str::FromStr;

use anyhow::ensure;
use but_core::{ChangeId, DiffSpec, RefMetadata, ref_metadata::StackId, sync::RepoExclusive};
use but_db::{DbHandle, HunkAssignmentsHandleMut};
use but_hunk_assignment::HunkAssignment;
use but_rebase::graph_rebase::Editor;
use itertools::Itertools;

use crate::{
    Action, CommitContext, Filter, RequestReviewAction, StackTarget, Trigger, WorkspaceRule,
};

/// Apply matching workspace `rules` to the current worktree `assignments`.
#[expect(clippy::too_many_arguments)]
pub fn process_workspace_rules(
    rules: Vec<WorkspaceRule>,
    assignments: &[HunkAssignment],
    repo: &gix::Repository,
    ws: &mut but_graph::Workspace,
    db: &mut DbHandle,
    meta: &mut impl RefMetadata,
    perm: &mut RepoExclusive,
    context_lines: u32,
) -> anyhow::Result<usize> {
    let mut updates = 0;
    if assignments.is_empty() {
        return Ok(updates);
    }
    let rules = rules
        .into_iter()
        .filter(|r| r.enabled)
        .filter(|r| matches!(r.trigger, super::Trigger::FileSytemChange))
        .filter(|r| {
            matches!(
                &r.action,
                super::Action::Explicit(super::Operation::Assign { .. })
            ) || matches!(
                &r.action,
                super::Action::Explicit(super::Operation::Amend { .. })
            )
        })
        .collect_vec();

    if rules.is_empty() {
        return Ok(updates);
    }

    let stack_ids: Vec<_> = ws.stacks.iter().filter_map(|s| s.id).collect();
    let mut new_ws = None;

    for rule in rules {
        match rule.action {
            super::Action::Explicit(super::Operation::Assign { target }) => {
                if let Some((stack_id, maybe_new_ws)) =
                    get_or_create_stack_id(repo, ws, meta, perm, target, &stack_ids)
                {
                    if let Some(ws) = maybe_new_ws {
                        ensure!(
                            new_ws.is_none(),
                            "BUG: new stacks are only created once if there are no stacks"
                        );
                        new_ws = Some(ws);
                    }
                    let assignments = matching(assignments, rule.filters.clone())
                        .into_iter()
                        .filter(|e| e.stack_id != Some(stack_id))
                        .map(|mut e| {
                            e.stack_id = Some(stack_id);
                            e.branch_ref_bytes = None;
                            e
                        })
                        .collect_vec();
                    updates += handle_assign(
                        db.hunk_assignments_mut()?,
                        repo,
                        new_ws.as_ref().unwrap_or(&*ws),
                        assignments,
                        context_lines,
                    )
                    .unwrap_or_default();
                }
            }
            super::Action::Explicit(super::Operation::Amend { change_id }) => {
                let assignments = matching(assignments, rule.filters.clone());
                let ws = if let Some(new_ws) = new_ws.as_mut() {
                    new_ws
                } else {
                    &mut *ws
                };
                handle_amend(repo, ws, meta, assignments, &change_id, context_lines)
                    .unwrap_or_default();
            }
            // The remaining `Action` variants have no FileSytemChange handler:
            //  - `Explicit(NewCommit)` and `Implicit(*)` are pre-filtered above
            //    and have no meaning against hunk assignments.
            //  - `RequestReview` is a Commit-trigger action handled by
            //    `process_commit_rules` below; it MUST NOT silently fire on a
            //    FileSytemChange trigger. The explicit arm keeps this match
            //    exhaustive — no wildcard that could hide a future variant.
            super::Action::Explicit(super::Operation::NewCommit { .. })
            | super::Action::Implicit(_) => continue,
            super::Action::RequestReview(_) => continue,
        };
    }

    if let Some(new_ws) = new_ws {
        *ws = new_ws;
    }

    Ok(updates)
}

fn handle_amend(
    repo: &gix::Repository,
    ws: &mut but_graph::Workspace,
    meta: &mut impl but_core::RefMetadata,
    assignments: Vec<HunkAssignment>,
    change_id: &ChangeId,
    context_lines: u32,
) -> anyhow::Result<()> {
    let changes: Vec<DiffSpec> =
        but_workspace::flatten_diff_specs(assignments.into_iter().map(DiffSpec::from));
    let mut commit_id: Option<gix::ObjectId> = None;
    'outer: for commit in ws.commits() {
        let commit_change_id = commit.attach(repo)?.headers().and_then(|hdr| hdr.change_id);
        if commit_change_id.is_some_and(|cid| cid == *change_id) {
            commit_id = Some(commit.id);
            break 'outer;
        }
    }

    let commit_id = commit_id.ok_or_else(|| {
        anyhow::anyhow!("No commit with Change-Id {change_id} found in the current workspace")
    })?;

    let editor = Editor::create(ws, meta, repo)?;
    let outcome = but_workspace::commit::commit_amend(editor, commit_id, changes, context_lines)?;
    if !outcome.rejected_specs.is_empty() {
        tracing::warn!(
            ?outcome.rejected_specs,
            "Failed to commit at least one hunk"
        );
    }
    outcome.rebase.materialize()?;
    Ok(())
}

fn get_or_create_stack_id(
    repo: &gix::Repository,
    ws: &but_graph::Workspace,
    meta: &mut impl but_core::RefMetadata,
    perm: &mut RepoExclusive,
    target: StackTarget,
    stack_ids_in_ws: &[StackId],
) -> Option<(StackId, Option<but_graph::Workspace>)> {
    match target {
        StackTarget::StackId(stack_id) => {
            if let Ok(stack_id) = StackId::from_str(&stack_id) {
                if stack_ids_in_ws.iter().any(|e| e == &stack_id) {
                    Some((stack_id, None))
                } else {
                    None
                }
            } else {
                None
            }
        }
        StackTarget::Leftmost => {
            if stack_ids_in_ws.is_empty() {
                create_stack(repo, ws, meta, perm)
                    .ok()
                    .map(|(id, ws)| (id, Some(ws)))
            } else {
                stack_ids_in_ws.first().copied().map(|id| (id, None))
            }
        }
        StackTarget::Rightmost => {
            if stack_ids_in_ws.is_empty() {
                create_stack(repo, ws, meta, perm)
                    .ok()
                    .map(|(id, ws)| (id, Some(ws)))
            } else {
                stack_ids_in_ws.last().copied().map(|id| (id, None))
            }
        }
    }
}

fn create_stack(
    repo: &gix::Repository,
    ws: &but_graph::Workspace,
    meta: &mut impl but_core::RefMetadata,
    _perm: &mut RepoExclusive,
) -> anyhow::Result<(StackId, but_graph::Workspace)> {
    use anyhow::Context;
    let branch_name = but_core::branch::unique_canned_refname(repo)?;
    let new_ws = but_workspace::branch::create_reference(
        branch_name.as_ref(),
        None,
        repo,
        ws,
        meta,
        |_| StackId::generate(),
        None,
    )?;
    let (stack, _) = new_ws
        .find_segment_and_stack_by_refname(branch_name.as_ref())
        .context("BUG: need to find stack that was just created")?;
    stack
        .id
        .context("BUG: newly created stacks always have an ID")
        .map(|id| (id, new_ws.into_owned()))
}

fn handle_assign(
    db: HunkAssignmentsHandleMut,
    repo: &gix::Repository,
    workspace: &but_graph::Workspace,
    assignments: Vec<HunkAssignment>,
    context_lines: u32,
) -> anyhow::Result<usize> {
    let len = assignments.len();
    but_hunk_assignment::assign(
        db,
        repo,
        workspace,
        but_hunk_assignment::assignments_to_requests(assignments),
        context_lines,
    )
    .map(|()| len)
    .or_else(|_| Ok(0))
}

fn matching(wt_assignments: &[HunkAssignment], filters: Vec<Filter>) -> Vec<HunkAssignment> {
    if filters.is_empty() {
        return wt_assignments.to_vec();
    }
    let mut assignments = Vec::new();
    for filter in filters {
        match filter {
            Filter::PathMatchesRegex(regex) => {
                for change in wt_assignments.iter() {
                    if regex.is_match(&change.path) {
                        assignments.push(change.clone());
                    }
                }
            }
            Filter::ContentMatchesRegex(regex) => {
                for change in wt_assignments.iter() {
                    if let Some(diff) = change.diff.clone() {
                        let diff = diff.to_string();
                        let matching_lines: Vec<&str> =
                            diff.lines().filter(|line| line.starts_with('+')).collect();
                        if matching_lines.iter().any(|line| regex.is_match(line)) {
                            assignments.push(change.clone());
                        }
                    }
                }
            }
            Filter::FileChangeType(_) => continue,
            Filter::SemanticType(_) => continue,
            Filter::ClaudeCodeSessionId(_) => continue,
        }
    }
    assignments
}

// ---- LPR-007: commit Trigger + RequestReview Action firing path -------------

/// Reevaluate commit-trigger `rules` against `commit_ctx`, firing the
/// `Action::RequestReview` matches by writing a `pending`
/// `local_review_assignments` row for the configured reviewer.
///
/// See [`crate::process_commit_rules`] for the public contract.
pub(crate) fn process_commit_rules(
    rules: Vec<WorkspaceRule>,
    commit_ctx: &CommitContext,
    db: &mut DbHandle,
) -> anyhow::Result<usize> {
    let mut written = 0usize;
    for rule in rules
        .into_iter()
        .filter(|r| r.enabled)
        .filter(|r| matches!(r.trigger, Trigger::Commit))
    {
        let reviewer = match &rule.action {
            Action::RequestReview(RequestReviewAction { reviewer }) => reviewer.clone(),
            // Other actions have no commit-trigger firing path — skip them
            // explicitly so a new action variant can never silently no-op
            // behind a wildcard.
            Action::Explicit(_) | Action::Implicit(_) => continue,
        };
        if !commit_matches_filters(&rule, commit_ctx) {
            continue;
        }
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: uuid::Uuid::new_v4().to_string(),
                target: commit_ctx.target.clone(),
                reviewer_principal: reviewer,
                state: but_authz::AssignmentState::Pending.name().to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
        written += 1;
    }
    Ok(written)
}

/// Evaluate `rule`'s filters against `commit_ctx` with AND semantics.
///
/// [`Filter::PathMatchesRegex`] matches if any of the commit's changed paths
/// matches (file axis). [`Filter::ClaudeCodeSessionId`] matches if the commit's
/// session id equals the filter's id (principal axis). The remaining filter
/// variants need info that a [`CommitContext`] does not carry — they are
/// treated as non-matching (a commit rule carrying them silently no-ops rather
/// than firing on every commit).
fn commit_matches_filters(rule: &WorkspaceRule, commit_ctx: &CommitContext) -> bool {
    if rule.filters.is_empty() {
        return true;
    }
    rule.filters.iter().all(|filter| match filter {
        Filter::PathMatchesRegex(regex) => commit_ctx
            .changed_paths
            .iter()
            .any(|path| regex.is_match(path)),
        Filter::ClaudeCodeSessionId(id) => commit_ctx.session_id.as_deref() == Some(id.as_str()),
        Filter::ContentMatchesRegex(_) | Filter::FileChangeType(_) | Filter::SemanticType(_) => {
            false
        }
    })
}
