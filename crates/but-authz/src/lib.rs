//! Functional authorization primitives for governed GitButler actions.

mod authority;
mod authorize;
mod config;
mod denial;
mod principal;

pub use authority::{Authority, AuthoritySet, ParseAuthorityError};
pub use authorize::{
    authorize, effective_authority, resolve_principal, resolve_principal_from_env,
};
pub use config::{
    BranchName, BranchProtection, ConfigError, GovConfig, GroupWire, PermissionsWire,
    PrincipalWire, gates_path, governance_present, load_governance_config, permissions_path,
};
pub use denial::Denial;
pub use principal::{Group, GroupName, Principal, PrincipalId};
