//! The typed boundary enum for the `local_review_assignments.state` TEXT column.
//!
//! `AssignmentState` mirrors the shipped `Authority::parse`/`::name` round-trip
//! (see `crates/but-authz/src/authority.rs`): a fieldless `Copy` enum whose
//! `name()` returns the exact literal the column stores and whose `parse()` is
//! total — every unknown string is an `Err`, never a silent default. The DB
//! column stays `TEXT` (migration-tolerant, matching `LocalReviewVerdict.verdict`);
//! this enum is the boundary concern that validates the literal on read/write.
//!
//! `AssignmentState` is a DRIVE-state enum — it tells an orchestrator what to do
//! next (`pending` → dispatch reviewer; `changes_requested` → remediation;
//! `approved` → eligible). It is NOT an `Authority`: it never enters a merge /
//! commit / authorize path (the safe seam — gates read `local_review_verdicts`,
//! not assignment state).

/// The lifecycle state of a local review assignment.
///
/// ```
/// use but_authz::AssignmentState;
///
/// assert_eq!(AssignmentState::Approved.name(), "approved");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssignmentState {
    /// Reviewer has been assigned but has not yet acted.
    Pending,
    /// Reviewer approved the change.
    Approved,
    /// Reviewer requested changes.
    ChangesRequested,
}

impl AssignmentState {
    /// Every drive state in deterministic order.
    ///
    /// ```
    /// use but_authz::AssignmentState;
    ///
    /// assert!(AssignmentState::ALL.contains(&AssignmentState::Approved));
    /// ```
    pub const ALL: &'static [Self] = &[Self::Pending, Self::Approved, Self::ChangesRequested];

    /// Return the stable literal this state serializes to.
    ///
    /// These are the EXACT values the `local_review_assignments.state` TEXT
    /// column stores and that `approve_review` / `request_changes_review` write.
    ///
    /// ```
    /// use but_authz::AssignmentState;
    ///
    /// assert_eq!(AssignmentState::ChangesRequested.name(), "changes_requested");
    /// ```
    pub fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::ChangesRequested => "changes_requested",
        }
    }

    /// Parse a drive-state literal.
    ///
    /// Total and fail-closed: an unknown / garbage / wrong-case string returns
    /// `Err`, never a default variant (mirroring `Authority::parse`).
    ///
    /// ```
    /// use but_authz::AssignmentState;
    ///
    /// assert_eq!(AssignmentState::parse("approved"), Ok(AssignmentState::Approved));
    /// assert!(AssignmentState::parse("merged").is_err());
    /// ```
    pub fn parse(token: &str) -> Result<Self, AssignmentStateParseError> {
        match token {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "changes_requested" => Ok(Self::ChangesRequested),
            unknown => Err(AssignmentStateParseError::UnknownToken(unknown.to_owned())),
        }
    }
}

impl std::fmt::Display for AssignmentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// A typed parse failure for `AssignmentState` literals.
///
/// ```
/// use but_authz::{AssignmentState, AssignmentStateParseError};
///
/// assert!(matches!(
///     AssignmentState::parse("merged"),
///     Err(AssignmentStateParseError::UnknownToken(_))
/// ));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssignmentStateParseError {
    /// The token is not one of the three drive-state literals.
    #[error("unknown assignment state token: {0}")]
    UnknownToken(String),
}

impl AssignmentStateParseError {
    /// Return the rejected token.
    ///
    /// ```
    /// use but_authz::{AssignmentState, AssignmentStateParseError};
    ///
    /// let error = AssignmentState::parse("merged").err();
    /// assert_eq!(error.as_ref().map(AssignmentStateParseError::token), Some("merged"));
    /// ```
    pub fn token(&self) -> &str {
        match self {
            Self::UnknownToken(token) => token,
        }
    }
}
