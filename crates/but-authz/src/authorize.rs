use std::{
    env,
    ffi::{OsStr, OsString},
};

use crate::{
    Authority, AuthoritySet, Denial, DenialClass, DeniedRoute, GovConfig, Principal, PrincipalId,
    Registry, authorized_actions,
    process::{current_pid, process_start_time},
};

const BUT_AGENT_HANDLE: &str = "BUT_AGENT_HANDLE";
const BUT_AUTHZ_ALLOW_ENV_HANDLE: &str = "BUT_AUTHZ_ALLOW_ENV_HANDLE";

/// Do-not-retry hint for unresolved-principal denials (no-handle /
/// unknown-principal). These resolve NO principal, so the caller cannot
/// self-correct in-system — an empty menu + do-not-retry is correct, not
/// `actor_correctable` (security HIGH #2).
const DO_NOT_UNRESOLVED_PRINCIPAL: &str =
    "register the principal / set BUT_AGENT_HANDLE; do not retry as-is";

// ---------------------------------------------------------------------------
// DenialCause — the classification input for determining DenialClass
// ---------------------------------------------------------------------------

/// The classification input for determining [`DenialClass`].
///
/// This enum is matched EXHAUSTIVELY (NO `_ =>` arm) by [`DenialCause::class`]
/// to determine who can recover from a denial. Adding or removing a variant
/// without updating the match is a NON-EXHAUSTIVE-MATCH COMPILE ERROR — this
/// IS the security property: a new cause can never silently default to
/// `ActorCorrectable`.
///
/// # Variant → class mapping
///
/// | Variant | Class | Carriers |
/// |---|---|---|
/// | [`MissingAuthorityResolved`] | `ActorCorrectable` | `perm.denied` (principal resolved) |
/// | [`BranchProtected`] | `ActorCorrectable` | `branch.protected` |
/// | [`ReviewRequired`] | `ActorCorrectable` | `gate.review_required` |
/// | [`UnresolvedPrincipal`] | `OperatorRequired` | `perm.denied` (no-handle/unknown-principal) |
/// | [`ConfigInvalid`] | `OperatorRequired` | `config.invalid` |
///
/// [`MissingAuthorityResolved`]: `Self::MissingAuthorityResolved`
/// [`BranchProtected`]: `Self::BranchProtected`
/// [`ReviewRequired`]: `Self::ReviewRequired`
/// [`UnresolvedPrincipal`]: `Self::UnresolvedPrincipal`
/// [`ConfigInvalid`]: `Self::ConfigInvalid`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialCause {
    /// A resolved principal lacks the required authority (`perm.denied` with
    /// a resolved principal). The actor can self-correct by requesting a
    /// reviewed merge or using a different verb.
    MissingAuthorityResolved,
    /// The target ref is branch-protected (`branch.protected`). The actor
    /// holds the route's authority but a composed branch-protection predicate
    /// denied the action. The actor can self-correct by committing to a
    /// feature branch and opening a reviewed merge.
    BranchProtected,
    /// The review requirement is unmet (`gate.review_required`). The actor
    /// holds the merge authority but the review-requirement predicate denied
    /// the merge. The actor can self-correct by collecting approvals.
    ReviewRequired,
    /// The acting principal could not be resolved from trusted identity state
    /// (no-handle / unknown-principal / unregistered / stale registration).
    /// These carry the `perm.denied` code but resolve NO principal, so the
    /// caller cannot self-correct in-system — an operator must register the
    /// principal/process or set `BUT_AGENT_HANDLE` where explicitly allowed.
    UnresolvedPrincipal,
    /// The committed `.gitbutler` governance config is malformed, incomplete,
    /// or unreadable (`config.invalid`). An operator must fix the committed
    /// config; the actor cannot self-correct.
    ConfigInvalid,
}

impl DenialCause {
    /// Map this cause to a [`DenialClass`] via an EXHAUSTIVE, NON-DEFAULTED
    /// `match`.
    ///
    /// There is NO `_ =>` wildcard arm — adding or removing a
    /// [`DenialCause`] variant without updating this match is a
    /// non-exhaustive-match COMPILE ERROR. This IS the security property:
    /// a new cause can never silently default to `ActorCorrectable`.
    ///
    /// ```
    /// use but_authz::{DenialCause, DenialClass};
    ///
    /// assert_eq!(
    ///     DenialCause::MissingAuthorityResolved.class(),
    ///     DenialClass::ActorCorrectable
    /// );
    /// assert_eq!(
    ///     DenialCause::ConfigInvalid.class(),
    ///     DenialClass::OperatorRequired
    /// );
    /// ```
    pub fn class(self) -> DenialClass {
        match self {
            Self::MissingAuthorityResolved | Self::BranchProtected | Self::ReviewRequired => {
                DenialClass::ActorCorrectable
            }
            Self::UnresolvedPrincipal | Self::ConfigInvalid => DenialClass::OperatorRequired,
        }
    }
}

