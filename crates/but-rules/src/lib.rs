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
    pub fn session_id(&self) -> Option<String> {
        self.filters.iter().find_map(|f| match f {
            Filter::ClaudeCodeSessionId(id) => Some(id.clone()),
            _ => None,
        })
    }

    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    pub fn trigger(&self) -> &Trigger {
        &self.trigger
    }

    pub fn action(&self) -> &Action {
        &self.action
    }

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

    pub fn target_change_id(&self) -> Option<ChangeId> {
        if let Action::Explicit(Operation::Amend { change_id }) = &self.action {
            Some(change_id.clone())
        } else {
            None
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn created_at(&self) -> chrono::NaiveDateTime {
        self.created_at
    }

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Trigger {
    FileSytemChange,
    ClaudeCodeHook,
    /// LPR-007: when a commit lands on a watched branch. The commit context
    /// is supplied at firing time via [`CommitContext`] to
    /// [`process_commit_rules`]. Opens a pending drive-only
    /// `local_review_assignments` row — never blocks the commit.
    Commit,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Filter {
    #[serde(with = "serde_regex")]
    PathMatchesRegex(regex::Regex),
    #[serde(with = "serde_regex")]
    ContentMatchesRegex(regex::Regex),
    FileChangeType(TreeStatus),
    SemanticType(SemanticType),
    ClaudeCodeSessionId(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum TreeStatus {
    Addition,
    Deletion,
    Modification,
    Rename,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum SemanticType {
    Refactor,
    NewFeature,
    BugFix,
    Documentation,
    UserDefined(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Action {
    Explicit(Operation),
    Implicit(ImplicitOperation),
    /// LPR-007: open a `pending` `local_review_assignments` row for the
    /// configured reviewer on the commit's branch. Drive-only — never enters
    /// the commit gate or merge gate. Reuses the LPR-001 Handle / LPR-003
    /// pending-assignment write internals.
    RequestReview(RequestReviewAction),
}

/// Payload for [`Action::RequestReview`].
///
/// ```
/// use but_rules::RequestReviewAction;
/// let action = RequestReviewAction { reviewer: "rev2".to_owned() };
/// assert_eq!(action.reviewer, "rev2");
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestReviewAction {
    pub reviewer: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum Operation {
    Assign { target: StackTarget },
    Amend { change_id: ChangeId },
    NewCommit { branch_name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum StackTarget {
    StackId(String),
    Leftmost,
    Rightmost,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", tag = "type", content = "subject")]
pub enum ImplicitOperation {
    AssignToAppropriateBranch,
    AbsorbIntoDependentCommit,
    LLMPrompt(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateRuleRequest {
    pub trigger: Trigger,
    pub filters: Vec<Filter>,
    pub action: Action,
}

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

pub fn delete_rule(db: &mut DbHandle, id: &str) -> anyhow::Result<()> {
    db.workspace_rules_mut().delete(id)?;
    Ok(())
}

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

pub fn get_rule(db: &DbHandle, id: &str) -> anyhow::Result<WorkspaceRule> {
    let rule = db
        .workspace_rules()
        .get(id)?
        .ok_or_else(|| anyhow::anyhow!("Rule with ID {id} not found"))?
        .try_into()?;
    Ok(rule)
}

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
/// originating Claude Code session (the principal axis). `changed_paths` are
/// the paths the commit touched (the file axis). Drive metadata only — never
/// reaches the commit gate or merge gate.
#[derive(Debug, Clone)]
pub struct CommitContext {
    pub target: String,
    pub session_id: Option<String>,
    pub changed_paths: Vec<String>,
}

/// Reevaluate commit-trigger `rules` against `commit_ctx` and fire any
/// `Action::RequestReview` matches by writing a `pending`
/// `local_review_assignments` row for the configured reviewer.
///
/// This is the LPR-007 "review-requested" hook. Only enabled rules whose
/// trigger is [`Trigger::Commit`] and whose action is [`Action::RequestReview`]
/// are evaluated; other rules are ignored. Filters are evaluated with AND
/// semantics. Reuses the LPR-001 Handle + `but_authz::AssignmentState::Pending`
/// literal — never forks a parallel writer. The hook is post-commit
/// drive-metadata — it NEVER blocks the commit and the row NEVER gates a merge.
pub fn process_commit_rules(
    rules: Vec<WorkspaceRule>,
    commit_ctx: &CommitContext,
    db: &mut DbHandle,
) -> anyhow::Result<usize> {
    handler::process_commit_rules(rules, commit_ctx, db)
}
