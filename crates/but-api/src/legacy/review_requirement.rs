use std::collections::BTreeSet;

use but_authz::{GovConfig, GroupName, PrincipalId};
use but_db::LocalReviewVerdict;

use super::ReviewRequirement;

const APPROVED: &str = "approved";

/// Stable reason emitted when no qualifying approval exists for a required slot.
pub(crate) const NO_APPROVAL: &str = "no_approval";

/// Stable reason emitted when a qualifying approval exists only at an older head.
pub(crate) const APPROVAL_STALE_AT_HEAD: &str = "approval_stale_at_head";

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

/// Evaluate a target-ref review requirement against supplied verdict rows.
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

    for group_name in &requirement.require_approval_from_group {
        if !has_group_approval(cfg, group_name, &current_approvals) {
            let reason =
                reason_for_shortfall(has_group_approval(cfg, group_name, &stale_approvals));
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
            .any(|principal| group.members().contains(principal))
    })
}

fn reason_for_shortfall(has_stale_approval: bool) -> &'static str {
    if has_stale_approval {
        APPROVAL_STALE_AT_HEAD
    } else {
        NO_APPROVAL
    }
}
