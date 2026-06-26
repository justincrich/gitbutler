//! Functional authorization primitives for governed GitButler actions.
//!
//! ```
//! assert_eq!(but_authz::agents_path(), ".gitbutler/agents.toml");
//!
//! let registry = but_authz::Registry::empty();
//! let resolver: fn(
//!     Option<&but_authz::Registry>,
//!     &but_authz::GovConfig,
//! ) -> Result<but_authz::Principal, but_authz::Denial> =
//!     but_authz::resolve_principal_with_registry;
//! # let _ = (registry, resolver);
//! ```

mod assignment_state;
mod authority;
mod authorize;
mod config;
mod denial;
mod menu;
mod principal;
mod process;
mod registry;
mod route;

pub use assignment_state::{AssignmentState, AssignmentStateParseError};
pub use authority::{Authority, AuthoritySet, ParseAuthorityError, serialize_authority_tokens};
pub use authorize::{
    DenialCause, authorize, effective_authority, resolve_principal, resolve_principal_from_env,
    resolve_principal_with_registry,
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
pub use principal::{Group, GroupName, Principal, PrincipalId};
pub use process::{current_pid, process_start_time};
pub use registry::{AgentId, ProcessKey, Registration, Registry};
pub use route::{ROUTE_AUTHORITY_TABLE, Route};
