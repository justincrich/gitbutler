use std::fmt::Display;

use anyhow::Context as _;
use bstr::{BString, ByteSlice};
use but_api::{diff::ComputeLineStats, json::HexHash};
use but_core::{
    DiffSpec, DryRun, RefMetadata,
    diff::CommitDetails,
    ref_metadata::StackId,
    sync::{RepoExclusive, RepoExclusiveGuard},
};
use but_ctx::Context;
use but_error::Code;
use but_rebase::graph_rebase::mutate::{InsertSide, RelativeTo};
use but_transaction::{DynamicOutcome, IntermediateCommitCreateResult, Transaction};
use but_workspace::{
    RefInfo,
    branch::create_reference::{Anchor, Position},
};
use gitbutler_oplog::entry::{OperationKind, SnapshotDetails};
use gix::{prelude::ObjectIdExt as _, refs::FullName};
use nonempty::NonEmpty;
use serde::Serialize;

use crate::{
    CliError, CliId, CliResult, CliResultExt, IdMap,
    args::{
        atoms::{BranchArg, BranchOrCommit, CliIdArg, Purpose},
        commit2::Platform,
    },
    bad_input,
    command::legacy::{
        ShowDiffInEditor,
        reword::get_commit_message_from_editor,
        status::{TuiOutcome, TuiRunOptions, tui_with_options},
    },
    id::{UNASSIGNED, UncommittedCliId},
    theme::{self, Theme},
    utils::{
        CliOutput, CliOutputHuman, IntermediateChannel, WriteWithUtils, diff_specs::DiffSpecBuilder,
    },
};

#[must_use]
pub struct CommitOutcome {
    new_commit: gix::ObjectId,
    branch_name: Option<BranchNameTarget>,
}

/// `--format json` should only include newly created things. So if the branch already existed it
/// wont be included in the JSON output.
enum BranchNameTarget {
    Existing(FullName),
    New(FullName),
}

impl CliOutputHuman for CommitOutcome {
    fn on_human(self, out: &mut dyn WriteWithUtils, _theme: &Theme) -> anyhow::Result<()> {
        let Self {
            new_commit,
            branch_name,
        } = self;

        match branch_name {
            Some(BranchNameTarget::New(branch_name)) => writeln!(
                out,
                "Created commit {} on new branch {}",
                theme::Commit(new_commit),
                theme::Branch(branch_name.shorten().as_bstr())
            )?,
            Some(BranchNameTarget::Existing(branch_name)) => writeln!(
                out,
                "Created commit {} on branch {}",
                theme::Commit(new_commit),
                theme::Branch(branch_name.shorten().as_bstr())
            )?,
            None => writeln!(out, "Created commit {}", theme::Commit(new_commit))?,
        }

        Ok(())
    }
}

impl CliOutput for CommitOutcome {
    fn on_shell(self, out: &mut dyn WriteWithUtils) -> anyhow::Result<()> {
        let Self {
            new_commit,
            branch_name: _,
        } = self;
        writeln!(out, "{}", new_commit.to_hex_with_len(7))?;
        Ok(())
    }

    fn on_json(self) -> impl serde::Serialize {
        #[derive(Serialize)]
        struct Output {
            commit: HexHash,
            #[serde(skip_serializing_if = "Option::is_none")]
            branch: Option<String>,
        }

        let Self {
            new_commit,
            branch_name,
        } = self;

        let branch_name = match branch_name {
            Some(BranchNameTarget::New(branch_name)) => Some(branch_name.shorten().to_string()),
            _ => None,
        };

        Output {
            commit: new_commit.into(),
            branch: branch_name,
        }
    }
}

