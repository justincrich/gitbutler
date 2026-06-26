use serde::ser::SerializeStruct as _;
use serde::{Serialize, Serializer};

use crate::Authority;

/// Steering classification describing who can correct a denial.
///
/// Every denial carrier exposes this as the `class` field so a steering
/// consumer can route the denial to the actor (`ActorCorrectable` — the
/// principal that issued the denied verb can recover on their own) or to
/// an operator (`OperatorRequired` — recovery needs a maintainer/admin).
///
/// Serializes to its snake_case name token via [`DenialClass::name`] so the
/// wire shape stays stable across variant renames.
///
/// ```
/// use but_authz::DenialClass;
///
/// assert_eq!(DenialClass::ActorCorrectable.name(), "actor_correctable");
/// assert_eq!(
///     serde_json::to_string(&DenialClass::OperatorRequired).unwrap(),
///     "\"operator_required\""
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DenialClass {
    /// The acting principal can self-recover (request a review, grant a
    /// missing permission they hold, switch to an authorized verb, ...).
    #[default]
    ActorCorrectable,
    /// Recovery requires a maintainer/operator (the actor has no path to
    /// satisfy this denial on their own — e.g. a merge requires approvals
    /// the actor cannot self-approve).
    OperatorRequired,
}

impl DenialClass {
    /// Return the stable wire token for this class.
    pub fn name(self) -> &'static str {
        match self {
            Self::ActorCorrectable => "actor_correctable",
            Self::OperatorRequired => "operator_required",
        }
    }
}

impl Serialize for DenialClass {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

/// A recovery action a steering consumer can offer the denied actor.
///
/// Carried alongside every denial so the consumer (CLI menu, agent priming,
/// telemetry) can surface a concrete recovery verb without re-deriving it
/// from the denial code. The `command` is the canonical CLI/agent verb and
/// `effect` is a short human-readable description of what running it does.
///
/// Serializes as `{ "command": ..., "effect": ... }` so consumers can read
/// the pair off any carrier's `authorized_actions` array.
///
/// ```
/// use but_authz::AuthorizedAction;
///
/// let action = AuthorizedAction::new("but review request", "request a review on the branch");
/// assert_eq!(action.command, "but review request");
/// assert_eq!(
///     serde_json::to_string(&action).unwrap(),
///     "{\"command\":\"but review request\",\"effect\":\"request a review on the branch\"}"
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorizedAction {
    /// Canonical CLI/agent recovery verb (e.g. `but review request`).
    pub command: &'static str,
    /// Short human-readable description of the recovery effect.
    pub effect: &'static str,
}

impl AuthorizedAction {
    /// Create a recovery action from a static verb + effect description.
    pub const fn new(command: &'static str, effect: &'static str) -> Self {
        Self { command, effect }
    }
}

impl Serialize for AuthorizedAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("AuthorizedAction", 2)?;
        state.serialize_field("command", self.command)?;
        state.serialize_field("effect", self.effect)?;
        state.end()
    }
}

/// Agent-readable authorization denial.
///
/// The legacy `code`/`message`/`remediation_hint` triple is preserved
/// verbatim for back-compat. The four steering fields (`class`,
/// `held_permissions`, `authorized_actions`, `do_not`) are always present
/// — STEER-004 wires their values per denial site; this carrier only
/// proves the shape compiles, serializes, and round-trips.
///
/// Construct with [`Denial::new`] to default the steering fields to an
/// empty `ActorCorrectable` shape, or build the full struct literal when
/// populating every field.
///
/// ```
/// use but_authz::Denial;
///
/// let denial = Denial::new(
///     Denial::PERM_DENIED_CODE,
///     "action requires contents:write".to_owned(),
///     "ask an administrator to grant contents:write".to_owned(),
/// );
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
    /// Steering classification — who can recover.
    pub class: DenialClass,
    /// Authority tokens the principal already holds (sorted lexically on
    /// serialization via [`crate::serialize_authority_tokens`]).
    pub held_permissions: Vec<Authority>,
    /// Recovery verbs the consumer may offer the actor.
    pub authorized_actions: Vec<AuthorizedAction>,
    /// Optional "do not" hint — verbs the actor must NOT attempt. Serialized
    /// only when `Some`; `None` omits the key entirely.
    pub do_not: Option<&'static str>,
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

    /// Construct a denial with the legacy triple populated and the steering
    /// fields defaulted to an empty `ActorCorrectable` shape.
    ///
    /// STEER-004 populates the steering fields per denial site; this
    /// constructor exists so existing call sites compile without
    /// repeating four empty defaults each time.
    pub fn new(code: &'static str, message: String, remediation_hint: String) -> Self {
        Self {
            code,
            message,
            remediation_hint,
            class: DenialClass::ActorCorrectable,
            held_permissions: Vec::new(),
            authorized_actions: Vec::new(),
            do_not: None,
        }
    }
}

