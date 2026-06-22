//! Functional authorization primitives for governed GitButler actions.

mod assignment_state;
mod authority;
mod authorize;
mod config;
mod denial;
mod menu;
mod principal;
mod route;

pub use assignment_state::{AssignmentState, AssignmentStateParseError};
pub use authority::{Authority, AuthoritySet, ParseAuthorityError, serialize_authority_tokens};
pub use authorize::{
    DenialCause, authorize, effective_authority, resolve_principal, resolve_principal_from_env,
};
pub use config::{
    BranchName, BranchProtection, ConfigError, GovConfig, GroupWire, PermissionsWire,
    PrincipalWire, governance_present, load_governance_config, load_permissions_wire,
    permissions_path,
};
pub use denial::{AuthorizedAction, Denial, DenialClass, to_envelope};
pub use menu::{
    AFFORDANCE_MAP, Affordance, CATALOG, DenialPredicate, DeniedRoute, authorized_actions,
};
pub use principal::{Group, GroupName, Principal, PrincipalId};
pub use route::{ROUTE_AUTHORITY_TABLE, Route};