pub fn commit(
    ctx: &mut Context,
    mut out: IntermediateChannel<'_>,
    args: Platform,
) -> CliResult<CommitOutcome> {
    let Platform {
        no_message,
        message,
        branch,
        empty,
        above,
        below,
        interactive,
        changes,
    } = args;
    let target_ish = commit_operation_target_ish(branch, above, below)?;
    let reword_op = reword_operation(no_message, message);

    let (commit_op, id_map) = {
        let guard = ctx.shared_worktree_access();
        let id_map = IdMap::new_from_context(ctx, None, guard.read_permission())?;
        let head_info = but_api::legacy::workspace::head_info(ctx)?;
        let commit_op =
            route_commit_operation(&*ctx.repo.get()?, &head_info, &mut out, &id_map, target_ish)?;
        let gate_target = commit_op.gate_target(&head_info)?;
        {
            let repo = ctx.repo.get()?;
            but_api::commit::create::gate::enforce_commit_gate_for_target(&repo, &gate_target)
                .map_err(commit_gate_cli_error)?;
        }
        (commit_op, id_map)
    };

    let guard = ctx.exclusive_worktree_access();
    let (mut guard, commit_selection) =
        resolve_commit_selection(guard, ctx, &mut out, changes, interactive, empty, &id_map)?;
    let mut meta = ctx.meta()?;
    run(
        ctx,
        &mut meta,
        guard.write_permission(),
        commit_op,
        commit_selection,
        reword_op,
    )
}

fn commit_operation_target_ish(
    branch: Option<Option<BranchArg>>,
    above: Option<CliIdArg>,
    below: Option<CliIdArg>,
) -> CliResult<CommitOperationTargetIsh> {
    match (branch, above, below) {
        (Some(Some(branch)), None, None) => Ok(CommitOperationTargetIsh::Branch(branch)),
        (Some(None), None, None) => Ok(CommitOperationTargetIsh::UnstackedCannedBranch),
        (None, Some(cli_id), None) => Ok(CommitOperationTargetIsh::Above(cli_id)),
        (None, None, Some(cli_id)) => Ok(CommitOperationTargetIsh::Below(cli_id)),
        (None, None, None) => Ok(CommitOperationTargetIsh::Default),
        _ => Err(anyhow::anyhow!(
            "BUG: Should not be able to supply more than one of above, below or branch"
        )
        .into()),
    }
}

fn reword_operation(no_message: bool, message: Option<Vec<String>>) -> RewordCommitOperation {
    match (no_message, message) {
        (true, None) => RewordCommitOperation::NoMessage,
        (false, None) => RewordCommitOperation::UseEditor,
        (false, Some(message)) => RewordCommitOperation::Message(message.join("\n\n")),
        (true, Some(_)) => {
            unreachable!("--no-message and --message are mutually exclusive")
        }
    }
}

