//! Shared JSON types and utilities to produce decent JSON from API types.
//!
//! This module is reserved for general-purpose transport helpers and JSON types
//! that are shared across API modules.
//!
//! If a JSON type only mirrors one API submodule, define it next to that API in
//! a local `json` module instead of adding it here. See `crate::branch::json`,
//! `crate::commit::json`, and `crate::diff::json` for the intended pattern.
pub use error::{
    ConfigInvalid, Error, STEER_TAURI_JSON_ERROR_DEFERRAL, ToJsonError, UnmarkedError,
};
use gix::refs::Target;
use schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

mod hex_hash {
    use std::{ops::Deref, str::FromStr};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// A type that deserializes a hexadecimal hash into an object id automatically.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct HexHash(pub gix::ObjectId);

    impl From<HexHash> for gix::ObjectId {
        fn from(value: HexHash) -> Self {
            value.0
        }
    }

    impl From<gix::ObjectId> for HexHash {
        fn from(value: gix::ObjectId) -> Self {
            HexHash(value)
        }
    }

    impl Deref for HexHash {
        type Target = gix::ObjectId;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<'de> Deserialize<'de> for HexHash {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let hex = String::deserialize(deserializer)?;
            gix::ObjectId::from_str(&hex)
                .map(HexHash)
                .map_err(serde::de::Error::custom)
        }
    }

    impl Serialize for HexHash {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.0.to_hex().to_string())
        }
    }

    mod stringy {
        use std::str::FromStr;

        use schemars::JsonSchema;
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        /// A type that deserializes a hexadecimal hash into a string, unchanged.
        /// This is to workaround `schemars` which doesn't (always) work with transformations.
        #[derive(Debug, Clone, JsonSchema)]
        pub struct HexHashString(String);

        impl TryFrom<HexHashString> for gix::ObjectId {
            type Error = gix::hash::decode::Error;

            fn try_from(value: HexHashString) -> Result<Self, Self::Error> {
                value.0.parse()
            }
        }

        impl From<gix::ObjectId> for HexHashString {
            fn from(value: gix::ObjectId) -> Self {
                HexHashString(value.to_hex().to_string())
            }
        }

        impl<'de> Deserialize<'de> for HexHashString {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let hex = String::deserialize(deserializer)?;
                gix::ObjectId::from_str(&hex)
                    .map(|_| HexHashString(hex))
                    .map_err(serde::de::Error::custom)
            }
        }

        impl Serialize for HexHashString {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }
    }
    pub use stringy::HexHashString;

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn hex_hash() {
            let hex_str = "5c69907b1244089142905dba380371728e2e8160";
            let expected = gix::ObjectId::from_str(hex_str).expect("valid SHA1 hex-string");
            let actual =
                serde_json::from_str::<HexHash>(&format!("\"{hex_str}\"")).expect("input is valid");
            assert_eq!(actual.0, expected);

            let actual = serde_json::to_string(&actual);
            assert_eq!(
                actual.unwrap(),
                "\"5c69907b1244089142905dba380371728e2e8160\""
            );
        }
    }
}
pub use hex_hash::{HexHash, HexHashString};

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(HexHashString);

/// Shared JSON transport type for mutation workspace results.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "export-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceState {
    /// Commits that were replaced by the operation. Maps `oldId -> newId`.
    #[cfg_attr(
        feature = "export-schema",
        schemars(with = "std::collections::BTreeMap<String, String>")
    )]
    pub replaced_commits: std::collections::BTreeMap<HexHash, HexHash>,
    /// The post-operation workspace view presented to the frontend.
    pub head_info: but_workspace::ui::RefInfo,
}

#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(WorkspaceState);

impl TryFrom<crate::WorkspaceState> for WorkspaceState {
    type Error = anyhow::Error;