/// Authorize a principal for a functional governance action.
///
/// ```
/// # use but_authz::{Authority, AuthoritySet, GovConfig, Principal, PrincipalId};
/// # let principal = Principal::new(
/// #     PrincipalId::new("dev"),
/// #     AuthoritySet::parse(["contents:write"])?,
/// #     std::iter::empty(),
/// # );
/// # let config = GovConfig::new(
/// #     [(PrincipalId::new("dev"), AuthoritySet::parse(["contents:write"])? )],
/// #     [],
/// #     [],
/// # );
/// but_authz::authorize(&principal, Authority::ContentsWrite, &config)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn authorize(principal: &Principal, action: Authority, cfg: &GovConfig) -> Result<(), Denial> {
    let held = effective_authority(principal, cfg);
    if held.contains(action) {
        Ok(())
    } else {
        Err(Denial::missing_permission(action, &held))
    }
}

/// Return the principal's effective authority set for a committed config.
///
/// ```
/// # use but_authz::{Authority, AuthoritySet, GovConfig, Principal, PrincipalId};
/// # let principal = Principal::new(
/// #     PrincipalId::new("dev"),
/// #     AuthoritySet::parse(["contents:write"])?,
/// #     std::iter::empty(),
/// # );
/// # let config = GovConfig::new(
/// #     [(PrincipalId::new("dev"), AuthoritySet::parse(["contents:write"])? )],
/// #     [],
/// #     [],
/// # );
/// let held = but_authz::effective_authority(&principal, &config);
/// assert!(held.contains(Authority::ContentsWrite));
/// # Ok::<(), but_authz::ParseAuthorityError>(())
/// ```
pub fn effective_authority(principal: &Principal, cfg: &GovConfig) -> AuthoritySet {
    // `config::normalize_permissions` folds direct grants and both group
    // membership directions at load time, so this is equal to
    // `GovConfig::principal_authorities` by construction.
    cfg.principal_authorities(principal.id())
        .cloned()
        .unwrap_or_else(AuthoritySet::empty)
}

/// Resolve the acting principal from an injected environment lookup.
///
/// ```
/// # let config = but_authz::GovConfig::new([], [], []);
/// let denial = but_authz::resolve_principal(|_| None, &config).err();
/// assert_eq!(denial.map(|denial| denial.code), Some("perm.denied"));
/// ```
pub fn resolve_principal(
    lookup: impl Fn(&str) -> Option<OsString>,
    cfg: &GovConfig,
) -> Result<Principal, Denial> {
    let Some(handle) = lookup(BUT_AGENT_HANDLE).filter(|value| !value.is_empty()) else {
        return Err(Denial::no_handle());
    };

    let handle = handle.to_string_lossy().into_owned();
    principal_from_handle(&handle, cfg)
}

/// Resolve the acting principal through the process registry, optionally
/// falling back to the legacy environment handle when explicitly enabled.
///
/// Resolution order is strict:
///
/// 1. registry hit for the current `(pid, start_time)` -> registered principal;
/// 2. registry miss + `BUT_AUTHZ_ALLOW_ENV_HANDLE=1` -> environment fallback
///    through [`resolve_principal`];
/// 3. else -> [`Denial::unregistered`] for an unregistered process. A
///    same-pid registration with a different start time is still denied as
///    stale and never resolves a principal.
///
/// `reg=None` is treated as a registry miss.
pub fn resolve_principal_with_registry(
    reg: Option<&Registry>,
    cfg: &GovConfig,
) -> Result<Principal, Denial> {
    let pid = current_pid();
    resolve_principal_with_registry_lookup(
        reg,
        pid,
        process_start_time(pid),
        |key| env::var_os(key),
        cfg,
    )
}