fn resolve_commit_selection(
    guard: RepoExclusiveGuard,
    ctx: &mut Context,
    out: &mut IntermediateChannel<'_>,
    changes: Vec<CliIdArg>,
    interactive: bool,
    empty: bool,
    id_map: &IdMap,
) -> CliResult<(RepoExclusiveGuard, CommitSelection)> {
    let (guard, commit_selection) = if !changes.is_empty() {
        let changes = changes
            .into_iter()
            .map(|change| change.resolve_uncommitted(&*ctx.repo.get()?, id_map))
            .collect::<Result<Vec<_>, _>>()?;
        let Some(changes) = NonEmpty::from_vec(changes) else {
            return Err(bad_input("No changes to commit")
                .hint("Run `but status` to show applicable targets")
                .into());
        };
        (guard, CommitSelection::Changes(Box::new(changes)))
    } else if interactive {
        let (guard, outcome) = tui_with_options(ctx, guard, out, TuiRunOptions::PickChanges)?;
        let cli_ids = match outcome {
            TuiOutcome::CliIds(cli_ids) => cli_ids,
            TuiOutcome::None => {
                return Err(bad_input("No changes to commit")
                    .hint("Pick changes by pressing space. Confirm with enter.")
                    .into());
            }
        };
        let changes = cli_ids
            .into_iter()
            .map(|change| {
                match change {
                    CliId::Uncommitted(id) => Ok(id),
                    _ => {
                        Err(anyhow::anyhow!("BUG: tui should only return uncommitted changes in PickChanges mode but got {change:?}"))
                    }
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let Some(changes) = NonEmpty::from_vec(changes) else {
            return Err(bad_input("No changes to commit")
                .hint("Pick changes by pressing space. Confirm with enter.")
                .into());
        };
        (guard, CommitSelection::Changes(Box::new(changes)))
    } else if empty {
        (guard, CommitSelection::Nothing)
    } else {
        (guard, CommitSelection::AllChanges)
    };

    Ok((guard, commit_selection))
}

fn run(
    ctx: &mut Context,
    meta: &mut impl RefMetadata,
    perm: &mut RepoExclusive,
    commit_op: CommitOperation,
    commit_selection: CommitSelection,
    reword_op: RewordCommitOperation,
) -> CliResult<CommitOutcome> {
    let changes = {
        let context_lines = ctx.settings.context_lines;
        let (repo, ws, mut db) = ctx.workspace_and_db_mut_with_perm(perm.read_permission())?;
        let mut builder = DiffSpecBuilder::new(&mut db, &repo, &ws, context_lines);

        match commit_selection {
            CommitSelection::AllChanges => {
                builder.push_changes_from_unassigned(&UNASSIGNED.to_string())?;
            }
            CommitSelection::Changes(changes) => {
                for change in *changes {
                    builder.push_changes_from_uncommitted(&change)?;
                }
            }
            CommitSelection::Nothing => {}
        }

        builder.into_diff_specs()
    };
    let snapshot_details = SnapshotDetails::new(OperationKind::CreateCommit);
    let result = but_transaction::with_transaction_with_perm(
        ctx,
        meta,
        perm,
        snapshot_details,
        DryRun::No,
        |mut tx| {
            let (
                IntermediateCommitCreateResult {
                    new_commit,
                    rejected_specs,
                },
                branch_name,
            ) = commit_op.execute(&mut tx, changes)?;

            anyhow::ensure!(rejected_specs.is_empty(), "Couldn't commit all changes");

            let new_commit =
                new_commit.context("BUG: rejected_specs is empty yet nothing was committed")?;

            let reworded_commit = reword_op.execute(new_commit, &mut tx)?;

            Ok(DynamicOutcome::<_, std::convert::Infallible>::Commit((
                reworded_commit,
                branch_name,
            )))
        },
    );

    let DynamicOutcome::Commit(((new_commit, branch_name), _ws)) = match result {
        Ok(outcome) => outcome,
        Err(err) => {
            return Err(
                if let Some(Code::EditorExitedWithNonZeroStatus) =
                    err.downcast_ref::<but_error::Code>()
                {
                    bad_input("Editor exited with non-zero status").into()
                } else {
                    err.into()
                },
            );
        }
    };

    Ok(CommitOutcome {
        new_commit,
        branch_name,
    })
}

/// Targeting modes for committing.
enum CommitOperationTargetIsh {
    /// Target the branch if it exists, or create it at the newest base if it does not.
    Branch(BranchArg),
    /// Target newest base with a new canned branch name.
    UnstackedCannedBranch,
    /// Targets above the [`CliIdArg`], which must denote either a commit or a branch.
    Above(CliIdArg),
    /// Targets below the [`CliIdArg`], which must denote either a commit or a branch. For commits,
    /// this is directly below. For branches, this is below the segment.
    Below(CliIdArg),
    /// The default target, makes a sensible choice about where to put the commit, creating a branch
    /// if necessary. This should be used if there is no input from the user about where to put the
    /// commit.
    Default,
}

fn route_commit_operation(
    repo: &gix::Repository,
    head_info: &RefInfo,
    out: &mut IntermediateChannel<'_>,
    id_map: &IdMap,
    target: CommitOperationTargetIsh,
) -> CliResult<CommitOperation> {
    match target {
        CommitOperationTargetIsh::Above(cli_id) => {
            let position = CommitRelativeToTargetPosition::Above;
            route_commit_above_or_below(repo, head_info, id_map, cli_id, position)
        }
        CommitOperationTargetIsh::Below(cli_id) => {
            let position = CommitRelativeToTargetPosition::Below;
            route_commit_above_or_below(repo, head_info, id_map, cli_id, position)
        }
        CommitOperationTargetIsh::Branch(branch) => {
            if let Some(segment) = branch.try_resolve_segment(head_info)? {
                let ref_info = segment.ref_info.with_context(|| {
                    format!("BUG: Segment resolved from branch name {branch} has no ref info")
                })?;

                let target = CommitRelativeToTarget::BranchTip {
                    name: ref_info.ref_name,
                };

                Ok(CommitOperation::CommitAt(CommitAtOperation { target }))
            } else {
                let branch_name =
                    BranchArg(branch.resolve_for_creation(repo, head_info).with_hint(|| {
                        format!("Run `but apply {branch}` to apply the branch first")
                    })?)
                    .resolve_local_branch_name()?;
                Ok(CommitOperation::CommitToNewBranch(
                    CommitToNewBranchOperation {
                        branch_name: Some(branch_name),
                    },
                ))
            }
        }
        CommitOperationTargetIsh::UnstackedCannedBranch => Ok(CommitOperation::CommitToNewBranch(
            CommitToNewBranchOperation { branch_name: None },
        )),
        CommitOperationTargetIsh::Default => match &head_info.stacks[..] {
            [] => Ok(CommitOperation::CommitToNewBranch(
                CommitToNewBranchOperation { branch_name: None },
            )),
            [stack] => {
                let ref_info = stack
                    .segments
                    .first()
                    .and_then(|segment| segment.ref_info.as_ref())
                    .context("Head stack has no ref")?;
                Ok(CommitOperation::CommitAt(CommitAtOperation {
                    target: CommitRelativeToTarget::BranchTip {
                        name: ref_info.ref_name.clone(),
                    },
                }))
            }
            stacks => {
                let stack_heads = stacks
                    .iter()
                    .filter_map(|stack| stack.segments.first())
                    .filter_map(|segment| segment.ref_info.as_ref())
                    .map(|ref_info| (ref_info.ref_name.shorten(), &ref_info.ref_name))
                    .collect::<Vec<_>>();

                let Some(stack_heads) = NonEmpty::from_vec(stack_heads) else {
                    return Err(anyhow::anyhow!(
                        "BUG: found multiple stacks but none of them have heads"
                    )
                    .into());
                };

                let Some(mut input) = out.prepare_for_terminal_input() else {
                    return Err(
                        bad_input("Unclear where to commit. Found more than one stack")
                            .hint("You can specify where to commit with `--branch [<BRANCH>]`")
                            .into(),
                    );
                };

                let Some(selection) = input.prompt_select(
                    "Multiple stacks found. Choose one to commit to",
                    &stack_heads,
                )?
                else {
                    return Err(bad_input("No stack picked").into());
                };

                Ok(CommitOperation::CommitAt(CommitAtOperation {
                    target: CommitRelativeToTarget::BranchTip {
                        name: (*selection).clone(),
                    },
                }))
            }
        },
    }
}

fn route_commit_above_or_below(
    repo: &gix::Repository,
    head_info: &RefInfo,
    id_map: &IdMap,
    cli_id: CliIdArg,
    position: CommitRelativeToTargetPosition,
) -> CliResult<CommitOperation> {
    let target = match cli_id
        .resolve_in_workspace(repo, id_map, Purpose::Target, None)
        .hint(
            "Target must be an applied branch or commit. Run `but status` for applicable targets.",
        )?
        .into_branch_or_commit()
        .hint("Run `but status` to show applicable targets")?
    {
        BranchOrCommit::Commit(commit_id) => {
            let config_ref = find_branch_ref_for_commit(head_info, commit_id)?;
            CommitRelativeToTarget::Commit {
                commit_id,
                position,
                config_ref,
            }
        }
        BranchOrCommit::Branch(arg) => CommitRelativeToTarget::BranchBucket {
            name: arg.resolve_local_branch_name()?,
            position,
        },
    };
    Ok(CommitOperation::CommitAt(CommitAtOperation { target }))
}

fn find_branch_ref_for_commit(
    head_info: &RefInfo,
    commit_id: gix::ObjectId,
) -> CliResult<FullName> {
    head_info
        .stacks
        .iter()
        .flat_map(|stack| stack.segments.iter())
        .find_map(|segment| {
            let contains_commit = segment.commits.iter().any(|commit| commit.id == commit_id)
                || segment
                    .commits_on_remote
                    .iter()
                    .any(|commit| commit.id == commit_id);
            contains_commit.then(|| {
                segment
                    .ref_info
                    .as_ref()
                    .map(|ref_info| ref_info.ref_name.clone())
                    .or_else(|| segment.remote_tracking_ref_name.clone())
            })?
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "BUG: commit {} resolved from workspace without an owning branch ref",
                commit_id.to_hex_with_len(7)
            )
            .into()
        })
}

enum CommitSelection {
    AllChanges,
    Changes(Box<NonEmpty<UncommittedCliId>>),
    Nothing,
}

enum CommitOperation {
    CommitToNewBranch(CommitToNewBranchOperation),
    CommitAt(CommitAtOperation),
}

impl CommitOperation {
    fn gate_target(
        &self,
        head_info: &RefInfo,
    ) -> anyhow::Result<but_api::commit::create::gate::CommitGateTarget> {
        match self {
            CommitOperation::CommitToNewBranch(op) => op.gate_target(head_info),
            CommitOperation::CommitAt(op) => op.target.gate_target(),
        }
    }

    fn execute(
        self,
        tx: &mut Transaction<'_, '_, impl RefMetadata>,
        changes: Vec<DiffSpec>,
    ) -> anyhow::Result<(IntermediateCommitCreateResult, Option<BranchNameTarget>)> {
        match self {
            CommitOperation::CommitToNewBranch(op) => op.execute(tx, changes),
            CommitOperation::CommitAt(op) => op.execute(tx, changes),
        }
    }
}

struct CommitToNewBranchOperation {
    branch_name: Option<FullName>,
}

impl CommitToNewBranchOperation {
    fn gate_target(
        &self,
        head_info: &RefInfo,
    ) -> anyhow::Result<but_api::commit::create::gate::CommitGateTarget> {
        let config_ref = default_gate_config_ref(head_info)?;
        Ok(but_api::commit::create::gate::CommitGateTarget::config_only(config_ref))
    }

    fn execute(
        self,
        tx: &mut Transaction<'_, '_, impl RefMetadata>,
        changes: Vec<DiffSpec>,
    ) -> anyhow::Result<(IntermediateCommitCreateResult, Option<BranchNameTarget>)> {
        let Self { branch_name } = self;

        let branch_name = if let Some(branch_name) = branch_name {
            branch_name
        } else {
            but_core::branch::unique_canned_refname(tx.repo())?
        };

        tx.create_reference(branch_name.as_ref(), None, |_| StackId::generate(), None)?;

        let commit_create_result = tx.create_commit(
            RelativeTo::Reference(branch_name.clone()),
            InsertSide::Below,
            changes,
            String::new(),
        )?;

        Ok((
            commit_create_result,
            Some(BranchNameTarget::New(branch_name)),
        ))
    }
}

struct CommitAtOperation {
    target: CommitRelativeToTarget,
}

impl CommitAtOperation {
    fn execute(
        self,
        tx: &mut Transaction<'_, '_, impl RefMetadata>,
        changes: Vec<DiffSpec>,
    ) -> anyhow::Result<(IntermediateCommitCreateResult, Option<BranchNameTarget>)> {
        let (relative_to, side, branch_name_target) = match self.target {
            CommitRelativeToTarget::Commit {
                commit_id,
                position,
                config_ref: _,
            } => (RelativeTo::Commit(commit_id), position.into(), None),
            CommitRelativeToTarget::BranchBucket { name, position } => {
                let new_branch_name = but_core::branch::unique_canned_refname(tx.repo())?;
                let anchor = Anchor::at_segment(name.as_ref(), position.into());
                tx.create_reference(
                    new_branch_name.as_ref(),
                    Some(anchor),
                    |_| StackId::generate(),
                    Some(0),
                )?;

                (
                    RelativeTo::Reference(new_branch_name.clone()),
                    InsertSide::Below,
                    Some(BranchNameTarget::New(new_branch_name)),
                )
            }
            CommitRelativeToTarget::BranchTip { name } => (
                RelativeTo::Reference(name.clone()),
                InsertSide::Below,
                Some(BranchNameTarget::Existing(name)),
            ),
        };

        let commit_create_result =
            tx.create_commit(relative_to.clone(), side, changes, String::new())?;

        Ok((commit_create_result, branch_name_target))
    }
}

/// Place a commit relative to something in the workspace.
#[derive(Clone)]
enum CommitRelativeToTarget {
    /// Place the commit relative to this commit, within the same branch.
    Commit {
        commit_id: gix::ObjectId,
        position: CommitRelativeToTargetPosition,
        config_ref: FullName,
    },
    /// Place the commit at the tip of the branch denoted by this reference, moving the reference to
    /// the new commit. This is effectively the same as committing to a branch.
    BranchTip { name: FullName },
    /// Place the commit relative to this branch, treating the branch as a bucket.
    ///
    /// The commit is always inserted on a new branch with a canned name.
    BranchBucket {
        name: FullName,
        position: CommitRelativeToTargetPosition,
    },
}

impl CommitRelativeToTarget {
    fn gate_target(&self) -> anyhow::Result<but_api::commit::create::gate::CommitGateTarget> {
        match self {
            Self::Commit { config_ref, .. } => Ok(
                but_api::commit::create::gate::CommitGateTarget::config_only(config_ref.clone()),
            ),
            Self::BranchTip { name } => Ok(
                but_api::commit::create::gate::CommitGateTarget::direct_ref(name.clone()),
            ),
            Self::BranchBucket { name, .. } => {
                Ok(but_api::commit::create::gate::CommitGateTarget::config_only(name.clone()))
            }
        }
    }
}

fn default_gate_config_ref(head_info: &RefInfo) -> anyhow::Result<FullName> {
    if let Some(target_ref) = &head_info.target_ref {
        return Ok(target_ref.ref_name.clone());
    }

    head_info
        .stacks
        .iter()
        .find_map(|stack| stack.ref_name().cloned())
        .context("target ref is required to load governance config for commit authorization")
}

fn commit_gate_cli_error(err: anyhow::Error) -> CliError {
    if let Some(gate_error) = but_api::commit::create::gate::classify_error(&err) {
        return anyhow::anyhow!(
            "{}",
            serde_json::json!({
                "error": {
                    "code": gate_error.code,
                    "message": gate_error.message,
                }
            })
        )
        .into();
    }

    err.into()
}

#[cfg(test)]
mod commit_gate_tests {
    use super::*;

    #[test]
    fn commit_gate_target_uses_branch_tip_ref() -> anyhow::Result<()> {
        let main = FullName::try_from("refs/heads/main")?;
        let target = CommitRelativeToTarget::BranchTip { name: main.clone() }.gate_target()?;
        assert_eq!(
            target,
            but_api::commit::create::gate::CommitGateTarget::direct_ref(main),
            "CLI branch-tip commits must authorize against the direct target ref"
        );

        Ok(())
    }
}

#[derive(Clone)]
enum CommitRelativeToTargetPosition {
    Above,
    Below,
}

impl Display for CommitRelativeToTargetPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pretty = match self {
            Self::Above => "above",
            Self::Below => "below",
        };
        write!(f, "{pretty}")
    }
}

