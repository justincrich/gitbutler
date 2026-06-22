use but_core::{ChangeId, RefMetadata, sync::RepoExclusive};
use but_ctx::Context;
use but_db::DbHandle;
use serde::{Deserialize, Serialize};

pub mod db;
pub mod handler;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRule {
    id: String,
    created_at: chrono::NaiveDateTime,
    enabled: bool,
    trigger: Trigger,
    filters: Vec<Filter>,
    action: Action,
}

impl WorkspaceRule {
    /// If the rule has a session ID filter, this returns the first one found.
    pub fn session_id(&self) -> Option<String> {
        self.filters.iter().find_map(|f| match f {
            Filter::ClaudeCodeSessionId(id) => Some(id.clone()),
            _ => None,
        })
    }

    /// Return a reference to the rule's filters.
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Return the rule's trigger.
    pub fn trigger(&self) -> &Trigger {
        &self.trigger
    }

    /// Return the rule's action.
    pub fn action(&self) -> &Action {
        &self.action
    }

    /// Returns the target stack ID if the action is an explicit assignment operation.
    pub fn target_stack_id(&self) -> Option<String> {
        if let Action::Explicit(Operation::Assign { target }) = &self.action {
            match target {
                StackTarget::StackId(id) => Some(id.clone()),
                StackTarget::Leftmost | StackTarget::Rightmost => None,
            }
        } else {
            None
        }
    }

    /// Return the target change ID if its action is an explicit amend operation.
    pub fn target_change_id(&self) -> Option<ChangeId> {
        if let Action::Explicit(Operation::Amend { change_id }) = &self.action {
            Some(change_id.clone())
        } else {
            None
        }
    }

    /// Return the persistent rule ID.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    /// Return the creation timestamp.
    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

    /// Construct a `WorkspaceRule` with a fixed id/created_at for tests.
    #[doc(hidden)]
    pub fn for_test(
        id: &str,
        trigger: Trigger,
        filters: Vec<Filter>,
        action: Action,
    ) -> WorkspaceRule {
        WorkspaceRule {
            id: id.to_owned(),
            created_at: chrono::Local::now().naive_local(),
            enabled: true,
            trigger,
            filters,
            action,
        }
    }
}

/// Represents the kinds of events in the app that can cause a rule to be evaluated.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    /// When a file is added, removed or modified in the Git worktree.
    FileSytemChange,
    /// Whenever a Claude Code hook is invoked.
    ClaudeCodeHook,
    /// When a commit lands on a watched branch. The commit context (target
    /// branch, changed paths, originating Claude Code session) is supplied at
    /// firing time via [`CommitContext`] to [`process_commit_rules`]. Used by
    /// the LPR-007 "review-requested" hook to open a pending drive-only
    /// `local_review_assignments` row — never blocks the commit.
    Commit,
}

/// A filter is a condition that determines what files or changes the rule applies to.
/// Within a filter, multiple conditions are combined with AND logic (i.e. to match all conditions must be met)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Filter {
    /// Matches the file path (relative to the repository root).
    #[serde(with = "serde_regex")]
    PathMatchesRegex(regex::Regex),
    /// Match the file content.
    #[serde(with = "serde_regex")]
    ContentMatchesRegex(regex::Regex),
    /// Matches the file change operation type (e.g. addition, deletion, modification, rename)
    FileChangeType(TreeStatus),
    /// Matches the semantic type of the change.
    SemanticType(SemanticType),
    /// Matches changes that originated from a specific Claude Code session.
    ClaudeCodeSessionId(String),
}

/// Represents the type of change that occurred in the Git worktree.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TreeStatus {
    Addition,
    Deletion,
    Modification,
    Rename,
}

/// Represents a semantic type of change that was inferred for the change.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum SemanticType {
    Refactor,
    NewFeature,
    BugFix,
    Documentation,
    UserDefined(String),
}

/// Represents an action that can be taken based on the rule evaluation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Action {
    /// An action that has an explicit operation defined by the user.
    Explicit(Operation),
    /// An action where the operation is determined by heuristics or AI.
    Implicit(ImplicitOperation),
    /// Open a `pending` `local_review_assignments` row for the configured
    /// [`RequestReviewAction::reviewer`] on the commit's branch. Drive-only —
    /// the row never enters the commit gate or merge gate (LPR-007). Reuses
    /// the LPR-001 Handle and the LPR-003 pending-assignment write internals.
    RequestReview(RequestReviewAction),
}

