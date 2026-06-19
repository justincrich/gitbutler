//! Functional authorization primitives for governed GitButler actions.

mod authority;
mod config;
mod denial;
mod principal;

pub use authority::{Authority, AuthoritySet, ParseAuthorityError};
pub use config::{BranchName, BranchProtection, ConfigError, GovConfig, load_governance_config};
pub use denial::Denial;
pub use principal::{Group, GroupName, Principal, PrincipalId};