impl From<CommitRelativeToTargetPosition> for InsertSide {
    fn from(value: CommitRelativeToTargetPosition) -> Self {
        match value {
            CommitRelativeToTargetPosition::Above => Self::Above,
            CommitRelativeToTargetPosition::Below => Self::Below,
        }
    }
}

impl From<CommitRelativeToTargetPosition> for Position {
    fn from(value: CommitRelativeToTargetPosition) -> Self {
        match value {
            CommitRelativeToTargetPosition::Above => Self::Above,
            CommitRelativeToTargetPosition::Below => Self::Below,
        }
    }
}

enum RewordCommitOperation {
    NoMessage,
    Message(String),
    UseEditor,
}

impl RewordCommitOperation {
    fn execute(
        self,
        new_commit: gix::ObjectId,
        tx: &mut Transaction<'_, '_, impl RefMetadata>,
    ) -> anyhow::Result<gix::ObjectId> {
        let message = match self {
            RewordCommitOperation::NoMessage => String::new(),
            RewordCommitOperation::Message(message) => message,
            RewordCommitOperation::UseEditor => {
                let repo = tx.repo();
                let commit_details = CommitDetails::from_commit_id(
                    new_commit.attach(repo),
                    ComputeLineStats::No.into(),
                )?;

                let editor_initial_message = String::new();
                let current_message_for_comparison = "";
                get_commit_message_from_editor(
                    tx.repo(),
                    tx.context_lines(),
                    commit_details,
                    editor_initial_message,
                    current_message_for_comparison,
                    ShowDiffInEditor::Unspecified,
                )?
                .unwrap_or_default()
            }
        };

        let reworded_commit = tx.reword_commit(new_commit, BString::from(message).as_ref())?;

        Ok(reworded_commit)
    }
}
