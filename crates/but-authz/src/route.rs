//! Route enum + `ROUTE_AUTHORITY_TABLE` — the single source of truth for
//! governed routes.
//!
//! Every gated route is a row in [`ROUTE_AUTHORITY_TABLE`], mapping a
//! [`Route`] to its required [`Authority`], the literal `but` command that
//! drives it, and a one-line effect description. The table is the single
//! symbol consulted by:
//!
//! - every gate's required-authority lookup (`but_api::commit::gate`,
//!   `but_api::legacy::merge_gate`, `but_api::legacy::config_mutate`, and
//!   `but_api::legacy::forge::authorize_branch_action`), and
//! - the menu module (STEER-003) for the
//!   `usable = {r | r.required_authority ⊆ held}` derivation.
//!
//! ## What the table is NOT
//!
//! The table SUPPLIES the (route → required [`Authority`], `but` command,
//! effect) data. It does NOT replace the [`crate::authorize`] call that
//! makes the deny/allow decision — the literal
//! `but_authz::authorize(&principal, Authority::X, &cfg)` call stays at
//! each enforcement site so the `AUTHORITY_POSITIVE_PATTERN` honesty grep
//! (`but_authz::authorize|Authority::contains|but_authz::Authority`) keeps
//! matching.
//!
//! The non-authority predicates (branch-protection, review-requirement)
//! stay composed AROUND the table; they are NOT folded into it. Folding
//! them would make `required_authority ⊆ held` a lying menu (the C5
//! unsoundness STEER-003 must avoid).

use crate::Authority;

/// A governed route — one row per gate that calls [`crate::authorize`].
///
/// Each variant maps 1:1 to a row in [`ROUTE_AUTHORITY_TABLE`] carrying its
/// required [`Authority`], the literal `but` command that drives the route,
/// and a one-line effect description. Non-authority predicates
/// (branch-protection, review-requirement) stay composed AROUND the table;
/// they are NOT folded into it.
///
/// ```
/// use but_authz::{Authority, Route};
///
/// assert_eq!(Route::Commit.required_authority(), Authority::ContentsWrite);
/// assert_eq!(Route::Merge.required_authority(), Authority::Merge);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Route {
    /// Direct commit creation — gated by
    /// `but_api::commit::gate::enforce_commit_gate`. The
    /// branch-protection predicate is composed AROUND this row, not folded
    /// into it.
    Commit,
    /// Forge review merge — gated by
    /// `but_api::legacy::merge_gate::enforce_merge_gate`. The
    /// review-requirement predicate is composed AROUND this row, not folded
    /// into it.
    Merge,
    /// Forge review assignment / verdict (`reviews:write`) — gated by
    /// `but_api::legacy::forge::authorize_branch_action`. Reconciles the
    /// `Authority::ReviewsWrite` arm of the forge match into an explicit
    /// table row.
    ForgeReviewsWrite,
    /// Forge review comment (`comments:write`) — gated by
    /// `but_api::legacy::forge::authorize_branch_action`. Reconciles the
    /// `Authority::CommentsWrite` arm of the forge match into an explicit
    /// table row.
    ForgeCommentsWrite,
    /// Forge review open / close / publish (`pull_requests:write`) — gated
    /// by `but_api::legacy::forge::authorize_branch_action`. Reconciles the
    /// `Authority::PullRequestsWrite` arm of the forge match into an
    /// explicit table row.
    ForgePullRequestsWrite,
    /// Administration-write — gated by
    /// `but_api::legacy::config_mutate::enforce_administration_write_gate`.
    Admin,
}