    fn try_from(
        crate::WorkspaceState {
            replaced_commits,
            head_info,
        }: crate::WorkspaceState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            replaced_commits: replaced_commits
                .into_iter()
                .map(|(old, new)| (HexHash::from(old), HexHash::from(new)))
                .collect(),
            head_info: head_info.try_into()?,
        })
    }
}

mod error {
    //! Utilities to control which errors show in the frontend.
    //!
    //! ## How to use this
    //!
    //! Just make sure this `Error` type is used for each provided `tauri` command. The rest happens automatically
    //! such that [context](gitbutler_error::error::Context) is handled correctly.
    //!
    //! ### Interfacing with `tauri` using `Error`
    //!
    //! `tauri` serializes backend errors and makes these available as JSON objects to the frontend. The format
    //! is an implementation detail, but here it's implemented to turn each `Error` into a dict with `code`
    //! and `message` fields.
    //!
    //! The values in these fields are controlled by attaching context, please [see the `error` docs](gitbutler_error::error))
    //! on how to do this.

    use std::borrow::Cow;

    use but_error::AnyhowContextExt;
    use serde::{Serialize, ser::SerializeMap};

    /// An error type for serialization which isn't expected to carry a code.
    #[derive(Debug)]
    pub struct UnmarkedError(anyhow::Error);

    impl<T> From<T> for UnmarkedError
    where
        T: std::error::Error + Send + Sync + 'static,
    {
        fn from(err: T) -> Self {
            Self(err.into())
        }
    }

