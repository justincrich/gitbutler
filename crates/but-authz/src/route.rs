//! STEER-002 single-source route/authority table.
//!
//! [`ROUTE_AUTHORITY_TABLE`] is the single source of truth that maps every
//! gated [`Route`] to (a) the [`Authority`] its gate site requires, (b) the
//! consumer-facing command string, and (c) the effect string. Gate sites keep
//! their literal `but_authz::authorize(principal, Authority::X, &cfg)` calls —
//! the table does not replace them — but the table is the only place where the
//! route→authority mapping lives, so menu derivation (STEER-009) and CLI
//! serializers (STEER-005) can trust it without re-deriving from gate sites.
//!
//! Adding a new gated route without adding a row here fails
//! `steer_route_table_covers_every_gated_route`; assigning a row an authority
//! that disagrees with its route family fails the same test.

use crate::Authority;

/// A forge review sub-action governed by a distinct functional authority.
///
/// Each variant maps to a distinct [`Authority`] in [`ROUTE_AUTHORITY_TABLE`]:
/// `RequestChanges` → `ReviewsWrite`, `Comment` → `CommentsWrite`,
/// `Approve` → `PullRequestsWrite`. Splitting them lets the table distinguish
/// the three forge-dispatched review verbs without losing the shared
/// `Review(_)` family label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReviewAction {
    /// Request changes on a review — requires `reviews:write`.
    RequestChanges,
    /// Post a comment on a review — requires `comments:write`.
    Comment,
    /// Approve a review — requires `pull_requests:write`.
    Approve,
}

/// A gated route surface.
///
/// Variants correspond to the four gate-site families: the commit gate, the
/// merge gate, the forge review gates (split by [`ReviewAction`]), and the
/// administration-write gate. The variant carries no authority by itself —
/// the canonical route→authority mapping lives in
/// [`ROUTE_AUTHORITY_TABLE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Route {
    /// Land a commit directly — `crates/but-api/src/commit/gate.rs`.
    Commit,
    /// Land a reviewed merge — `crates/but-api/src/legacy/merge_gate.rs`.
    Merge,
    /// Forge review sub-action — `crates/but-api/src/legacy/forge.rs`.
    Review(ReviewAction),
    /// Mutate governed configuration —
    /// `crates/but-api/src/legacy/config_mutate.rs`.
    Admin,
}

/// The single-source route → (required authority, command, effect) table.
///
/// Every row MUST agree with the literal `authorize(principal, Authority::X,
/// &cfg)` call at its gate site. The table is read-only and lives in
/// `but-authz` so gate sites, menu derivation, and the CLI denial serializers
/// share one definition without `but-authz` depending on `but-api`.
///
/// Row order is the canonical catalog order of [`Route`]: Commit, Merge,
/// Review(RequestChanges), Review(Comment), Review(Approve), Admin.
pub const ROUTE_AUTHORITY_TABLE: &[(Route, Authority, &str, &str)] = &[
    // Commit gate — crates/but-api/src/commit/gate.rs:82.
    (
        Route::Commit,
        Authority::ContentsWrite,
        "commit",
        "create or amend a commit on a governed ref",
    ),
    // Merge gate — crates/but-api/src/legacy/merge_gate.rs:62.
    (
        Route::Merge,
        Authority::Merge,
        "merge",
        "land a reviewed merge into the target branch",
    ),
    // Forge review gate (request changes) —
    // crates/but-api/src/legacy/forge.rs::request_changes_review.
    (
        Route::Review(ReviewAction::RequestChanges),
        Authority::ReviewsWrite,
        "request-changes",
        "request changes on a local review",
    ),
    // Forge review gate (comment) —
    // crates/but-api/src/legacy/forge.rs::comment_review / post_comment.
    (
        Route::Review(ReviewAction::Comment),
        Authority::CommentsWrite,
        "comment",
        "post or resolve a local review comment",
    ),
    // Forge review gate (approve) —
    // crates/but-api/src/legacy/forge.rs::approve_review.
    (
        Route::Review(ReviewAction::Approve),
        Authority::PullRequestsWrite,
        "approve",
        "record an approval verdict on a local review",
    ),
    // Admin-write gate — crates/but-api/src/legacy/config_mutate.rs:47.
    (
        Route::Admin,
        Authority::AdministrationWrite,
        "admin",
        "mutate governed configuration on the target ref",
    ),
];