/// Payload for [`Action::RequestReview`] — the reviewer principal to assign.
///
/// ```
/// use but_rules::RequestReviewAction;
/// let action = RequestReviewAction { reviewer: "rev2".to_owned() };
/// assert_eq!(action.reviewer, "rev2");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestReviewAction {
    /// The reviewer principal id to assign (mirrors the `reviewer_principal`
    /// column on `local_review_assignments`).
    pub reviewer: String,
}

/// Represents the operation that a user can configure to be performed in an explicit action.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Operation {
    /// Assign the matched changes to a specific stack ID.
    Assign { target: StackTarget },
    /// Amend the matched changes into a specific commit.
    Amend { change_id: ChangeId },
    /// Create a new commit with the matched changes on a specific branch.
    NewCommit { branch_name: String },
}

/// The target stack for a given operation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum StackTarget {
    StackId(String),
    Leftmost,
    Rightmost,
}

/// Represents the implicit operation that is determined by heuristics or AI.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum ImplicitOperation {
    /// Assign the matched changes to the appropriate branch based on offline heuristics.
    AssignToAppropriateBranch,
    /// Absorb the matched changes into a dependent commit based on offline heuristics.
    AbsorbIntoDependentCommit,
    /// Perform an operation based on LLM-driven analysis and tool calling.
    LLMPrompt(String),
}

/// A request to create a new workspace rule.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub trigger: Trigger,
    pub filters: Vec<Filter>,
    pub action: Action,
}

/// Create a new workspace rule and attempt to reevaluate all workspace rules.
pub fn create_rule(
    ctx: &mut Context,
    req: CreateRuleRequest,
    perm: &mut RepoExclusive,
) -> anyhow::Result<WorkspaceRule> {
    let rule = {
        let mut db = ctx.db.get_cache_mut()?;
        insert_rule(&mut db, req)?
    };
    process_rules_from_context(ctx, perm).ok();
    Ok(rule)
}

fn insert_rule(db: &mut DbHandle, req: CreateRuleRequest) -> anyhow::Result<WorkspaceRule> {
    let rule = WorkspaceRule {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Local::now().naive_local(),
        enabled: true,
        trigger: req.trigger,
        filters: req.filters,
        action: req.action,
    };

    db.workspace_rules_mut().insert(rule.clone().try_into()?)?;
    Ok(rule)
}

/// Delete the workspace rule with `id` from `db`.
pub fn delete_rule(db: &mut DbHandle, id: &str) -> anyhow::Result<()> {
    db.workspace_rules_mut().delete(id)?;
    Ok(())
}

/// A request to update an existing workspace rule.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRuleRequest {
    id: String,
    pub enabled: Option<bool>,
    pub trigger: Option<Trigger>,
    pub filters: Option<Vec<Filter>>,
    pub action: Option<Action>,
}

impl From<WorkspaceRule> for UpdateRuleRequest {
    fn from(rule: WorkspaceRule) -> Self {
        UpdateRuleRequest {
            id: rule.id,
            enabled: Some(rule.enabled),
            trigger: Some(rule.trigger),
            filters: Some(rule.filters),
            action: Some(rule.action),
        }
    }
}

/// Update an existing workspace rule and attempt to reevaluate all workspace rules.
pub fn update_rule(
    ctx: &mut Context,
    req: UpdateRuleRequest,
    perm: &mut RepoExclusive,
) -> anyhow::Result<WorkspaceRule> {
    let rule = {
        let mut db = ctx.db.get_cache_mut()?;
        update_rule_record(&mut db, req)?
    };
    process_rules_from_context(ctx, perm).ok();
    Ok(rule)
}

fn update_rule_record(db: &mut DbHandle, req: UpdateRuleRequest) -> anyhow::Result<WorkspaceRule> {
    let mut rule: WorkspaceRule = {
        db.workspace_rules()
            .get(&req.id)?
            .ok_or_else(|| anyhow::anyhow!("Rule with ID {} not found", req.id))?
            .try_into()?
    };

    if let Some(enabled) = req.enabled {
        rule.enabled = enabled;
    }
    if let Some(trigger) = req.trigger {
        rule.trigger = trigger;
    }
    if let Some(filters) = req.filters {
        rule.filters = filters;
    }
    if let Some(action) = req.action {
        rule.action = action;
    }

    db.workspace_rules_mut()
        .update(&req.id, rule.clone().try_into()?)?;
    Ok(rule)
}