    impl Serialize for UnmarkedError {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let ctx = self.0.custom_context_or_error_chain();

            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("code", &ctx.code.to_string())?;
            let message = ctx.message.unwrap_or_else(|| {
                self.0
                    .source()
                    .map(|err| Cow::Owned(err.to_string()))
                    .unwrap_or_else(|| Cow::Borrowed("Something went wrong"))
            });
            map.serialize_entry("message", &message)?;
            map.end()
        }
    }

    /// Structured config.invalid error payload for JSON transport callers.
    ///
    /// ```
    /// let error = but_api::json::ConfigInvalid {
    ///     code: "config.invalid",
    ///     message: "governance config is malformed".to_owned(),
    ///     remediation_hint: "fix the malformed governance config".to_owned(),
    /// };
    /// assert_eq!(error.code, "config.invalid");
    /// ```
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct ConfigInvalid {
        /// Stable consumer-facing error code.
        pub code: &'static str,
        /// Human-readable config load or parse message.
        pub message: String,
        /// Actionable recovery hint for the invalid config.
        pub remediation_hint: String,
    }

    impl std::fmt::Display for ConfigInvalid {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str(&self.message)
        }
    }

    impl std::error::Error for ConfigInvalid {}

    /// An error type for serialization, dynamically extracting context information during serialization,
    /// meant for consumption by the frontend.
    #[derive(Debug)]
    pub struct Error(anyhow::Error);

    impl From<anyhow::Error> for Error {
        fn from(value: anyhow::Error) -> Self {
            Self(value)
        }
    }

    impl From<Error> for anyhow::Error {
        fn from(value: Error) -> Self {
            value.0
        }
    }

    /// A utility to convert any `Result<T, impl std::error::Error>` into a [JSON-Error](Error).
    pub trait ToJsonError<T> {
        /// Convert this instance into a Result<T, [JSON-Error](Error)>.
        fn to_json_error(self) -> Result<T, Error>;
    }

    impl<T, E: std::error::Error + Send + Sync + 'static> ToJsonError<T> for Result<T, E> {
        fn to_json_error(self) -> Result<T, Error> {
            self.map_err(|e| Error(e.into()))
        }
    }

    struct HintCarrier<'a> {
        code: &'static str,
        remediation_hint: &'a str,
    }

    fn hint_carrier(err: &anyhow::Error) -> Option<HintCarrier<'_>> {
        for cause in err.chain() {
            if let Some(denial) = cause.downcast_ref::<but_authz::Denial>() {
                return Some(HintCarrier {
                    code: denial.code,
                    remediation_hint: &denial.remediation_hint,
                });
            }

            #[cfg(feature = "legacy")]
            if let Some(error) = cause.downcast_ref::<crate::legacy::merge_gate::MergeGateError>() {
                return Some(HintCarrier {
                    code: error.code,
                    remediation_hint: &error.remediation_hint,
                });
            }

            if let Some(error) = cause.downcast_ref::<ConfigInvalid>() {
                return Some(HintCarrier {
                    code: error.code,
                    remediation_hint: &error.remediation_hint,
                });
            }
        }

        None
    }

    impl Serialize for Error {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let ctx = self.0.custom_context_or_error_chain();
            let hint_carrier = hint_carrier(&self.0);

            let mut map =
                serializer.serialize_map(Some(if hint_carrier.is_some() { 3 } else { 2 }))?;
            let code = hint_carrier
                .as_ref()
                .map_or_else(|| ctx.code.to_string(), |carrier| carrier.code.to_owned());
            map.serialize_entry("code", &code)?;
            let message = ctx.message.unwrap_or_else(|| {
                self.0
                    .source()
                    .map(|err| Cow::Owned(err.to_string()))
                    .unwrap_or_else(|| Cow::Borrowed("An unknown backend error occurred"))
            });
            map.serialize_entry("message", &message)?;
            if let Some(carrier) = hint_carrier {
                map.serialize_entry("remediation_hint", carrier.remediation_hint)?;
            }
            map.end()
        }
    }

    /// STEER-005 (SA-8): the Tauri/MGMT desktop surface rides this `Error`
    /// type, which emits `{code, message, remediation_hint}`. The four
    /// steering fields (`class`/`held_permissions`/`authorized_actions`/
    /// `do_not`) are NOT yet co-landed here because Sprint 06a `MGMT-IPC-002`
    /// owns the `remediation_hint` addition and its task file is frozen.
    ///
    /// This constant is the explicit, verifiable recorded decision: the
    /// desktop steering-field gap is DEFERRED to MGMT-IPC-002. The test
    /// `steer_json_error_decision_recorded` asserts this decision exists so
    /// the gap is never silent.
    pub const STEER_TAURI_JSON_ERROR_DEFERRAL: &str = "STEER-005 SA-8: the four steering fields (class/held_permissions/\
         authorized_actions/do_not) are deferred on json::Error until Sprint \
         06a MGMT-IPC-002 lands. The desktop surface currently emits \
         {code,message,remediation_hint} only. This is a tracked deferral, \
         not a silent gap.";

    #[cfg(test)]
    mod tests {
        use anyhow::anyhow;
        use but_authz::{Authority, AuthoritySet, Denial};
        use but_error::{Code, Context};
        use serde_json::{Map, Value};

        use crate::legacy::merge_gate::MergeGateError;

        use super::*;

        fn json(err: anyhow::Error) -> String {
            serde_json::to_string(&Error(err)).unwrap()
        }

        fn json_object(err: anyhow::Error) -> Map<String, Value> {
            serde_json::from_str::<Value>(&json(err))
                .unwrap()
                .as_object()
                .cloned()
                .unwrap()
        }

        fn assert_remediation_hint(
            object: &Map<String, Value>,
            expected_code: &str,
            expected_hint: &str,
        ) {
            assert_eq!(
                object.get("code").and_then(Value::as_str),
                Some(expected_code),
                "structured carrier code should cross the JSON error transport"
            );
            assert_eq!(
                object.len(),
                3,
                "only code, message, and remediation_hint should cross the JSON error transport"
            );
            let hint = object
                .get("remediation_hint")
                .and_then(Value::as_str)
                .expect(
                    "structured carrier remediation_hint should cross the JSON error transport",
                );
            assert!(
                hint.contains(expected_hint),
                "remediation_hint should come from the structured error carrier"
            );
        }

        #[test]
        fn error_serializes_remediation_hint_from_denial() {
            let denial =
                Denial::missing_permission(Authority::ReviewsWrite, &AuthoritySet::empty());
            let object = json_object(anyhow::Error::from(denial));

            assert_remediation_hint(
                &object,
                Denial::PERM_DENIED_CODE,
                "request a reviewed merge or ask a maintainer to grant reviews:write",
            );
            assert!(
                object
                    .get("message")
                    .and_then(Value::as_str)
                    .is_some_and(|message| message.contains("reviews:write")),
                "the carrier message should still name the denied authority"
            );
        }

        #[test]
        fn error_serializes_remediation_hint_from_merge_gate_error() {
            let error = MergeGateError {
                code: "gate.review_required",
                message: "review requirement for refs/heads/main is not satisfied: min_approvals"
                    .to_owned(),
                remediation_hint: "collect the required approvals at the current review head"
                    .to_owned(),
                unmet: vec!["min_approvals".to_owned()],
                class: but_authz::DenialClass::OperatorRequired,
                held_permissions: Vec::new(),
                authorized_actions: Vec::new(),
                do_not: None,
            };
            let object = json_object(anyhow::Error::from(error));

            assert_remediation_hint(
                &object,
                "gate.review_required",
                "collect the required approvals at the current review head",
            );
            assert!(
                !object.contains_key("unmet"),
                "carrier-private unmet fragments must not cross the JSON error transport"
            );
        }

        #[test]
        fn error_without_denial_keeps_two_field_shape() {
            let err = anyhow!("err msg").context(Code::Validation);
            let object = json_object(err);

            assert_eq!(
                serde_json::to_string(&object).unwrap(),
                "{\"code\":\"Validation\",\"message\":\"err msg\"}",
                "plain errors without a hint carrier keep the existing two-field JSON shape"
            );
            assert_eq!(
                object.len(),
                2,
                "plain errors without a hint carrier should serialize exactly two fields"
            );
            assert!(
                !object.contains_key("remediation_hint"),
                "plain errors without a hint carrier should not emit remediation_hint"
            );
        }

        #[test]
        fn error_recovers_hint_from_nested_denial() {
            let err = anyhow::Error::from(Denial::no_handle())
                .context("failed to authorize governance write");
            let object = json_object(err);

            assert_remediation_hint(
                &object,
                Denial::PERM_DENIED_CODE,
                "set BUT_AGENT_HANDLE to a principal committed in governance config",
            );
        }

        #[test]
        fn error_serializes_remediation_hint_from_config_invalid() {
            let carrier = Denial::new(
                "config.invalid",
                "governance config .gitbutler/permissions.toml is malformed".to_owned(),
                "fix the malformed governance config and recommit it to the target branch"
                    .to_owned(),
            );
            let object = json_object(anyhow::Error::from(carrier));

            assert_remediation_hint(
                &object,
                "config.invalid",
                "fix the malformed governance config",
            );
            assert!(
                object
                    .get("remediation_hint")
                    .and_then(Value::as_str)
                    .is_some_and(|hint| !hint.is_empty()),
                "config.invalid carrier should provide a non-empty remediation_hint"
            );
        }

        #[test]
        fn no_context_or_code_shows_root_error() {
            let err = anyhow!("err msg");
            assert_eq!(
                format!("{err:#}"),
                "err msg",
                "just one error on display here"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Unknown\",\"message\":\"err msg\"}",
                "if there is no explicit error code or context, the original error message is shown (and chain)"
            );
        }

        #[test]
        fn find_code() {
            let err = anyhow!("err msg").context(Code::Validation);
            assert_eq!(
                format!("{err:#}"),
                "Validation: err msg",
                "note how the context becomes an error, in front of the original one"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"err msg\"}",
                "the 'code' is available as string, but the message is taken from the source error"
            );
        }

        #[test]
        fn error_chain_display_without_context_or_code() {
            let original_err = std::io::Error::other("actual cause");
            let err = anyhow::Error::from(original_err).context("err msg");

            insta::assert_json_snapshot!(Error(err), @r#"
            {
              "code": "Unknown",
              "message": "err msg\n\nCaused by:\n    1: actual cause\n"
            }
            "#);
        }

        #[test]
        fn find_code_after_cause() {
            let original_err = std::io::Error::other("actual cause");
            let err = anyhow::Error::from(original_err)
                .context("err msg")
                .context(Code::Validation);

            assert_eq!(
                format!("{err:#}"),
                "Validation: err msg: actual cause",
                "an even longer chain, with the cause as root as one might expect"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"err msg\"}",
                "in order to attach a custom message to an original cause, our messaging (and Code) is the tail"
            );
        }

        #[test]
        fn find_context() {
            let err = anyhow!("err msg").context(Context::new_static(Code::Validation, "ctx msg"));
            assert_eq!(format!("{err:#}"), "ctx msg: err msg");
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"ctx msg\"}",
                "Contexts often provide their own message, so the error message is ignored"
            );
        }

        #[test]
        fn find_context_without_message() {
            let err = anyhow!("err msg").context(Context::from(Code::Validation));
            assert_eq!(
                format!("{err:#}"),
                "Something went wrong: err msg",
                "on display, `Context` does just insert a generic message"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"err msg\"}",
                "Contexts without a message show the error's message as well"
            );
        }

        #[test]
        fn find_nested_code() {
            let err = anyhow!("bottom msg")
                .context("top msg")
                .context(Code::Validation);
            assert_eq!(
                format!("{err:#}"),
                "Validation: top msg: bottom msg",
                "now it's clear why bottom is bottom"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"top msg\"}",
                "the 'code' gets the message of the error that it provides context to, and it finds it down the chain"
            );
        }

        #[test]
        fn multiple_codes() {
            let err = anyhow!("bottom msg")
                .context(Code::ProjectGitAuth)
                .context("top msg")
                .context(Code::Validation);
            assert_eq!(
                format!("{err:#}"),
                "Validation: top msg: ProjectGitAuth: bottom msg",
                "each code is treated like its own error in the chain"
            );
            assert_eq!(
                json(err),
                "{\"code\":\"Validation\",\"message\":\"top msg\"}",
                "it finds the most recent 'code' (and the same would be true for contexts, of course)"
            );
        }
    }
}

