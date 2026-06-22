//! Functional authorization primitives for governed GitButler actions.

mod assignment_state;
mod authority;
mod authorize;
mod config;
mod denial;
mod principal;

pub use assignment_state::{AssignmentState, AssignmentStateParseError};
pub use authority::{Authority, AuthoritySet, ParseAuthorityError};
pub use authorize::{
    authorize, effective_authority, resolve_principal, resolve_principal_from_env,
};
pub use config::{
    BranchName, BranchProtection, ConfigError, GovConfig, GroupWire, PermissionsWire,
    PrincipalWire, governance_present, load_governance_config, load_permissions_wire,
    permissions_path,
};
pub use denial::Denial;
pub use principal::{Group, GroupName, Principal, PrincipalId};