fn resolve_principal_with_registry_lookup(
    reg: Option<&Registry>,
    pid: u32,
    observed_start_time: anyhow::Result<u64>,
    lookup: impl Fn(&str) -> Option<OsString>,
    cfg: &GovConfig,
) -> Result<Principal, Denial> {
    match observed_start_time {
        Ok(observed_start_time) => {
            if let Some(agent_id) =
                reg.and_then(|registry| registry.resolve((pid, observed_start_time)))
            {
                return principal_from_handle(agent_id.as_str(), cfg);
            }

            if allow_env_handle(&lookup) {
                return resolve_principal(lookup, cfg);
            }

            if let Some(registered_start_time) =
                registered_start_time_for_pid(reg, pid, observed_start_time)
            {
                return Err(Denial::stale_registration(
                    pid,
                    observed_start_time,
                    registered_start_time,
                ));
            }

            Err(Denial::unregistered(pid, observed_start_time))
        }
        Err(error) => {
            if allow_env_handle(&lookup) {
                return resolve_principal(lookup, cfg);
            }

            Err(Denial::unregistered_start_time_unresolved(pid, &error))
        }
    }
}

fn allow_env_handle(lookup: &impl Fn(&str) -> Option<OsString>) -> bool {
    lookup(BUT_AUTHZ_ALLOW_ENV_HANDLE).as_deref() == Some(OsStr::new("1"))
}

fn registered_start_time_for_pid(
    reg: Option<&Registry>,
    pid: u32,
    observed_start_time: u64,
) -> Option<u64> {
    reg?.registrations()
        .keys()
        .find_map(|(registered_pid, registered_start_time)| {
            (*registered_pid == pid && *registered_start_time != observed_start_time)
                .then_some(*registered_start_time)
        })
}

fn principal_from_handle(handle: &str, cfg: &GovConfig) -> Result<Principal, Denial> {
    let principal_id = PrincipalId::new(handle);
    let authorities = cfg
        .principal_authorities(&principal_id)
        .ok_or_else(|| Denial::unknown_principal(handle))?
        .clone();
    let groups = cfg
        .groups()
        .values()
        .filter(|group| group.members().contains(&principal_id))
        .map(|group| group.name().clone())
        .collect::<Vec<_>>();

    Ok(Principal::new(principal_id, authorities, groups))
}

/// Resolve the acting principal from the process environment.
///
/// This is a thin wrapper around [`resolve_principal`]; tests should use the
/// injected lookup variant to avoid mutating process environment.
///
/// ```
/// # let config = but_authz::GovConfig::new([], [], []);
/// let _ = but_authz::resolve_principal_from_env(&config);
/// ```
pub fn resolve_principal_from_env(cfg: &GovConfig) -> Result<Principal, Denial> {
    resolve_principal(|key| env::var_os(key), cfg)
}

impl Denial {
    /// Build a structured denial for a missing functional permission.
    ///
    /// Populates `class=ActorCorrectable` (via
    /// [`DenialCause::MissingAuthorityResolved`]), `held_permissions` from
    /// the held set, and a positive remediation hint. The
    /// `authorized_actions` menu is left empty here; the gate enriches it
    /// with a route-scoped [`crate::authorized_actions`] derivation after
    /// `authorize()` returns the denial.
    ///
    /// ```
    /// use but_authz::{Authority, AuthoritySet, Denial, DenialClass};
    ///
    /// let denial = Denial::missing_permission(
    ///     Authority::ContentsWrite,
    ///     &AuthoritySet::parse(["contents:read"]).unwrap(),
    /// );
    /// assert_eq!(denial.code, Denial::PERM_DENIED_CODE);
    /// assert_eq!(denial.class, DenialClass::ActorCorrectable);
    /// ```
    pub fn missing_permission(missing: Authority, held: &AuthoritySet) -> Self {
        let held_summary = if held.is_empty() {
            "no permissions".to_owned()
        } else {
            let names = held
                .iter()
                .map(Authority::name)
                .collect::<Vec<_>>()
                .join(", ");
            format!("held permissions: {names}")
        };

        Self {
            code: Self::PERM_DENIED_CODE,
            message: format!(
                "action requires {}; authorization denied ({held_summary})",
                missing.name()
            ),
            remediation_hint: format!(
                "request a reviewed merge or ask a maintainer to grant {}",
                missing.name()
            ),
            class: DenialCause::MissingAuthorityResolved.class(),
            held_permissions: held.iter().collect(),
            authorized_actions: Vec::new(),
            do_not: None,
        }
    }

