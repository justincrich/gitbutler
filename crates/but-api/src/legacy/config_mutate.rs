use but_authz::{
    Authority, AuthorizedAction, Denial, DenialClass, DenialPredicate, DeniedRoute, Route,
    load_governance_config, serialize_authority_tokens,
};
use serde::Serialize;

use crate::commit::create::gate::resolve_principal_with_runtime_registry;

/// Structured administration-write gate error payload for API callers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdminWriteGateError {
    /// Stable consumer-facing error code.
    pub code: &'static str,
    /// Human-readable denial message.
    pub message: String,
    /// Steering classification — who can recover. Copied off the underlying
    /// [`but_authz::Denial`] (or defaulted to `ActorCorrectable` for
    /// `ConfigError`-sourced carriers) so the governance CLI serializer
    /// (STEER-005) has a real field source.
    pub class: DenialClass,
    /// Authority tokens the principal already holds. Serialized as a
    /// stably-sorted array of `:`-token strings via
    /// [`but_authz::serialize_authority_tokens`]. Copied off the
    /// underlying [`but_authz::Denial`].
    #[serde(serialize_with = "serialize_authority_tokens")]
    pub held_permissions: Vec<Authority>,
    /// Recovery verbs the consumer may offer the actor. Copied off the
    /// underlying [`but_authz::Denial`].
    pub authorized_actions: Vec<AuthorizedAction>,
    /// Optional "do not" hint — verbs the actor must NOT attempt. Omitted
    /// entirely when `None` (no `null` key). Copied off the underlying
    /// [`but_authz::Denial`] / [`but_authz::ConfigError`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub do_not: Option<&'static str>,
}

/// Enforce authorization before mutating governed configuration.
///
/// The guard reads governance files from the committed `target_ref`, resolves the
/// acting principal from the runtime registry, and requires `administration:write`.
pub fn enforce_administration_write_gate(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<()> {
    let cfg = load_governance_config(repo, target_ref)?;
    let principal = resolve_principal_with_runtime_registry(repo, &cfg)?;

    // STEER-002: Route::Admin row in ROUTE_AUTHORITY_TABLE supplies the
    // required Authority for this gate; the literal `but_authz::authorize`
    // call is preserved so the AUTHORITY_POSITIVE_PATTERN honesty grep
    // keeps matching.
    let required = but_authz::Route::Admin.required_authority();
    // STEER-004: enrich the authorize denial with a route-scoped menu
    // (ActorCorrectable path), mirroring the commit and merge gates. The
    // deny/allow decision is unchanged. Without this, an admin-write denial
    // would carry an empty `authorized_actions` (no `but perm list` discovery
    // affordance) — the red-hat post-complete review flagged this gap.
    but_authz::authorize(&principal, required, &cfg).map_err(|denial| {
        denial.with_authorized_actions(
            &principal,
            &DeniedRoute::new(Route::Admin, DenialPredicate::Authority),
            &cfg,
        )
    })?;

    Ok(())
}

/// Extract a structured administration-write gate payload from an error chain.
///
/// Copies the four steering fields (`class`, `held_permissions`,
/// `authorized_actions`, `do_not`) off the underlying [`Denial`] (all four)
/// or [`but_authz::ConfigError`] (`class` + `do_not` only) so the governance
/// CLI serializer (STEER-005) has a real field source rather than a two-key
/// `{code,message}` flatten.
pub fn classify_error(err: &anyhow::Error) -> Option<AdminWriteGateError> {
    if let Some(denial) = err.downcast_ref::<Denial>() {
        return Some(AdminWriteGateError {
            code: denial.code,
            message: denial.message.clone(),
            class: denial.class,
            held_permissions: denial.held_permissions.clone(),
            authorized_actions: denial.authorized_actions.clone(),
            do_not: denial.do_not,
        });
    }

    err.downcast_ref::<but_authz::ConfigError>()
        .map(|error| AdminWriteGateError {
            code: error.code(),
            message: error.to_string(),
            // Fail loudly rather than silently defaulting to ActorCorrectable
            // if a future ConfigError constructor forgets to set `class`.
            // `ConfigError::invalid()` always sets `Some(OperatorRequired)`.
            class: error
                .class
                .expect("ConfigError must carry class (set via ConfigError::invalid())"),
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: error.do_not,
        })
}
