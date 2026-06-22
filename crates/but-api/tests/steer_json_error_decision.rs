//! STEER-005 AC-7 (SA-8): the Tauri/MGMT desktop `json::Error` surface
//! decision is explicitly recorded.
//!
//! The four steering fields (`class`/`held_permissions`/`authorized_actions`/
//! `do_not`) are NOT co-landed on `json::Error` because Sprint 06a
//! `MGMT-IPC-002` owns its `remediation_hint` addition and the task file is
//! frozen. This test asserts the deferral is an explicit, verifiable recorded
//! decision — never a silent gap.

use but_api::json::STEER_TAURI_JSON_ERROR_DEFERRAL;

/// AC-7 / TC-11 — the json::Error steering-field decision is recorded.
///
/// The deferral note MUST reference `MGMT-IPC-002` so the dependency is
/// verifiable. The note is a `pub const` on `json::Error`'s module so a
/// downstream consumer can assert it programmatically.
#[test]
fn steer_json_error_decision_recorded() {
    // The constant exists and is non-empty.
    assert!(
        !STEER_TAURI_JSON_ERROR_DEFERRAL.is_empty(),
        "the Tauri json::Error steering-field decision must be explicitly recorded"
    );

    // The decision references the Sprint 06a dependency (MGMT-IPC-002) so the
    // deferral is verifiable, not silent.
    assert!(
        STEER_TAURI_JSON_ERROR_DEFERRAL.contains("MGMT-IPC-002"),
        "the deferral MUST reference MGMT-IPC-002 as the dependency: {STEER_TAURI_JSON_ERROR_DEFERRAL}"
    );

    // The decision names the four steering fields so a reviewer can verify
    // what is deferred.
    assert!(
        STEER_TAURI_JSON_ERROR_DEFERRAL.contains("class"),
        "the deferral MUST name `class` as a deferred field: {STEER_TAURI_JSON_ERROR_DEFERRAL}"
    );

    // The decision states it is a deferral (not a silent gap).
    assert!(
        STEER_TAURI_JSON_ERROR_DEFERRAL.contains("deferred")
            || STEER_TAURI_JSON_ERROR_DEFERRAL.contains("deferral"),
        "the decision MUST state it is a deferral: {STEER_TAURI_JSON_ERROR_DEFERRAL}"
    );

    println!("AC-7: Tauri json::Error steering-field decision recorded (MGMT-IPC-002 deferral)");
}
