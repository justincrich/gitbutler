use std::{env, ffi::OsString};

use crate::{Authority, AuthoritySet, Denial, DenialClass, GovConfig, Principal, PrincipalId};

const BUT_AGENT_HANDLE: &str = "BUT_AGENT_HANDLE";

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
    let principal_id = PrincipalId::new(handle.clone());
    let authorities = cfg
        .principal_authorities(&principal_id)
        .ok_or_else(|| Denial::unknown_principal(&handle))?
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
    /// ```
    /// use but_authz::{Authority, AuthoritySet, Denial};
    ///
    /// let denial = Denial::missing_permission(Authority::ContentsWrite, &AuthoritySet::empty());
    /// assert_eq!(denial.code, Denial::PERM_DENIED_CODE);
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

        Self::new(
            Self::PERM_DENIED_CODE,
            format!(
                "action requires {}; authorization denied ({held_summary})",
                missing.name()
            ),
            format!(
                "request a reviewed merge or ask a maintainer to grant {}",
                missing.name()
            ),
        )
    }

    /// Build a structured denial for an unset or empty agent handle.
    ///
    /// ```
    /// use but_authz::Denial;
    ///
    /// let denial = Denial::no_handle();
    /// assert_eq!(denial.code, Denial::PERM_DENIED_CODE);
    /// ```
    pub fn no_handle() -> Self {
        Self {
            // STEER-004: no handle → operator must provision the principal in
            // committed governance config before the actor can recover.
            class: DenialClass::OperatorRequired,
            ..Self::new(
                Self::PERM_DENIED_CODE,
                "BUT_AGENT_HANDLE is required to resolve a governed principal".to_owned(),
                "set BUT_AGENT_HANDLE to a principal committed in governance config".to_owned(),
            )
        }
    }

    /// Build a structured denial for a handle absent from committed config.
    ///
    /// ```
    /// use but_authz::Denial;
    ///
    /// let denial = Denial::unknown_principal("ghost");
    /// assert!(denial.message.contains("ghost"));
    /// ```
    pub fn unknown_principal(handle: &str) -> Self {
        Self {
            // STEER-004: unknown principal → operator must commit the
            // principal to governance config before the actor can recover.
            class: DenialClass::OperatorRequired,
            ..Self::new(
                Self::PERM_DENIED_CODE,
                format!("principal \"{handle}\" not found in committed governance config"),
                "commit the principal to governance config before running governed actions"
                    .to_owned(),
            )
        }
    }
}

impl std::fmt::Display for Denial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Denial {}