    /// Build a structured denial for an unset or empty agent handle.
    ///
    /// Sets `class=OperatorRequired` (the caller cannot self-correct
    /// in-system), empty `held_permissions`/`authorized_actions`, and a
    /// do-not-retry `do_not`.
    ///
    /// ```
    /// use but_authz::{Denial, DenialClass};
    ///
    /// let denial = Denial::no_handle();
    /// assert_eq!(denial.code, Denial::PERM_DENIED_CODE);
    /// assert_eq!(denial.class, DenialClass::OperatorRequired);
    /// ```
    pub fn no_handle() -> Self {
        Self {
            code: Self::PERM_DENIED_CODE,
            message: "BUT_AGENT_HANDLE is required to resolve a governed principal".to_owned(),
            remediation_hint: "set BUT_AGENT_HANDLE to a principal committed in governance config"
                .to_owned(),
            class: DenialCause::UnresolvedPrincipal.class(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: Some(DO_NOT_UNRESOLVED_PRINCIPAL),
        }
    }

    /// Build a structured denial for a handle absent from committed config.
    ///
    /// Sets `class=OperatorRequired` (the caller cannot self-correct
    /// in-system), empty `held_permissions`/`authorized_actions`, and a
    /// do-not-retry `do_not`.
    ///
    /// ```
    /// use but_authz::{Denial, DenialClass};
    ///
    /// let denial = Denial::unknown_principal("ghost");
    /// assert_eq!(denial.class, DenialClass::OperatorRequired);
    /// assert!(denial.message.contains("ghost"));
    /// ```
    pub fn unknown_principal(handle: &str) -> Self {
        Self {
            code: Self::PERM_DENIED_CODE,
            message: format!("principal \"{handle}\" not found in committed governance config"),
            remediation_hint:
                "commit the principal to governance config before running governed actions"
                    .to_owned(),
            class: DenialCause::UnresolvedPrincipal.class(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: Some(DO_NOT_UNRESOLVED_PRINCIPAL),
        }
    }

    /// Build a structured denial for a process that has no current registry
    /// entry for its `(pid, start_time)`.
    ///
    /// Sets `class=OperatorRequired`, empty `held_permissions` /
    /// `authorized_actions`, and a do-not-retry hint because the process
    /// identity must be registered by an operator before governed actions can
    /// proceed.
    pub fn unregistered(pid: u32, start_time: u64) -> Self {
        Self {
            code: Self::PERM_DENIED_CODE,
            message: format!(
                "unregistered process pid {pid} start_time {start_time}; no governed principal resolved"
            ),
            remediation_hint:
                "create or refresh the process registration before running governed actions"
                    .to_owned(),
            class: DenialCause::UnresolvedPrincipal.class(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: Some(DO_NOT_UNRESOLVED_PRINCIPAL),
        }
    }

    /// Build a structured denial for a same-pid registry entry whose start
    /// time no longer matches the observed process start time.
    ///
    /// This fails closed against pid reuse: a stale registration must not
    /// authenticate the current process.
    pub fn stale_registration(
        pid: u32,
        observed_start_time: u64,
        registered_start_time: u64,
    ) -> Self {
        Self {
            code: Self::PERM_DENIED_CODE,
            message: format!(
                "stale process registration for pid {pid}: observed start_time {observed_start_time} differs from registered start_time {registered_start_time}; refusing possible pid reuse"
            ),
            remediation_hint: "refresh the process registration before running governed actions"
                .to_owned(),
            class: DenialCause::UnresolvedPrincipal.class(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: Some(DO_NOT_UNRESOLVED_PRINCIPAL),
        }
    }

    fn unregistered_start_time_unresolved(pid: u32, error: &anyhow::Error) -> Self {
        Self {
            code: Self::PERM_DENIED_CODE,
            message: format!(
                "unregistered process pid {pid}; process start time could not be resolved: {error}"
            ),
            remediation_hint:
                "create or refresh the process registration after process start time is available"
                    .to_owned(),
            class: DenialCause::UnresolvedPrincipal.class(),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: Some(DO_NOT_UNRESOLVED_PRINCIPAL),
        }
    }

    /// Enrich this denial with a route-scoped menu of authorized recovery
    /// actions.
    ///
    /// Gates call this after [`authorize`] returns a `missing_permission`
    /// denial to populate the `authorized_actions` field with the
    /// STEER-003 gate-state-aware derivation. The `held_permissions` set
    /// is also re-derived from the cfg so the menu and held set are always
    /// consistent with the same cfg the gate loaded.
    ///
    /// This method is behavior-neutral for the deny/allow decision — it
    /// derives FROM the denial, never changes it.
    pub fn with_authorized_actions(
        mut self,
        principal: &Principal,
        denied: &DeniedRoute,
        cfg: &GovConfig,
    ) -> Self {
        self.held_permissions = effective_authority(principal, cfg).iter().collect();
        self.authorized_actions = authorized_actions(principal, denied, cfg);
        self
    }
}

impl std::fmt::Display for Denial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Denial {}
