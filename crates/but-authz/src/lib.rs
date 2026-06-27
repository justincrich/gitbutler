//! Functional authorization primitives for governed GitButler actions.
//!
//! Identity resolution is environment-primary: governed gates resolve the acting
//! principal from the `BUT_AGENT_HANDLE` environment variable against the
//! committed `.gitbutler/agents.toml`. The handle is set by the trusted harness
//! wrapper (the git→but steerer), not self-asserted by the agent — see
//! `crates/but-authz/README.md` for the trust model.
//!
//! ```
//! assert_eq!(but_authz::agents_path(), ".gitbutler/agents.toml");
//!
//! let resolver: fn(
//!     &but_authz::GovConfig,
//! ) -> Result<but_authz::Principal, but_authz::Denial> =
//!     but_authz::resolve_principal_from_env;
//! # let _ = resolver;
//! ```

mod assignment_state;
mod authority;
mod authorize;
mod config;
mod denial;
mod menu;
mod migrate;
mod principal;
mod route;

pub use assignment_state::{AssignmentState, AssignmentStateParseError};
pub use authority::{Authority, AuthoritySet, ParseAuthorityError, serialize_authority_tokens};
pub use authorize::{
    DenialCause, authorize, effective_authority, resolve_principal, resolve_principal_from_env,
};
pub use config::agents_path;
pub use config::{
    BranchName, BranchProtection, ConfigError, GovConfig, GroupWire, PermissionsWire,
    PrincipalWire, governance_present, load_governance_config, load_permissions_wire,
    permissions_path,
};
pub use denial::{AuthorizedAction, Denial, DenialClass, steer_envelope_from_parts, to_envelope};
pub use menu::{
    AFFORDANCE_MAP, Affordance, CATALOG, DISCOVERY_COMMAND, DenialPredicate, DeniedRoute,
    authorized_actions,
};
pub use migrate::rewrite_principals_to_agents;
pub use principal::{Group, GroupName, Principal, PrincipalId};
pub use route::{ROUTE_AUTHORITY_TABLE, Route};