/// Render a [`Denial`] as the canonical steering envelope JSON.
///
/// This is the shared superset shape consumed by the CLI serializers
/// (STEER-005). It emits every legacy key (`code`/`message`/
/// `remediation_hint`) PLUS the four steering fields (`class`,
/// `held_permissions`, `authorized_actions`, `do_not` when `Some`) so a
/// consumer reading only the legacy keys sees no regression, while a
/// steering consumer reads the additive fields off the same object.
///
/// `held_permissions` is sorted lexically by [`Authority::name()`] (via
/// [`crate::serialize_authority_tokens`]) so set-equality assertions on the
/// envelope are not order-flaky.
///
/// ```
/// use but_authz::{Authority, Denial, DenialClass, to_envelope};
///
/// let denial = Denial {
///     class: DenialClass::OperatorRequired,
///     held_permissions: vec![Authority::ReviewsWrite, Authority::CommentsWrite],
///     ..Denial::new(
///         Denial::PERM_DENIED_CODE,
///         "merge requires reviews:write".to_owned(),
///         "ask a reviews:write holder to approve".to_owned(),
///     )
/// };
/// let envelope = to_envelope(&denial);
/// assert_eq!(envelope["code"], "perm.denied");
/// assert_eq!(envelope["class"], "operator_required");
/// // held_permissions is sorted lexically, not insertion order:
/// assert_eq!(envelope["held_permissions"][0], "comments:write");
/// assert_eq!(envelope["held_permissions"][1], "reviews:write");
/// assert!(!envelope.as_object().unwrap().contains_key("do_not"));
/// ```
pub fn to_envelope(denial: &Denial) -> serde_json::Value {
    build_envelope(
        denial.code,
        &denial.message,
        Some(&denial.remediation_hint),
        denial.class,
        &denial.held_permissions,
        &denial.authorized_actions,
        denial.do_not,
    )
}

/// Build the canonical steering envelope from individual field parts.
///
/// This is the inner builder shared by [`to_envelope`] (pure, from a
/// [`Denial`]) and [`steer_envelope_from_parts`] (best-effort, from CLI
/// carrier parts). It does NOT apply the fault seam — callers control that.
fn build_envelope(
    code: &str,
    message: &str,
    remediation_hint: Option<&str>,
    class: DenialClass,
    held_permissions: &[Authority],
    authorized_actions: &[AuthorizedAction],
    do_not: Option<&str>,
) -> serde_json::Value {
    fn sorted_authority_tokens(authorities: &[Authority]) -> serde_json::Value {
        let mut sorted: Vec<Authority> = authorities.to_vec();
        sorted.sort_by_key(|authority| authority.name());
        serde_json::Value::Array(
            sorted
                .iter()
                .map(|authority| serde_json::Value::String(authority.name().to_owned()))
                .collect(),
        )
    }

    fn authorized_actions_value(actions: &[AuthorizedAction]) -> serde_json::Value {
        serde_json::Value::Array(
            actions
                .iter()
                .map(|action| {
                    serde_json::json!({
                        "command": action.command,
                        "effect": action.effect,
                    })
                })
                .collect(),
        )
    }

    let mut object = serde_json::Map::new();
    object.insert(
        "code".to_owned(),
        serde_json::Value::String(code.to_owned()),
    );
    object.insert(
        "message".to_owned(),
        serde_json::Value::String(message.to_owned()),
    );
    if let Some(hint) = remediation_hint {
        object.insert(
            "remediation_hint".to_owned(),
            serde_json::Value::String(hint.to_owned()),
        );
    }
    object.insert(
        "class".to_owned(),
        serde_json::Value::String(class.name().to_owned()),
    );
    object.insert(
        "held_permissions".to_owned(),
        sorted_authority_tokens(held_permissions),
    );
    object.insert(
        "authorized_actions".to_owned(),
        authorized_actions_value(authorized_actions),
    );
    if let Some(do_not) = do_not {
        object.insert(
            "do_not".to_owned(),
            serde_json::Value::String(do_not.to_owned()),
        );
    }
    serde_json::Value::Object(object)
}

