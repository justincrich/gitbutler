//! STEER-006: `but whoami` and `but can-i` self-scoped discovery commands.
//!
//! Both commands disclose only the caller's own data (principal handle, own
//! authorities, own group memberships). Group rosters stay gated by
//! `administration:read` from Sprint 05 — `whoami` lists group NAMES the caller
//! belongs to, never the members of those groups.
//!
//! Both commands run without a git repo: the resolved handle is printed with
//! empty authorities/groups and `not_configured: true` when no repo (or no
//! committed governance config) is available. These are discovery commands, not
//! repo-scoped operations.

use std::env;

use anyhow::anyhow;
use but_authz::{
    Authority, Denial, PrincipalId, ROUTE_AUTHORITY_TABLE, governance_present,
    load_governance_config,
};
use but_ctx::Context;
use serde::Serialize;

use crate::{CliError, utils::OutputChannel};

/// JSON output shape for `but whoami`.
///
/// Field names are snake_case to match the existing `GovernanceStatus`
/// carrier consumed by the desktop renderer (`governance_status_read`).
#[derive(Debug, Clone, Serialize)]
pub struct WhoamiOutcome {
    /// Resolved principal handle (from `BUT_AGENT_HANDLE`).
    pub principal: String,
    /// Effective functional authority tokens (sorted lexically).
    pub authorities: Vec<String>,
    /// Group names the caller belongs to (their own memberships only).
    pub groups: Vec<String>,
    /// CLI command tokens from ROUTE_AUTHORITY_TABLE whose required authority
    /// the caller holds, in table order (e.g. `["commit", "merge"]`).
    pub authorized_actions: Vec<String>,
    /// True when no committed governance config could be loaded (no repo, no
    /// project, or no governance blob at the target ref). A normal state — the
    /// handle is still disclosed so callers can self-identify.
    pub not_configured: bool,
}

/// JSON output shape for `but can-i <authority>`.
#[derive(Debug, Clone, Serialize)]
pub struct CanIOutcome {
    /// Whether the caller is authorized for the requested action.
    pub authorized: bool,
    /// The action token that was checked (omitted on unresolved-principal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    /// The principal handle the check ran against (omitted on unresolved-principal).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<String>,
    /// Reason for an `authorized: false` result that isn't a clean denial.
    /// `unresolved_principal` — caller's principal couldn't be resolved.
    /// `not_configured` — no committed governance config to check against.
    /// `unknown_authority` — the supplied action token is not a known authority.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<&'static str>,
}

/// `but whoami` — print the caller's self-scoped principal info as JSON.
///
/// Returns `perm.denied` (exit 1) when the principal can't be resolved from
/// `BUT_AGENT_HANDLE`. Returns success (exit 0) with `not_configured: true`
/// when there's no repo or no committed governance config to load authorities
/// from — the handle alone is still useful to callers.
pub async fn exec_whoami(
    ctx: Option<&mut Context>,
    out: &mut OutputChannel,
) -> Result<(), CliError> {
    let handle = resolve_handle_or_denial()?;

    let outcome = match ctx {
        Some(ctx) => match load_principal_outcome(ctx, &handle).await {
            Ok(outcome) => outcome,
            // Graceful degradation: any failure loading the config (no project,
            // missing target ref, no committed blob, parse error) collapses to
            // `not_configured: true` with the handle still disclosed. This is
            // self-scoped discovery, not enforcement — we don't fail closed.
            Err(_) => not_configured_outcome(&handle),
        },
        None => not_configured_outcome(&handle),
    };

    write_whoami(out, &outcome)
}

/// `but can-i <authority>` — print whether the caller holds the authority.
///
/// Always exits 0. Returns `{"authorized": false, "reason": "unresolved_principal"}`
/// when `BUT_AGENT_HANDLE` is unset, so callers can pipe the result into a
/// query without distinguishing missing-principal from denial.
pub async fn exec_can_i(
    ctx: Option<&mut Context>,
    out: &mut OutputChannel,
    authority_token: String,
) -> Result<(), CliError> {
    let Some(handle) = env::var_os("BUT_AGENT_HANDLE").filter(|v| !v.is_empty()) else {
        return write_can_i(
            out,
            &CanIOutcome {
                authorized: false,
                action: None,
                principal: None,
                reason: Some(REASON_UNRESOLVED_PRINCIPAL),
            },
        );
    };
    let handle = handle.to_string_lossy().into_owned();

    let action = match Authority::parse(&authority_token) {
        Ok(action) => action,
        Err(_) => {
            return write_can_i(
                out,
                &CanIOutcome {
                    authorized: false,
                    action: Some(authority_token),
                    principal: Some(handle),
                    reason: Some(REASON_UNKNOWN_AUTHORITY),
                },
            );
        }
    };

    let outcome = match ctx {
        Some(ctx) => match load_principal_authorities(ctx, &handle).await {
            Ok((held, _groups, not_configured)) => {
                if not_configured {
                    CanIOutcome {
                        authorized: false,
                        action: Some(authority_token),
                        principal: Some(handle),
                        reason: Some(REASON_NOT_CONFIGURED),
                    }
                } else {
                    CanIOutcome {
                        authorized: held.contains(action),
                        action: Some(authority_token),
                        principal: Some(handle),
                        reason: None,
                    }
                }
            }
            Err(_) => CanIOutcome {
                authorized: false,
                action: Some(authority_token),
                principal: Some(handle),
                reason: Some(REASON_NOT_CONFIGURED),
            },
        },
        None => CanIOutcome {
            authorized: false,
            action: Some(authority_token),
            principal: Some(handle),
            reason: Some(REASON_NOT_CONFIGURED),
        },
    };

    write_can_i(out, &outcome)
}

