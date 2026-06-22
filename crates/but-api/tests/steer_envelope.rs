//! STEER-001 envelope-shape proofs for the but-api carriers.
//!
//! Companion suite to `but-authz/tests/steer_carriers.rs`. Covers the four
//! carriers that live in but-api (MergeGateError, CommitGateError,
//! ForgeGateError, AdminWriteGateError), proving:
//!
//! 1. Each carrier serializes the additive superset alongside the legacy
//!    keys (`steer_merge_envelope_uniform_shape`).
//! 2. A back-compat reader of `{code,message,remediation_hint,unmet}`
//!    parses post-STEER output with no missing key
//!    (`steer_carrier_backcompat_reader`).
//! 3. `ForgeGateError::classify_error` and `AdminWriteGateError::classify_error`
//!    copy the four steering fields off the underlying `Denial` / `ConfigError`
//!    (`steer_forge_and_admin_carriers_copy_steering_fields`).
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-001-steering-fields-on-denial-carriers.md` for the task contract.

use but_api::legacy::merge_gate::MergeGateError;
use but_authz::{Authority, AuthorizedAction, Denial, DenialClass};

/// Build a MergeGateError with the full steering shape, in non-lexical
/// held_permissions order to prove the serializer sorts.
fn sample_merge_gate_error() -> MergeGateError {
    MergeGateError {
        code: "gate.review_required",
        message: "review requirement for refs/heads/main is not satisfied: min_approvals"
            .to_owned(),
        remediation_hint: "collect the required approvals at the current review head".to_owned(),
        unmet: vec!["no_approval".to_owned()],
        class: DenialClass::OperatorRequired,
        // ReviewsWrite is declared before CommentsWrite in the enum, so any
        // producer dumping iter() would get reviews,comments — the serializer
        // must sort to comments,reviews for stable lexical order.
        held_permissions: vec![Authority::ReviewsWrite, Authority::CommentsWrite],
        authorized_actions: vec![AuthorizedAction::new(
            "but review request",
            "request a review on the branch",
        )],
        do_not: Some("do not retry — this requires an operator"),
    }
}

/// AC-1 — MergeGateError serializes to one uniform envelope shape alongside
/// the Denial envelope (in but-authz). Both carry the same additive keys
/// modulo the merge-only `unmet` array.
#[test]
fn steer_merge_envelope_uniform_shape() {
    let merge = sample_merge_gate_error();
    let value = serde_json::to_value(&merge).expect("MergeGateError serializes");
    let object = value
        .as_object()
        .expect("MergeGateError serializes to a JSON object");

    // Legacy keys preserved.
    assert_eq!(
        object.get("code").and_then(serde_json::Value::as_str),
        Some("gate.review_required"),
        "code key must be preserved as the legacy gate.review_required token"
    );
    assert!(
        object
            .get("message")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "message key must be present"
    );
    assert!(
        object
            .get("remediation_hint")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        "remediation_hint key must be present"
    );
    assert_eq!(
        object.get("unmet").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("no_approval".to_owned())]),
        "unmet key must be preserved on MergeGateError"
    );

    // Additive steering fields present.
    assert_eq!(
        object.get("class").and_then(serde_json::Value::as_str),
        Some("operator_required"),
        "class key must be the DenialClass name token"
    );
    assert_eq!(
        object
            .get("held_permissions")
            .and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("comments:write".to_owned()),
            serde_json::Value::String("reviews:write".to_owned()),
        ]),
        "held_permissions must be the sorted `:`-token array, not insertion order"
    );
    assert!(
        object
            .get("authorized_actions")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "authorized_actions key must be present as an array"
    );
    assert_eq!(
        object.get("do_not").and_then(serde_json::Value::as_str),
        Some("do not retry — this requires an operator"),
        "do_not must serialize as the literal string when Some"
    );

    // The uniform shape matches the Denial envelope's key set modulo `unmet`.
    // Use the SAME do_not Some-value on the Denial envelope so the only
    // merge-only addition is `unmet`.
    let denial = Denial {
        do_not: Some("do not retry — this requires an operator"),
        ..Denial::new(
            Denial::PERM_DENIED_CODE,
            "merge requires reviews:write".to_owned(),
            "ask a reviews:write holder to approve".to_owned(),
        )
    };
    let denial_envelope = but_authz::to_envelope(&denial);
    let denial_keys: std::collections::BTreeSet<String> = denial_envelope
        .as_object()
        .expect("denial envelope is a JSON object")
        .keys()
        .cloned()
        .collect();
    let merge_keys: std::collections::BTreeSet<String> = object.keys().cloned().collect();

    // Every Denial key MUST appear on the MergeGateError envelope — the
    // Denial envelope is the canonical uniform shape consumed at the CLI
    // serializers (STEER-005).
    for denial_key in &denial_keys {
        assert!(
            merge_keys.contains(denial_key),
            "MergeGateError envelope must carry every Denial-envelope key — missing {denial_key}"
        );
    }
    // The only key MergeGateError adds is the merge-only `unmet`.
    let merge_only: Vec<_> = merge_keys.difference(&denial_keys).collect();
    assert_eq!(
        merge_only.as_slice(),
        &[&"unmet".to_string()],
        "MergeGateError must add ONLY the `unmet` key relative to the uniform Denial envelope"
    );
}

/// AC-2 / TC-5 — back-compat reader of {code,message,remediation_hint,unmet}
/// parses post-STEER MergeGateError output with no missing key.
#[test]
fn steer_carrier_backcompat_reader() {
    let merge = sample_merge_gate_error();
    let value = serde_json::to_value(&merge).expect("MergeGateError serializes");
    let object = value.as_object().expect("object shape");

    // A pre-STEER consumer reading only the legacy keys sees no regression.
    assert!(object.contains_key("code"), "back-compat: code key present");
    assert!(
        object.contains_key("message"),
        "back-compat: message key present"
    );
    assert!(
        object.contains_key("remediation_hint"),
        "back-compat: remediation_hint key present"
    );
    assert!(
        object.contains_key("unmet"),
        "back-compat: unmet key present"
    );

    // And the legacy values are unchanged relative to the input.
    assert_eq!(
        object.get("code").and_then(serde_json::Value::as_str),
        Some("gate.review_required")
    );
    assert_eq!(
        object
            .get("remediation_hint")
            .and_then(serde_json::Value::as_str),
        Some("collect the required approvals at the current review head")
    );
    assert_eq!(
        object.get("unmet").and_then(serde_json::Value::as_array),
        Some(&vec![serde_json::Value::String("no_approval".to_owned())])
    );

    // Exit code 1 is unchanged: this is enforced by the CLI serializer
    // (STEER-005), not by this crate — but the JSON shape MUST not regress
    // so the CLI exit code stays the same. The assertion above proves the
    // legacy-key reader finds every key it expects.
}

/// AC-6 / TC-11, TC-12 — ForgeGateError and AdminWriteGateError carry the
/// four steering fields copied off the underlying Denial (and class+do_not
/// off ConfigError) via `classify_error`. Neither is left as a two-key
/// `{code,message}` flatten.
#[test]
fn steer_forge_and_admin_carriers_copy_steering_fields() {
    // Underlying Denial with all four steering fields populated.
    let denial = Denial {
        class: DenialClass::OperatorRequired,
        held_permissions: vec![Authority::ReviewsWrite, Authority::CommentsWrite],
        authorized_actions: vec![AuthorizedAction::new(
            "but review request",
            "request a review on the branch",
        )],
        do_not: Some("do not retry — this requires an operator"),
        ..Denial::new(
            Denial::PERM_DENIED_CODE,
            "action requires reviews:write".to_owned(),
            "ask a reviews:write holder to grant it".to_owned(),
        )
    };
    let chain = anyhow::Error::from(denial.clone());

    // ForgeGateError copies all four off the underlying Denial.
    let forge_error = but_api::legacy::forge::classify_error(&chain)
        .expect("classify_error lifts ForgeGateError off a Denial chain");
    let forge_value = serde_json::to_value(&forge_error).expect("ForgeGateError serializes");
    let forge_object = forge_value
        .as_object()
        .expect("ForgeGateError is a JSON object");

    assert_eq!(
        forge_object.get("code").and_then(serde_json::Value::as_str),
        Some(Denial::PERM_DENIED_CODE),
        "ForgeGateError preserves code"
    );
    assert_eq!(
        forge_object
            .get("class")
            .and_then(serde_json::Value::as_str),
        Some("operator_required"),
        "ForgeGateError copies `class` off the underlying Denial"
    );
    assert_eq!(
        forge_object
            .get("held_permissions")
            .and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("comments:write".to_owned()),
            serde_json::Value::String("reviews:write".to_owned()),
        ]),
        "ForgeGateError copies `held_permissions` off the underlying Denial (sorted)"
    );
    assert!(
        forge_object
            .get("authorized_actions")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "ForgeGateError copies `authorized_actions` off the underlying Denial"
    );
    assert_eq!(
        forge_object
            .get("do_not")
            .and_then(serde_json::Value::as_str),
        Some("do not retry — this requires an operator"),
        "ForgeGateError copies `do_not` off the underlying Denial"
    );

    // AdminWriteGateError copies all four off the underlying Denial.
    let admin_error = but_api::legacy::config_mutate::classify_error(&chain)
        .expect("classify_error lifts AdminWriteGateError off a Denial chain");
    let admin_value = serde_json::to_value(&admin_error).expect("AdminWriteGateError serializes");
    let admin_object = admin_value
        .as_object()
        .expect("AdminWriteGateError is a JSON object");

    assert_eq!(
        admin_object
            .get("class")
            .and_then(serde_json::Value::as_str),
        Some("operator_required"),
        "AdminWriteGateError copies `class` off the underlying Denial"
    );
    assert!(
        admin_object
            .get("held_permissions")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "AdminWriteGateError copies `held_permissions` off the underlying Denial"
    );
    assert!(
        admin_object
            .get("authorized_actions")
            .and_then(serde_json::Value::as_array)
            .is_some(),
        "AdminWriteGateError copies `authorized_actions` off the underlying Denial"
    );
    assert_eq!(
        admin_object
            .get("do_not")
            .and_then(serde_json::Value::as_str),
        Some("do not retry — this requires an operator"),
        "AdminWriteGateError copies `do_not` off the underlying Denial"
    );

    // CommitGateError also copies the four off the underlying Denial —
    // same classify_error pattern, same field-source invariant.
    let commit_error = but_api::commit::create::gate::classify_error(&chain)
        .expect("classify_error lifts CommitGateError off a Denial chain");
    let commit_value = serde_json::to_value(&commit_error).expect("CommitGateError serializes");
    let commit_object = commit_value
        .as_object()
        .expect("CommitGateError is a JSON object");

    assert_eq!(
        commit_object
            .get("class")
            .and_then(serde_json::Value::as_str),
        Some("operator_required"),
        "CommitGateError copies `class` off the underlying Denial"
    );
    assert_eq!(
        commit_object
            .get("held_permissions")
            .and_then(serde_json::Value::as_array),
        Some(&vec![
            serde_json::Value::String("comments:write".to_owned()),
            serde_json::Value::String("reviews:write".to_owned()),
        ]),
        "CommitGateError copies `held_permissions` off the underlying Denial (sorted)"
    );

    // Drop the do_not value to None on a fresh denial and prove the
    // resulting ForgeGateError / AdminWriteGateError JSON omits the key
    // entirely (skip_serializing_if = Option::is_none).
    let mut denial_no_do_not = denial.clone();
    denial_no_do_not.do_not = None;
    let chain_no_do_not = anyhow::Error::from(denial_no_do_not);
    let forge_no_do_not = but_api::legacy::forge::classify_error(&chain_no_do_not)
        .expect("classify_error lifts ForgeGateError");
    let forge_no_do_not_json = serde_json::to_string(&forge_no_do_not).unwrap();
    assert!(
        !forge_no_do_not_json.contains("\"do_not\""),
        "ForgeGateError MUST omit do_not when underlying Denial has do_not=None: {forge_no_do_not_json}"
    );

    let _ = forge_error;
    let _ = admin_error;
}
