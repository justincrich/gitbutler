//! Functional authorization primitives for governed GitButler actions.

mod authority;
mod denial;
mod principal;

pub use authority::{Authority, AuthoritySet, ParseAuthorityError};
pub use denial::Denial;
pub use principal::{Group, GroupName, Principal, PrincipalId};