/// Reason token used when `BUT_AGENT_HANDLE` is unset/empty.
const REASON_UNRESOLVED_PRINCIPAL: &str = "unresolved_principal";
/// Reason token used when no committed governance config could be loaded.
const REASON_NOT_CONFIGURED: &str = "not_configured";
/// Reason token used when the action argument isn't a known authority token.
const REASON_UNKNOWN_AUTHORITY: &str = "unknown_authority";

/// Pull `BUT_AGENT_HANDLE` from the process env, returning a structured
/// `perm.denied` `CliError` when unset/empty. `whoami` treats an unresolved
/// principal as a hard error (exit 1) so consumers can distinguish "I have no
/// identity" from "I have an identity but no permissions".
fn resolve_handle_or_denial() -> Result<String, CliError> {
    let Some(handle) = env::var_os("BUT_AGENT_HANDLE").filter(|v| !v.is_empty()) else {
        return Err(governance_denial_error(Denial::no_handle()));
    };
    Ok(handle.to_string_lossy().into_owned())
}

/// Build a `not_configured: true` outcome for a resolved handle.
fn not_configured_outcome(handle: &str) -> WhoamiOutcome {
    WhoamiOutcome {
        principal: handle.to_owned(),
        authorities: Vec::new(),
        groups: Vec::new(),
        authorized_actions: Vec::new(),
        not_configured: true,
    }
}

/// Try to load the caller's full principal info from the repo's committed
/// governance config. Falls back to `not_configured` on any failure.
async fn load_principal_outcome(ctx: &mut Context, handle: &str) -> anyhow::Result<WhoamiOutcome> {
    let (held, groups, not_configured) = load_principal_authorities(ctx, handle).await?;
    if not_configured {
        return Ok(not_configured_outcome(handle));
    }

    let mut authorities = held
        .iter()
        .map(|authority| authority.name().to_owned())
        .collect::<Vec<_>>();
    authorities.sort();

    let groups = groups
        .into_iter()
        .map(|group| group.as_str().to_owned())
        .collect::<Vec<_>>();

    let authorized_actions = authorized_action_commands(&held);

    Ok(WhoamiOutcome {
        principal: handle.to_owned(),
        authorities,
        groups,
        authorized_actions,
        not_configured: false,
    })
}

/// Load the caller's effective authority set + group memberships from the
/// committed governance config. Returns `(held, groups, not_configured)`:
/// `not_configured: true` means no committed governance blob was found at the
/// target ref (a normal state). Any other failure (no project, parse error)
/// propagates as `Err` so the caller can decide how to degrade.
async fn load_principal_authorities(
    ctx: &mut Context,
    handle: &str,
) -> anyhow::Result<(but_authz::AuthoritySet, Vec<but_authz::GroupName>, bool)> {
    let target_ref = resolve_target_ref(ctx)?;
    let repo = ctx.repo.get()?;

    if !governance_present(&repo, &target_ref)? {
        return Ok((but_authz::AuthoritySet::empty(), Vec::new(), true));
    }

    let config = load_governance_config(&repo, &target_ref)?;
    let principal_id = PrincipalId::new(handle.to_owned());
    let Some(held) = config.principal_authorities(&principal_id).cloned() else {
        // Handle is set but unknown to committed config. For `whoami`, the
        // caller treats this as `not_configured`-style graceful degradation
        // (caller has an identity but it's not provisioned here). For `can-i`,
        // we propagate as `not_authorized`. Either way: empty held set.
        return Ok((but_authz::AuthoritySet::empty(), Vec::new(), false));
    };

    let groups = config
        .groups()
        .values()
        .filter(|group| {
            group
                .members()
                .iter()
                .any(|member| member.as_str() == handle)
        })
        .map(|group| group.name().clone())
        .collect::<Vec<_>>();

    Ok((held, groups, false))
}

