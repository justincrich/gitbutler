//! Functional authorization primitives for governed GitButler actions.

pub mod menu;
pub mod route;

mod assignment_state;
mod authority;
mod authorize;
mod config;
mod denial;
mod principal;

pub use assignment_state::{AssignmentState, AssignmentStateParseError};
pub use authority::{Authority, AuthoritySet, ParseAuthorityError, serialize_authority_tokens};
pub use authorize::{
    authorize, effective_authority, resolve_principal, resolve_principal_from_env,
};
pub use config::{
    BranchName, BranchProtection, ConfigError, GovConfig, GroupWire, PermissionsWire,
    PrincipalWire, gates_path, governance_present, load_governance_config, load_permissions_wire,
    permissions_path,
};
pub use denial::{AuthorizedAction, Denial, DenialClass, to_envelope};
pub use menu::{
    AFFORDANCE_MAP, CATALOG, CatalogEntry, DenialCategory, DeniedRoute, authorized_actions,
};
pub use principal::{Group, GroupName, Principal, PrincipalId};
pub use route::{ROUTE_AUTHORITY_TABLE, ReviewAction, Route};

/// STEER-008: non-enforced agent-priming reference primer.
///
/// This is a `&'static str` constant carrying the L2 agent-priming reference
/// text. It is **non-enforced reference material** — no `but-authz` or
/// `but-api` engine path branches on, reads, or depends on it. The constant
/// exists so harnesses and tests can reference a single source of truth for
/// the primer text, and so a build-gate can prove the engine is independent
/// of it.
///
/// Contract encoded:
/// 1. `but` denials are redirects rather than terminal failures
///    (`actor_correctable` — the actor can recover).
/// 2. `authorized_actions` are authorized OPTIONS not orders — choose the
///    entry that serves your actual task (goal integrity).
/// 3. Bypass (raw git / `--no-verify`) is NEVER the route to a landed change.
/// 4. The `class` / `do_not` contract — stop on `operator_required`; never
///    bypass on `actor_correctable`.
pub const AGENT_PRIMER: &str = "\
# Governance denial agent primer (non-enforced reference material)

This primer is **non-enforced reference material** the harness MAY adopt. The
`but-authz` and `but-api` engine paths do NOT depend on it for correctness.

## Denials are redirects

`but` denials are redirects rather than terminal failures. A `perm.denied`
envelope tells the actor which authority was missing and which authorized
actions are still available. Treat `actor_correctable` denials as a recovery
prompt, not a stop sign.

## Authorized actions are options, not orders

`authorized_actions` lists the routes the caller already holds the authority
to run. They are OPTIONS, NOT ORDERS — choose the entry that serves your
actual task (goal integrity). Do not blindly pick the first item; the menu is
a closed catalog, not a queue.

## Bypass is never the route

Bypass (`git push --no-verify`, raw `git`, editing state directly) is NEVER
the route to a landed change. The governed path is the only route. bypass is
never faster, never cleaner, never safer.

## The class / do_not contract

Every denial carries a `class` and an implicit `do_not` contract:
- On `operator_required`: STOP and surface to the operator. Do not retry, do
  not improvise, do not roll a different governed path.
- On `actor_correctable`: the `do_not` contract is to use the surfaced
  `authorized_actions` to pick a real recovery route. Never bypass.
";