/// Retrieve the workspace rule with `id` from `db`.
pub fn get_rule(db: &DbHandle, id: &str) -> anyhow::Result<WorkspaceRule> {
    let rule = db
        .workspace_rules()
        .get(id)?
        .ok_or_else(|| anyhow::anyhow!("Rule with ID {id} not found"))?
        .try_into()?;
    Ok(rule)
}

/// List all workspace rules stored in `db`.
pub fn list_rules(db: &DbHandle) -> anyhow::Result<Vec<WorkspaceRule>> {
    let rules = db
        .workspace_rules()
        .list()?
        .into_iter()
        .map(|r| r.try_into())
        .collect::<Result<Vec<WorkspaceRule>, _>>()?;
    Ok(rules)
}

fn process_rules_from_context(ctx: &mut Context, perm: &mut RepoExclusive) -> anyhow::Result<()> {
    let context_lines = ctx.settings.context_lines;
    let mut meta = ctx.meta()?;
    let (repo, mut ws, mut db) = ctx.workspace_mut_and_db_mut_with_perm(perm)?;
    let rules = list_rules(&db)?;
    process_rules(
        rules,
        &repo,
        &mut ws,
        &mut db,
        &mut meta,
        perm,
        context_lines,
    )
}

/// Reevaluate `rules` against current worktree changes and apply matching actions.
///
/// NOTE: may create an empty branch!
pub fn process_rules(
    rules: Vec<WorkspaceRule>,
    repo: &gix::Repository,
    ws: &mut but_graph::Workspace,
    db: &mut DbHandle,
    meta: &mut impl RefMetadata,
    perm: &mut RepoExclusive,
    context_lines: u32,
) -> anyhow::Result<()> {
    let assignments = {
        let wt_changes = but_core::diff::worktree_changes(repo)?;

        let (assignments, _) = but_hunk_assignment::assignments_with_fallback(
            db.hunk_assignments_mut()?,
            repo,
            ws,
            Some(wt_changes.changes),
            context_lines,
        )
        .map_err(|e| anyhow::anyhow!("Failed to get assignments: {e}"))?;
        assignments
    };

    handler::process_workspace_rules(rules, &assignments, repo, ws, db, meta, perm, context_lines)?;
    Ok(())
}

/// The commit-landed firing context consumed by [`process_commit_rules`].
///
/// `target` is the branch ref the commit landed on. `session_id` is the
/// originating Claude Code session, if any — the principal axis the rule's
/// [`Filter::ClaudeCodeSessionId`] scopes on. `changed_paths` are the paths the
/// commit touched (relative to the repo root) — the file axis the rule's
/// [`Filter::PathMatchesRegex`] scopes on.
///
/// This is drive metadata only — it never reaches the commit gate (the commit
/// already landed) and the row it fires never reaches the merge gate.
#[derive(Debug, Clone)]
pub struct CommitContext {
    /// The branch ref the commit landed on, e.g. `refs/heads/feat`.
    pub target: String,
    /// The originating Claude Code session id, if any (the principal axis).
    pub session_id: Option<String>,
    /// The paths touched by the commit, relative to the repo root.
    pub changed_paths: Vec<String>,
}

/// Reevaluate commit-trigger `rules` against `commit_ctx` and fire any
/// `Action::RequestReview` matches by writing a `pending`
/// `local_review_assignments` row for the configured reviewer.
///
/// This is the LPR-007 "review-requested" hook — the post-commit drive-metadata
/// write that opens an assignment so the reconciler can dispatch a reviewer
/// without an explicit `but review request`. Only enabled rules whose trigger
/// is [`Trigger::Commit`] and whose action is [`Action::RequestReview`] are
/// evaluated; rules with any other trigger or action are ignored. The rule's
/// filters are evaluated against `commit_ctx` with AND semantics.
///
/// Reuses the LPR-001 `local_review_assignments_mut().upsert(...)` Handle and
/// the `but_authz::AssignmentState::Pending.name()` literal — never forks a
/// parallel writer. The hook is a post-commit drive-metadata write — it NEVER
/// blocks the commit (the commit already landed) and the row it writes NEVER
/// gates a merge.
pub fn process_commit_rules(
    rules: Vec<WorkspaceRule>,
    commit_ctx: &CommitContext,
    db: &mut DbHandle,
) -> anyhow::Result<usize> {
    handler::process_commit_rules(rules, commit_ctx, db)
}
