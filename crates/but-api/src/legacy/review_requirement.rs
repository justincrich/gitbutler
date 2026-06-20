use std::collections::BTreeSet;

use but_authz::{GovConfig, GroupName, PrincipalId};
use but_db::LocalReviewVerdict;

use super::ReviewRequirement;

const APPROVED: &str = "approved";

/// Stable reason emitted when no qualifying approval exists for a required slot.
pub(crate) const NO_APPROVAL: &str = "no_approval";

/// Stable reason emitted when a qualifying approval exists only at an older head.
pub(crate) const APPROVAL_STALE_AT_HEAD: &str = "approval_stale_at_head";

/// Stable reason emitted when approvals cannot map one-per-required-group.
pub(crate) const NO_DISTINCT_APPROVAL: &str = "no_distinct_approval";

/// Review requirement shortfall returned by the pure evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewUnmet {
    entries: Vec<String>,
}

impl ReviewUnmet {
    fn new(entries: Vec<String>) -> Self {
        Self { entries }
    }

    /// Convert unmet entries into the merge-gate payload representation.
    pub(crate) fn into_entries(self) -> Vec<String> {
        self.entries
    }
}

/// Evaluate a target-ref review requirement from supplied verdict rows.
pub(crate) fn evaluate(
    requirement: &ReviewRequirement,
    verdicts: &[LocalReviewVerdict],
    current_head_oid: &str,
    author: &PrincipalId,
    cfg: &GovConfig,
) -> Result<(), ReviewUnmet> {
    let approved = approved_verdicts(verdicts);
    let current_approvals = current_approvals(requirement, &approved, current_head_oid, author);
    let stale_approvals = stale_approvals(requirement, &approved, current_head_oid, author);

    let mut unmet = Vec::new();
    if current_approvals.len() < requirement.min_approvals {
        unmet.push(reason_for_shortfall(!stale_approvals.is_empty()).to_owned());
    }

    let satisfied_groups = distinctly_satisfied_groups(
        cfg,
        &requirement.require_approval_from_group,
        &current_approvals,
    );
    for group_name in &requirement.require_approval_from_group {
        if !satisfied_groups.iter().any(|group| group == group_name) {
            let reason = if has_group_approval(cfg, group_name, &current_approvals) {
                NO_DISTINCT_APPROVAL
            } else {
                reason_for_shortfall(has_group_approval(cfg, group_name, &stale_approvals))
            };
            unmet.push(format!(
                "require_approval_from_group {}: {reason}",
                group_name.as_str()
            ));
        }
    }

    if unmet.is_empty() {
        Ok(())
    } else {
        Err(ReviewUnmet::new(unmet))
    }
}

fn approved_verdicts(verdicts: &[LocalReviewVerdict]) -> Vec<&LocalReviewVerdict> {
    verdicts
        .iter()
        .filter(|verdict| verdict.verdict == APPROVED)
        .collect()
}

fn current_approvals(
    requirement: &ReviewRequirement,
    verdicts: &[&LocalReviewVerdict],
    current_head_oid: &str,
    author: &PrincipalId,
) -> BTreeSet<PrincipalId> {
    verdicts
        .iter()
        .filter(|verdict| verdict.head_oid == current_head_oid)
        .filter_map(|verdict| eligible_principal(requirement, verdict, author))
        .collect()
}

fn stale_approvals(
    requirement: &ReviewRequirement,
    verdicts: &[&LocalReviewVerdict],
    current_head_oid: &str,
    author: &PrincipalId,
) -> BTreeSet<PrincipalId> {
    verdicts
        .iter()
        .filter(|verdict| verdict.head_oid != current_head_oid)
        .filter_map(|verdict| eligible_principal(requirement, verdict, author))
        .collect()
}

fn eligible_principal(
    requirement: &ReviewRequirement,
    verdict: &LocalReviewVerdict,
    author: &PrincipalId,
) -> Option<PrincipalId> {
    let principal = PrincipalId::new(verdict.principal_id.clone());
    if requirement.require_distinct_from_author && principal == *author {
        None
    } else {
        Some(principal)
    }
}

fn has_group_approval(
    cfg: &GovConfig,
    group_name: &GroupName,
    approvals: &BTreeSet<PrincipalId>,
) -> bool {
    cfg.groups().get(group_name).is_some_and(|group| {
        approvals
            .iter()
            .any(|principal| group.members().iter().any(|member| member == principal))
    })
}

fn distinctly_satisfied_groups(
    cfg: &GovConfig,
    required_groups: &[GroupName],
    approvals: &BTreeSet<PrincipalId>,
) -> BTreeSet<GroupName> {
    let mut assignments = Vec::new();
    for group_name in required_groups {
        let mut visited = BTreeSet::new();
        assign_group(cfg, group_name, approvals, &mut assignments, &mut visited);
    }

    assignments
        .into_iter()
        .map(|assignment| assignment.group_name)
        .collect()
}

fn assign_group(
    cfg: &GovConfig,
    group_name: &GroupName,
    approvals: &BTreeSet<PrincipalId>,
    assignments: &mut Vec<GroupApprovalAssignment>,
    visited: &mut BTreeSet<PrincipalId>,
) -> bool {
    let Some(group) = cfg.groups().get(group_name) else {
        return false;
    };

    for principal in approvals
        .iter()
        .filter(|principal| group.members().iter().any(|member| member == *principal))
    {
        if !visited.insert(principal.clone()) {
            continue;
        }

        if reassign_principal(cfg, principal, group_name, approvals, assignments, visited) {
            return true;
        }
    }

    false
}

fn reassign_principal(
    cfg: &GovConfig,
    principal: &PrincipalId,
    group_name: &GroupName,
    approvals: &BTreeSet<PrincipalId>,
    assignments: &mut Vec<GroupApprovalAssignment>,
    visited: &mut BTreeSet<PrincipalId>,
) -> bool {
    let Some(existing) = assignments
        .iter()
        .position(|assignment| assignment.principal == *principal)
    else {
        assignments.push(GroupApprovalAssignment {
            group_name: group_name.clone(),
            principal: principal.clone(),
        });
        return true;
    };

    let previous_group_name = assignments.remove(existing).group_name;
    if assign_group(cfg, &previous_group_name, approvals, assignments, visited) {
        assignments.push(GroupApprovalAssignment {
            group_name: group_name.clone(),
            principal: principal.clone(),
        });
        true
    } else {
        assignments.push(GroupApprovalAssignment {
            group_name: previous_group_name,
            principal: principal.clone(),
        });
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GroupApprovalAssignment {
    group_name: GroupName,
    principal: PrincipalId,
}

fn reason_for_shortfall(has_stale_approval: bool) -> &'static str {
    if has_stale_approval {
        APPROVAL_STALE_AT_HEAD
    } else {
        NO_APPROVAL
    }
}