/// To make bstring work with schemars.
#[cfg(feature = "path-bytes")]
fn bstring_schema(generate: &mut schemars::SchemaGenerator) -> schemars::Schema {
    // TODO: implement this. How to get description and what not?
    generate.root_schema_for::<String>()
}

/// The full name of a Git reference.
#[derive(Debug, Clone, schemars::JsonSchema, Serialize)]
pub struct FullRefName {
    /// The full name, like `refs/heads/main` or `refs/remotes/origin/foo`.
    /// Note that it might be degenerated if it can't be represented in Unicode.
    pub full: String,
    /// `full` without degeneration, as plain bytes.
    #[cfg(feature = "path-bytes")]
    #[schemars(schema_with = "bstring_schema")]
    pub full_bytes: bstr::BString,
}
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(FullRefName);

impl From<gix::refs::FullName> for FullRefName {
    fn from(value: gix::refs::FullName) -> Self {
        FullRefName {
            full: value.as_bstr().to_string(),
            #[cfg(feature = "path-bytes")]
            full_bytes: value.as_bstr().into(),
        }
    }
}

/// An optional full reference name accepted as a string like `refs/heads/main`,
/// for use as a parameter transport via `#[but_api(...)]`.
///
/// The name is validated during deserialization. Note that it is lossy: a name
/// that can't be represented in Unicode can't be passed through this type.
#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
#[serde(try_from = "Option<String>")]
pub struct MaybeLossyFullNameRef(
    #[schemars(schema_with = "but_schemars::fullname_lossy_opt")] Option<gix::refs::FullName>,
);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(MaybeLossyFullNameRef);