/// Resolve the workspace target ref the same way `but perm` and `but group`
/// do: read the base-branch data, then fall back through candidate ref spellings
/// until one exists in the repo. This is intentionally a local copy of the
/// helpers in `command::perm` / `command::group` rather than a shared helper —
/// a refactor extracting the duplicate is a separate, behavior-neutral change.
fn resolve_target_ref(ctx: &mut Context) -> anyhow::Result<String> {
    let target = {
        let mut guard = ctx.exclusive_worktree_access();
        let perm = guard.write_permission();
        but_api::legacy::virtual_branches::get_base_branch_data(ctx, perm)?
    };
    let Some(target) = target else {
        return Err(anyhow!("target ref is required to load governance config"));
    };
    let repo = ctx.repo.get()?;
    for candidate in target_ref_candidates(&target) {
        if repo.try_find_reference(&candidate)?.is_some() {
            return Ok(candidate);
        }
    }
    Ok(target.branch_name)
}

/// Candidate ref spellings to try, in priority order. Mirrors `command::perm`.
fn target_ref_candidates(target: &gitbutler_branch_actions::BaseBranch) -> Vec<String> {
    let mut candidates = Vec::new();
    if !target.short_name.is_empty() {
        candidates.push(format!("refs/heads/{}", target.short_name));
    }
    candidates.push(target.branch_name.clone());
    if !target.branch_name.starts_with("refs/") {
        candidates.push(format!("refs/remotes/{}", target.branch_name));
        candidates.push(format!("refs/heads/{}", target.branch_name));
    }
    candidates
}

/// Derive the ordered list of ROUTE_AUTHORITY_TABLE command tokens whose
/// required authority the caller holds. Single source of truth: the table.
fn authorized_action_commands(held: &but_authz::AuthoritySet) -> Vec<String> {
    ROUTE_AUTHORITY_TABLE
        .iter()
        .filter(|(_, required, _, _)| held.contains(*required))
        .map(|(_, _, command, _)| (*command).to_owned())
        .collect()
}

/// Write the whoami outcome to the output channel in the appropriate format.
fn write_whoami(out: &mut OutputChannel, outcome: &WhoamiOutcome) -> Result<(), CliError> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(out, "principal: {}", outcome.principal)?;
        if outcome.not_configured {
            writeln!(
                out,
                "status: not configured (no committed governance config)"
            )?;
        }
        if outcome.authorities.is_empty() {
            writeln!(out, "permissions: (none)")?;
        } else {
            writeln!(out, "permissions:")?;
            for authority in &outcome.authorities {
                writeln!(out, "  {authority}")?;
            }
        }
        if outcome.groups.is_empty() {
            writeln!(out, "groups: (none)")?;
        } else {
            writeln!(out, "groups:")?;
            for group in &outcome.groups {
                writeln!(out, "  {group}")?;
            }
        }
        if outcome.authorized_actions.is_empty() {
            writeln!(out, "authorized actions: (none)")?;
        } else {
            writeln!(out, "authorized actions:")?;
            for action in &outcome.authorized_actions {
                writeln!(out, "  {action}")?;
            }
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(outcome)?;
    }
    Ok(())
}

/// Write the can-i outcome to the output channel.
fn write_can_i(out: &mut OutputChannel, outcome: &CanIOutcome) -> Result<(), CliError> {
    if let Some(out) = out.for_human_or_shell() {
        let verdict = if outcome.authorized { "yes" } else { "no" };
        match (&outcome.action, &outcome.principal, &outcome.reason) {
            (Some(action), Some(principal), _) => {
                writeln!(out, "{verdict}: {principal} may {action}")?;
            }
            (None, None, Some(reason)) => {
                writeln!(out, "{verdict}: {reason}")?;
            }
            _ => {
                writeln!(out, "{verdict}")?;
            }
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(outcome)?;
    }
    Ok(())
}

/// Wrap a [`Denial`] as a structured CLI error matching the existing
/// governance denial envelope shape (STEER-005 carriers).
fn governance_denial_error(denial: Denial) -> CliError {
    let envelope = serde_json::json!({
        "code": denial.code,
        "message": denial.message,
        "remediation_hint": denial.remediation_hint,
    });
    anyhow!("{}", serde_json::json!({ "error": envelope })).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authorized_actions_filters_by_held_set() {
        let held = but_authz::AuthoritySet::parse(["contents:write", "merge"]).unwrap();
        let actions = authorized_action_commands(&held);
        assert_eq!(actions, vec!["commit", "merge"]);
    }

    #[test]
    fn authorized_actions_empty_when_nothing_held() {
        let held = but_authz::AuthoritySet::empty();
        let actions = authorized_action_commands(&held);
        assert!(actions.is_empty());
    }

    #[test]
    fn authorized_actions_full_catalog_for_admin() {
        let held = but_authz::AuthoritySet::from_role("admin").unwrap();
        let actions = authorized_action_commands(&held);
        assert_eq!(
            actions,
            vec![
                "commit",
                "merge",
                "request-changes",
                "comment",
                "approve",
                "admin"
            ]
        );
    }
}
