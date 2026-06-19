/// Agent-readable authorization denial.
///
/// ```
/// use but_authz::Denial;
///
/// let denial = Denial {
///     code: Denial::PERM_DENIED_CODE,
///     message: "action requires contents:write".to_owned(),
///     remediation_hint: "ask an administrator to grant contents:write".to_owned(),
/// };
/// assert_eq!(denial.code, "perm.denied");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Denial {
    /// Stable denial code consumed by callers.
    pub code: &'static str,
    /// Human-readable denial message naming the missing authority.
    pub message: String,
    /// Actionable recovery hint for the denied actor.
    pub remediation_hint: String,
}

impl Denial {
    /// The stable permission-denied code for authorization failures.
    ///
    /// ```
    /// use but_authz::Denial;
    ///
    /// assert_eq!(Denial::PERM_DENIED_CODE, "perm.denied");
    /// ```
    pub const PERM_DENIED_CODE: &'static str = "perm.denied";
}