impl TryFrom<Option<String>> for MaybeLossyFullNameRef {
    type Error = gix::refs::name::Error;

    fn try_from(value: Option<String>) -> Result<Self, Self::Error> {
        Ok(Self(value.map(gix::refs::FullName::try_from).transpose()?))
    }
}

impl From<MaybeLossyFullNameRef> for Option<gix::refs::FullName> {
    fn from(value: MaybeLossyFullNameRef) -> Self {
        value.0
    }
}

/// A full reference name accepted as raw bytes.
///
/// Use this as parameter transport when callers must avoid lossy UTF-8
/// conversion at the API boundary.
#[derive(Debug, Clone, schemars::JsonSchema, Deserialize)]
#[serde(try_from = "Vec<u8>")]
#[schemars(schema_with = "but_schemars::fullname_bytes")]
pub struct FullNameBytes(gix::refs::FullName);
#[cfg(feature = "export-schema")]
but_schemars::register_sdk_type!(FullNameBytes);

impl TryFrom<Vec<u8>> for FullNameBytes {
    type Error = gix::refs::name::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self(gix::refs::FullName::try_from(bstr::BString::from(
            value,
        ))?))
    }
}

impl From<FullNameBytes> for gix::refs::FullName {
    fn from(value: FullNameBytes) -> Self {
        value.0
    }
}