impl Route {
    /// Every governed route in deterministic catalog order.
    ///
    /// ```
    /// use but_authz::Route;
    ///
    /// assert!(Route::ALL.contains(&Route::Commit));
    /// assert!(Route::ALL.contains(&Route::Merge));
    /// assert!(Route::ALL.contains(&Route::Admin));
    /// ```
    pub const ALL: &'static [Self] = &[
        Self::Commit,
        Self::Merge,
        Self::ForgeReviewsWrite,
        Self::ForgeCommentsWrite,
        Self::ForgePullRequestsWrite,
        Self::Admin,
    ];

    /// Return the stable token name for this route.
    ///
    /// The token is unique across variants and stable across refactorings —
    /// the menu module (STEER-003) indexes routes by this string.
    ///
    /// ```
    /// use but_authz::Route;
    ///
    /// assert_eq!(Route::Commit.name(), "commit");
    /// assert_eq!(Route::Merge.name(), "merge");
    /// assert_eq!(Route::Admin.name(), "admin");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Merge => "merge",
            Self::ForgeReviewsWrite => "forge.reviews:write",
            Self::ForgeCommentsWrite => "forge.comments:write",
            Self::ForgePullRequestsWrite => "forge.pull_requests:write",
            Self::Admin => "admin",
        }
    }

    /// Look up the required [`Authority`] for this route through
    /// [`ROUTE_AUTHORITY_TABLE`].
    ///
    /// This is the single source of truth consulted by every gate's
    /// required-authority lookup AND by the menu module (STEER-003). The
    /// lookup is total — every [`Route`] variant resolves to a row in the
    /// table; if a future variant is added without a row, this method will
    /// panic at the gate site, surfacing the omission loudly.
    ///
    /// The gate sites still call [`crate::authorize`] with the looked-up
    /// [`Authority`]; the table does NOT replace the decision call.
    ///
    /// ```
    /// use but_authz::{Authority, Route};
    ///
    /// assert_eq!(
    ///     Route::ForgeReviewsWrite.required_authority(),
    ///     Authority::ReviewsWrite
    /// );
    /// ```
    pub fn required_authority(self) -> Authority {
        // Mirrors `Authority::ALL` static-slice-of-enum pattern from
        // `authority.rs`; the linear scan over a 6-row table is the
        // idiomatic lookup shape for a `const` slice.
        for (route, authority, _, _) in ROUTE_AUTHORITY_TABLE {
            if *route == self {
                return *authority;
            }
        }
        // Every Route variant must have a row — this is the totality
        // invariant STEER-003's menu derivation depends on.
        unreachable!(
            "ROUTE_AUTHORITY_TABLE must cover every Route variant; missing row for {:?}",
            self
        )
    }
}

/// The single source of truth mapping every governed [`Route`] to its
/// required [`Authority`], the literal `but` command that drives it, and a
/// one-line effect description.
///
/// Each row is `(Route, Authority, command, effect)`. Every row corresponds
/// to a gate that calls [`crate::authorize`]:
///
/// - `Route::Commit` → `Authority::ContentsWrite` — `but_api::commit::gate`
/// - `Route::Merge` → `Authority::Merge` — `but_api::legacy::merge_gate`
/// - `Route::ForgeReviewsWrite` → `Authority::ReviewsWrite` —
///   `but_api::legacy::forge::authorize_branch_action`
/// - `Route::ForgeCommentsWrite` → `Authority::CommentsWrite` —
///   `but_api::legacy::forge::authorize_branch_action`
/// - `Route::ForgePullRequestsWrite` → `Authority::PullRequestsWrite` —
///   `but_api::legacy::forge::authorize_branch_action`
/// - `Route::Admin` → `Authority::AdministrationWrite` —
///   `but_api::legacy::config_mutate`
///
/// The non-authority predicates (branch-protection, review-requirement)
/// stay composed AROUND the table — they are NOT folded into it.
///
/// ```
/// use but_authz::{Authority, Route, ROUTE_AUTHORITY_TABLE};
///
/// assert!(ROUTE_AUTHORITY_TABLE.len() >= 6);
/// assert!(ROUTE_AUTHORITY_TABLE
///     .iter()
///     .any(|(route, authority, _, _)| *route == Route::Merge
///         && *authority == Authority::Merge));
/// ```
pub const ROUTE_AUTHORITY_TABLE: &[(Route, Authority, &str, &str)] = &[
    (
        Route::Commit,
        Authority::ContentsWrite,
        "but commit",
        "create a commit on a branch ref",
    ),
    (
        Route::Merge,
        Authority::Merge,
        "but review merge",
        "merge a forge review into the target branch",
    ),
    (
        Route::ForgeReviewsWrite,
        Authority::ReviewsWrite,
        "but review approve / request-changes / assign",
        "post or update a review verdict on a branch",
    ),
    (
        Route::ForgeCommentsWrite,
        Authority::CommentsWrite,
        "but review comment / resolve",
        "post or resolve a review comment on a branch",
    ),
    (
        Route::ForgePullRequestsWrite,
        Authority::PullRequestsWrite,
        "but review request / close / publish",
        "open, close, or publish a local forge review",
    ),
    (
        Route::Admin,
        Authority::AdministrationWrite,
        "but governance config write",
        "mutate committed governance config",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_all_matches_table_rows() {
        assert_eq!(Route::ALL.len(), ROUTE_AUTHORITY_TABLE.len());
        for route in Route::ALL {
            assert!(
                ROUTE_AUTHORITY_TABLE.iter().any(|(r, _, _, _)| r == route),
                "Route::{route:?} must have a row in ROUTE_AUTHORITY_TABLE"
            );
        }
    }
}