/// Build a minimal legacy-only envelope (`code`/`message`/
/// `remediation_hint`) used by the best-effort fault fallback so a
/// serialization fault still denies with the legacy fields + exit 1.
fn minimal_envelope(
    code: &str,
    message: &str,
    remediation_hint: Option<&str>,
) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert(
        "code".to_owned(),
        serde_json::Value::String(code.to_owned()),
    );
    object.insert(
        "message".to_owned(),
        serde_json::Value::String(message.to_owned()),
    );
    if let Some(hint) = remediation_hint {
        object.insert(
            "remediation_hint".to_owned(),
            serde_json::Value::String(hint.to_owned()),
        );
    }
    serde_json::Value::Object(object)
}

/// TEST-ONLY fault-injection seam for best-effort serialization.
///
/// When `BUT_STEER_FORCE_SERIALIZATION_FAULT=1` is set in the environment
/// AND the build is a debug/test build, this returns `true` to simulate a
/// steering-payload serialization fault. Compiled out of release builds
/// via `cfg(not(debug_assertions))` — never a production bypass (SA-1/RR-4).
///
/// The seam is consumed by [`steer_envelope_from_parts`] so CLI serializers
/// can prove a forced fault still denies with the legacy fields + exit 1
/// (invariant §9.5, best-effort fail-closed).
#[cfg(debug_assertions)]
fn serialization_fault_forced() -> bool {
    // SAFETY note: read-only env var check; no panics, no mutation.
    std::env::var("BUT_STEER_FORCE_SERIALIZATION_FAULT").as_deref() == Ok("1")
}

#[cfg(not(debug_assertions))]
fn serialization_fault_forced() -> bool {
    false
}

/// Best-effort steering envelope for CLI denial serializers (STEER-005).
///
/// Renders the full steering envelope (`code`/`message`/`remediation_hint`
/// when `Some`/`class`/`held_permissions`/`authorized_actions`/`do_not` when
/// `Some`) from the parts supplied by a CLI serializer's carrier
/// classification. If the TEST-ONLY `BUT_STEER_FORCE_SERIALIZATION_FAULT`
/// seam forces a fault (debug-only), degrades to a minimal
/// `{code, message, remediation_hint}` envelope so the denial still emits
/// the legacy fields + exit 1 (best-effort fail-closed, invariant §9.5).
/// The seam is compiled out of release builds.
///
/// ```
/// use but_authz::{Authority, DenialClass, steer_envelope_from_parts};
///
/// let envelope = steer_envelope_from_parts(
///     "perm.denied",
///     "action requires contents:write",
///     Some("ask an administrator to grant contents:write"),
///     DenialClass::ActorCorrectable,
///     &[Authority::ReviewsWrite],
///     &[],
///     None,
/// );
/// assert_eq!(envelope["code"], "perm.denied");
/// assert_eq!(envelope["class"], "actor_correctable");
/// assert_eq!(envelope["held_permissions"][0], "reviews:write");
/// assert!(envelope.as_object().unwrap().contains_key("remediation_hint"));
/// ```
pub fn steer_envelope_from_parts(
    code: &str,
    message: &str,
    remediation_hint: Option<&str>,
    class: DenialClass,
    held_permissions: &[Authority],
    authorized_actions: &[AuthorizedAction],
    do_not: Option<&str>,
) -> serde_json::Value {
    if serialization_fault_forced() {
        return minimal_envelope(code, message, remediation_hint);
    }
    build_envelope(
        code,
        message,
        remediation_hint,
        class,
        held_permissions,
        authorized_actions,
        do_not,
    )
}