#[cfg(test)]
mod maybe_lossy_full_name_ref_tests {
    use super::{FullNameBytes, MaybeLossyFullNameRef};

    #[test]
    fn maybe_lossy_full_name_ref() {
        let actual: Option<gix::refs::FullName> =
            serde_json::from_str::<MaybeLossyFullNameRef>("\"refs/heads/main\"")
                .expect("valid full ref name")
                .into();
        assert_eq!(actual.expect("present").as_bstr(), "refs/heads/main");

        let actual: Option<gix::refs::FullName> =
            serde_json::from_str::<MaybeLossyFullNameRef>("null")
                .expect("null is a valid absent name")
                .into();
        assert_eq!(actual, None);

        serde_json::from_str::<MaybeLossyFullNameRef>("\"not-a-full-name\"")
            .expect_err("partial ref names are rejected");
    }

    #[test]
    fn full_name_bytes() {
        let actual: gix::refs::FullName = serde_json::from_str::<FullNameBytes>(
            "[114,101,102,115,47,104,101,97,100,115,47,109,97,105,110]",
        )
        .expect("valid full ref name bytes")
        .into();
        assert_eq!(actual.as_bstr(), "refs/heads/main");

        serde_json::from_str::<FullNameBytes>("[109,97,105,110]")
            .expect_err("partial ref names are rejected");
    }
}

/// A Git reference identified by its full reference name, along with the information Git stores about it.
#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct Reference {
    /// The full name, like `refs/heads/main` or `refs/remotes/origin/foo`.
    /// Note that it might be degenerated if it can't be represented in Unicode.
    pub name: FullRefName,
    /// Set if the reference points to an object id. This is the common case.
    #[serde(default)]
    pub target_id: Option<HexHashString>,
    /// Set if the reference points to the name of another reference. This happens if the reference is symbolic.
    #[serde(default)]
    pub target_ref: Option<FullRefName>,
}

impl From<gix::refs::Reference> for Reference {
    fn from(
        gix::refs::Reference {
            name,
            target,
            peeled: _ignored,
        }: gix::refs::Reference,
    ) -> Self {
        Reference {
            name: name.into(),
            target_id: match &target {
                Target::Object(id) => Some((*id).into()),
                Target::Symbolic(_) => None,
            },
            target_ref: match target {
                Target::Object(_) => None,
                Target::Symbolic(rn) => Some(rn.into()),
            },
        }
    }
}
