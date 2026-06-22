use but_authz::{
    DenialPredicate, DeniedRoute, Principal as AuthzPrincipal, PrincipalId, Route,
    authorized_actions, load_governance_config,
};
use but_core::RefMetadata as _;
use but_db::ForgeReview;
use gix::refs::FullName;
use std::ffi::OsString;

use crate::utils::{CommandExt as _, Sandbox};

const REVIEW_ID: usize = 77;

#[test]
#[serial_test::serial]
fn governed_loop_reference_flow_full_loop() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;

    env.file("feature.txt", "feature work\n");
    env.but("--format json commit feat -m feature-work")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .assert()
        .success();
    let feat_after_implementer = ref_id(&repo, "refs/heads/feat")?;
    assert_ne!(
        feat_after_implementer, feat_before,
        "implementer has contents:write, so committing to the feature branch must advance it"
    );
    update_cached_review_head(&env, "feat", REVIEW_ID)?;

    let pr_new = env
        .but("--format json pr new feat -m 'Feature work'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_pr_new_reaches_forge_boundary(&pr_new, "reference-loop PR creation");

    assert_merge_denied_for_implementer(&env)?;
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied implementer merge must leave main unchanged"
    );

    let feat_before_reviewer = ref_id(&repo, "refs/heads/feat")?;
    env.file("reviewer-change.txt", "reviewer change\n");
    let reviewer_commit = env
        .but("--format json commit feat -m reviewer-change")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert_denial(
        &reviewer_commit,
        "perm.denied",
        "contents:write",
        None,
        "reviewer commit must be denied because reviews:write does not imply contents:write",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before_reviewer,
        "denied reviewer commit must leave feat unchanged"
    );

    let zero_approval_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_denial(
        &zero_approval_merge,
        "gate.review_required",
        "review requirement",
        Some(&["collect", "approvals", "current review head"]),
        "maintainer merge with no distinct approval must be denied by the review gate",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "zero-approval merge denial must leave main unchanged"
    );

    env.but("--format json review approve feat")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let maintainer_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_forge_boundary_after_gate(&maintainer_merge, REVIEW_ID);
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "local fixture has no forge completion, so permitted forge merge must not move main locally"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_remediation_traversable() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let denied_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    let denial = assert_denial(
        &denied_merge,
        "perm.denied",
        "merge",
        Some(&["request a reviewed merge"]),
        "implementer merge denial must carry a traversable remediation hint",
    );
    assert!(
        denial.remediation_hint.contains("reviewed merge"),
        "remediation rescope uses merge denial, so the structured hint must name the governed reviewed-merge path: {}",
        denial.remediation_hint
    );

    env.file("remediated.txt", "remediated feature\n");
    env.but("--format json commit feat -m remediated-feature")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .assert()
        .success();
    update_cached_review_head(&env, "feat", REVIEW_ID)?;

    let pr_new = env
        .but("--format json pr new feat -m 'Remediated feature'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_pr_new_reaches_forge_boundary(&pr_new, "remediation PR creation");

    env.but("--format json review approve feat")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .assert()
        .success();

    let maintainer_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "maintainer")
        .output()?;
    assert_forge_boundary_after_gate(&maintainer_merge, REVIEW_ID);
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "rescope: permitted local forge-bound merge does not fake remote landing"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_dryrun_no_bypass() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;
    let object_count_before = object_count(&env);

    let dry_run = env
        .but("--format json pr merge 77 --dry-run")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &dry_run,
        "perm.denied",
        "merge",
        Some(&["reviewed merge"]),
        "dry-run merge by an implementer without merge authority must still be denied",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied dry-run merge must leave main unchanged"
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before,
        "denied dry-run merge must leave the source branch unchanged"
    );
    assert_eq!(
        object_count(&env),
        object_count_before,
        "denied dry-run merge must not persist new git objects"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_auto_merge_denied() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    let auto_merge = env
        .but("--format json pr auto-merge 77")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &auto_merge,
        "perm.denied",
        "merge",
        Some(&["reviewed merge"]),
        "auto-merge must be gated by the same merge authority as explicit merge",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "denied auto-merge must leave main unchanged"
    );

    Ok(())
}

#[test]
#[serial_test::serial]
fn governed_loop_unset_handle_failclosed() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    for (label, handle) in [("unset", None), ("empty", Some(""))] {
        let mut cmd = env.but("--format json pr merge 77 --dry-run").allow_json();
        cmd = match handle {
            Some(value) => cmd.env("BUT_AGENT_HANDLE", value),
            None => cmd.env_remove("BUT_AGENT_HANDLE"),
        };
        let output = cmd.output()?;
        assert_denial(
            &output,
            "perm.denied",
            "BUT_AGENT_HANDLE",
            Some(&["BUT_AGENT_HANDLE"]),
            &format!("{label} principal handle must fail closed with a structured denial"),
        );
        assert_eq!(
            ref_id(&repo, "refs/heads/main")?,
            main_before,
            "{label} handle denial must leave main unchanged"
        );
    }

    // STEER-004 TC-2: verify via the authz API that the no_handle and
    // unknown_principal denials carry class=OperatorRequired (not
    // actor_correctable). STEER-005 serializes the class field at the CLI
    // sites; here we verify the underlying type carries the correct class.
    let cfg = load_governance_config(&repo, "refs/heads/feat")?;
    let no_handle_denial = but_authz::resolve_principal(|_| None, &cfg).unwrap_err();
    assert_eq!(
        no_handle_denial.class,
        but_authz::DenialClass::OperatorRequired,
        "no_handle() denial MUST be operator_required — security HIGH #2"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// STEER-004 — class-matrix + config.invalid-operator tests
// ---------------------------------------------------------------------------

/// AC-1 / TC-2 — the class is correct per (code, principal-resolution)
/// across the governed CLI loop.
///
/// Verifies both through the authz API (where STEER-004 wires the class
/// field on each Denial) AND through the CLI (which produces the denial).
/// STEER-005 serializes the class field at CLI sites; here we verify the
/// underlying type carries the correct class.
#[test]
#[serial_test::serial]
fn governed_loop_steer_class_matrix() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let main_before = ref_id(&repo, "refs/heads/main")?;

    // Load the SAME cfg the gates load at refs/heads/feat.
    let cfg = load_governance_config(&repo, "refs/heads/feat")?;

    // actor_correctable: implementer merge denied (missing merge authority).
    // The implementer IS resolved (registered in config) but lacks merge.
    let imp_id = PrincipalId::new("implementer");
    let imp_auth = cfg
        .principal_authorities(&imp_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("implementer must be in config"))?;
    let imp_principal = AuthzPrincipal::new(imp_id, imp_auth, []);
    let imp_denial =
        but_authz::authorize(&imp_principal, but_authz::Authority::Merge, &cfg).unwrap_err();
    assert_eq!(
        imp_denial.class,
        but_authz::DenialClass::ActorCorrectable,
        "resolved-principal perm.denied (implementer, missing merge) MUST be actor_correctable"
    );

    // Also verify the CLI produces the perm.denied for implementer.
    let imp_merge = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &imp_merge,
        "perm.denied",
        "merge",
        Some(&["reviewed merge"]),
        "implementer lacks merge authority — perm.denied is expected",
    );

    // operator_required: unset handle → no_handle() → OperatorRequired.
    let unset_denial = but_authz::resolve_principal(|_| None, &cfg).unwrap_err();
    assert_eq!(
        unset_denial.class,
        but_authz::DenialClass::OperatorRequired,
        "unset-handle perm.denied MUST be operator_required (security HIGH #2)"
    );
    assert!(
        unset_denial.do_not.is_some(),
        "unset-handle denial MUST carry a do_not"
    );

    // Also verify the CLI produces the perm.denied for unset handle.
    let unset_merge = env
        .but("--format json pr merge 77 --dry-run")
        .allow_json()
        .env_remove("BUT_AGENT_HANDLE")
        .output()?;
    assert_denial(
        &unset_merge,
        "perm.denied",
        "BUT_AGENT_HANDLE",
        Some(&["BUT_AGENT_HANDLE"]),
        "unset handle must fail closed with perm.denied",
    );

    // operator_required: unknown principal → OperatorRequired.
    let ghost_denial = but_authz::resolve_principal(
        |key| (key == "BUT_AGENT_HANDLE").then(|| OsString::from("ghost")),
        &cfg,
    )
    .unwrap_err();
    assert_eq!(
        ghost_denial.class,
        but_authz::DenialClass::OperatorRequired,
        "unknown-principal perm.denied MUST be operator_required"
    );

    assert_eq!(
        ref_id(&repo, "refs/heads/main")?,
        main_before,
        "all denial paths must leave main unchanged"
    );

    println!("AC-1 class matrix (via authz API):");
    println!("  implementer perm.denied  → actor_correctable");
    println!("  unset handle perm.denied → operator_required");
    println!("  ghost handle perm.denied → operator_required");
    Ok(())
}

/// AC-2 — a `config.invalid` denial is `operator_required` with an empty
/// menu and a do-not-retry `do_not`. Verified through the ConfigError type
/// directly (STEER-004 wires class+do_not at the config-load site).
#[test]
#[serial_test::serial]
fn governed_loop_steer_config_invalid_operator() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;

    // Corrupt the committed gates.toml at refs/heads/feat so loading the
    // governance config hits config.invalid.
    env.invoke_bash(
        r#"
index=$(mktemp)
export GIT_INDEX_FILE="$index"
base=$(git rev-parse refs/heads/feat)
git read-tree "$base"
bad_gates=$(printf '[[branch]\nname = "feat"\nprotected = nope\n' | git hash-object -w --stdin)
git update-index --add --cacheinfo 100644 "$bad_gates" .gitbutler/gates.toml
tree=$(git write-tree)
commit=$(printf 'corrupt gates\n' | git commit-tree "$tree" -p "$base")
git update-ref refs/heads/feat "$commit"
unset GIT_INDEX_FILE
"#,
    );

    // Load the governance config from the corrupted ref → ConfigError.
    let cfg_result = load_governance_config(&repo, "refs/heads/feat");
    let err = cfg_result.unwrap_err();

    // AC-2: ConfigError carries class=OperatorRequired + do_not.
    assert_eq!(
        err.code(),
        "config.invalid",
        "corrupt gates.toml must produce config.invalid"
    );
    assert_eq!(
        err.class,
        Some(but_authz::DenialClass::OperatorRequired),
        "config.invalid MUST carry class=OperatorRequired"
    );
    let do_not = err.do_not.expect("config.invalid MUST carry a do_not");
    assert!(
        do_not.contains("do not retry"),
        "config.invalid do_not MUST contain 'do not retry': {do_not}"
    );

    println!("AC-2: config.invalid → operator_required + do-not-retry do_not");
    Ok(())
}

// ---------------------------------------------------------------------------
// STEER-003 — capability-aware denial menu derivation tests
// ---------------------------------------------------------------------------

/// Load the governance config the gate uses at the target ref and resolve
/// the principal from the committed permissions. Used by the STEER-003 tests
/// to call `but_authz::authorized_actions` with the SAME inputs the gate
/// already loaded (same-cfg/ref by construction, M2).
fn steer_load_principal(
    env: &Sandbox,
    target_ref: &str,
    handle: &str,
) -> anyhow::Result<(AuthzPrincipal, but_authz::GovConfig)> {
    let repo = env.open_repo()?;
    let cfg = load_governance_config(&repo, target_ref)?;
    let id = PrincipalId::new(handle);
    let authorities = cfg
        .principal_authorities(&id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("principal {handle} must be in the governed config"))?;
    let principal = AuthzPrincipal::new(id, authorities, []);
    Ok((principal, cfg))
}

/// Collect the menu command strings for a denied-route context.
fn steer_menu_commands(
    principal: &AuthzPrincipal,
    denied: &DeniedRoute,
    cfg: &but_authz::GovConfig,
) -> Vec<&'static str> {
    authorized_actions(principal, denied, cfg)
        .iter()
        .map(|action| action.command)
        .collect()
}

/// AC-3 / TC-4, TC-5, TC-10 — a reviewer denied a commit on its OWN branch
/// sees runnable review actions (`but review request-changes`, `comment`),
/// `but review approve` is ABSENT (L1 self-approve exclusion), and following
/// `but review request-changes` returns exit 0.
#[test]
#[serial_test::serial]
fn governed_loop_steer_reviewer_menu_runnable_no_self_approve() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;

    // Step 1: reviewer commits to feat (their OWN branch) → denied perm.denied.
    env.file("reviewer-change.txt", "reviewer change\n");
    let reviewer_commit = env
        .but("--format json commit feat -m reviewer-change")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert_denial(
        &reviewer_commit,
        "perm.denied",
        "contents:write",
        None,
        "reviewer commit on own branch must be denied (perm.denied, missing contents:write)",
    );
    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before,
        "denied reviewer commit must leave feat unchanged"
    );

    // Step 2: derive the menu from the SAME cfg the gate loaded.
    let (principal, cfg) = steer_load_principal(&env, "refs/heads/feat", "reviewer")?;

    assert!(
        principal
            .authorities()
            .contains(but_authz::Authority::ReviewsWrite),
        "fixture: reviewer must hold reviews:write"
    );
    assert!(
        !principal
            .authorities()
            .contains(but_authz::Authority::ContentsWrite),
        "fixture: reviewer must NOT hold contents:write"
    );

    // RR-5: reviewer commits to protected main hits perm.denied (authority
    // checked BEFORE branch-protection predicate), NOT branch.protected.
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority).with_own_branch(true);
    let commands = steer_menu_commands(&principal, &denied, &cfg);

    assert!(
        commands.contains(&"but review request-changes"),
        "reviewer-denied-commit menu MUST include `but review request-changes`: {commands:?}"
    );
    assert!(
        commands.contains(&"but review comment"),
        "reviewer-denied-commit menu MUST include `but review comment`: {commands:?}"
    );
    assert!(
        !commands.contains(&"but review approve"),
        "reviewer-denied-commit menu on OWN branch must NOT include `but review approve` \
         (L1 self-approve exclusion): {commands:?}"
    );
    assert!(
        !commands.contains(&"but commit"),
        "reviewer-denied-commit menu must NOT include `but commit` (contents:write unheld): {commands:?}"
    );

    // Step 3: follow `but review request-changes` → exit 0 (runnable).
    let request_changes = env
        .but("--format json review request-changes feat -m 'please fix the tests'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert!(
        request_changes.status.success(),
        "`but review request-changes feat` must succeed (exit 0) for the reviewer — \
         it is the runnable verb the menu recommended. stderr: {}",
        String::from_utf8_lossy(&request_changes.stderr)
    );

    println!("AC-3: reviewer-denied-commit menu on own branch:");
    for command in &commands {
        println!("  - {command}");
    }
    println!("  -> followed `but review request-changes feat` -> exit 0 (runnable)");

    Ok(())
}

/// AC-6 / TC-9 — an actor-correctable denial's menu includes the
/// `but perm list` self-scoped discovery affordance.
#[test]
#[serial_test::serial]
fn governed_loop_steer_menu_includes_discovery() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;

    let (principal, cfg) = steer_load_principal(&env, "refs/heads/feat", "implementer")?;
    let denied = DeniedRoute::new(Route::Merge, DenialPredicate::Authority);
    let commands = steer_menu_commands(&principal, &denied, &cfg);

    assert!(
        commands.contains(&"but perm list"),
        "actor-correctable denial menu MUST include `but perm list` discovery: {commands:?}"
    );

    // The discovery verb is a REAL shipped CLI verb — prove it runs without
    // a governance denial.
    let discovery = env
        .but("--format json perm list")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    let stderr = String::from_utf8_lossy(&discovery.stderr);
    assert!(
        !stderr.contains(r#""code":"perm.denied""#),
        "`but perm list` must be a real verb — it must NOT return perm.denied: {stderr}"
    );

    println!("AC-6: menu includes `but perm list` discovery affordance");
    for command in &commands {
        println!("  - {command}");
    }

    Ok(())
}

/// AC-2 CLI proof — a `branch.protected` denial menu derived from the real
/// CLI fixture offers a feature-branch commit affordance, NOT the
/// protected-ref commit just denied (C5 gate-state-aware subtraction).
#[test]
#[serial_test::serial]
fn governed_loop_steer_protected_menu() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;

    // Load the cfg from feat (which carries the branch protection record for
    // main). The feat ref's gates.toml marks main as protected and feat as
    // unprotected — the C5 succeeding context.
    let (principal, cfg) = steer_load_principal(&env, "refs/heads/feat", "implementer")?;

    assert!(
        principal
            .authorities()
            .contains(but_authz::Authority::ContentsWrite),
        "fixture: implementer must hold contents:write"
    );
    assert!(
        cfg.branch("main").is_some_and(|b| b.protected()),
        "fixture: main must be protected"
    );

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::BranchProtected);
    let actions = authorized_actions(&principal, &denied, &cfg);
    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();

    assert!(
        commands.contains(&"but commit"),
        "branch.protected menu MUST include `but commit` (feature-branch affordance): {commands:?}"
    );

    let commit_action = actions
        .iter()
        .find(|a| a.command == "but commit")
        .expect("`but commit` must be in the menu");
    assert!(
        commit_action.effect.to_lowercase().contains("unprotected"),
        "`but commit` effect must name an UNPROTECTED feature branch (C5): {:?}",
        commit_action.effect
    );
    assert!(
        !commit_action
            .effect
            .to_lowercase()
            .contains("protected ref"),
        "`but commit` effect must NOT name the protected ref (C5 lying-menu guard): {:?}",
        commit_action.effect
    );

    assert!(
        commands.contains(&"but perm list"),
        "branch.protected menu must include discovery: {commands:?}"
    );

    println!("AC-2 CLI: branch.protected menu for implementer on main:");
    for action in &actions {
        println!("  - {} -> {}", action.command, action.effect);
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// STEER-005 — CLI denial serializers (four steering fields + fault seam)
// ---------------------------------------------------------------------------

/// Parse the full `error` object from a CLI denial's stderr as a JSON map.
///
/// Unlike [`parse_cli_error_envelope_opt`] (which extracts a fixed set of
/// fields), this returns every key the serializer emitted so STEER-005 tests
/// can assert on the steering fields (`class`, `held_permissions`,
/// `authorized_actions`, `do_not`) and the merge-site `unmet`.
fn parse_steering_envelope(
    output: &std::process::Output,
    reason: &str,
) -> serde_json::Map<String, serde_json::Value> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json_str = stderr
        .lines()
        .find_map(json_object_from_line)
        .unwrap_or_else(|| {
            panic!("{reason}; stderr must contain a parseable JSON error envelope, got: {stderr}")
        });
    let value: serde_json::Value = serde_json::from_str(json_str)
        .unwrap_or_else(|e| panic!("{reason}; stderr JSON must parse: {e}, got: {json_str}"));
    value
        .get("error")
        .and_then(serde_json::Value::as_object)
        .cloned()
        .unwrap_or_else(|| {
            panic!("{reason}; stderr JSON must have an `error` object, got: {json_str}")
        })
}

/// Assert that a JSON map has a key matching a string value, returning the
/// string. Panics with a clear message if the key is missing or not a string.
fn require_str<'v>(
    map: &'v serde_json::Map<String, serde_json::Value>,
    key: &str,
    reason: &str,
) -> &'v str {
    map.get(key)
        .and_then(serde_json::Value::as_str)
        .unwrap_or_else(|| panic!("{reason}; envelope must contain string `{key}`: {map:?}"))
}

/// Assert that a JSON map has a key that is a non-empty array, returning it.
fn require_array<'v>(
    map: &'v serde_json::Map<String, serde_json::Value>,
    key: &str,
    reason: &str,
) -> &'v [serde_json::Value] {
    map.get(key)
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{reason}; envelope must contain array `{key}`: {map:?}"))
}

/// AC-1 / TC-1, TC-2 — commit_gate_cli_error emits the four steering fields
/// (`class`/`held_permissions`/`authorized_actions`/`do_not`) PLUS the
/// long-missing `remediation_hint`, exit 1 unchanged.
#[test]
#[serial_test::serial]
fn steer_cli_serde_commit_gate_carries_class_held_menu_do_not() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;

    // Reviewer (holds reviews:write but NOT contents:write) commits to feat.
    env.file("steer-commit-test.txt", "steer commit test\n");
    let output = env
        .but("--format json commit feat -m steer-commit-test")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "commit denial must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_steering_envelope(&output, "AC-1 commit denial");

    // Legacy fields preserved.
    assert_eq!(require_str(&envelope, "code", "AC-1"), "perm.denied");
    assert!(
        require_str(&envelope, "message", "AC-1").contains("contents:write"),
        "AC-1 message must name the missing authority"
    );

    // The long-missing remediation_hint is now present.
    assert!(
        !require_str(&envelope, "remediation_hint", "AC-1").is_empty(),
        "AC-1 remediation_hint must be non-empty"
    );

    // The four steering fields.
    assert_eq!(
        require_str(&envelope, "class", "AC-1"),
        "actor_correctable",
        "AC-1 class must be the stable token actor_correctable"
    );
    assert!(
        !require_array(&envelope, "held_permissions", "AC-1").is_empty(),
        "AC-1 held_permissions must be non-empty (reviewer holds reviews:write)"
    );
    assert!(
        !require_array(&envelope, "authorized_actions", "AC-1").is_empty(),
        "AC-1 authorized_actions must carry the recovery menu"
    );
    // do_not is present only when Some (omitted entirely when None,
    // matching the carrier's skip_serializing_if = "Option::is_none").
    if let Some(do_not_val) = envelope.get("do_not") {
        assert!(
            do_not_val.as_str().is_some_and(|s| !s.is_empty()),
            "AC-1 do_not must be a non-empty string when present: {do_not_val:?}"
        );
    }

    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before,
        "denied commit must leave feat unchanged"
    );

    println!(
        "AC-1: commit_gate_cli_error carries class/held_permissions/authorized_actions/do_not + remediation_hint"
    );
    Ok(())
}

/// AC-2 / TC-3 — review_gate_cli_error emits the four steering fields PLUS
/// the newly-added remediation_hint, exit 1 unchanged.
#[test]
#[serial_test::serial]
fn steer_cli_serde_review_gate_carries_steering_fields() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;

    // Implementer (holds contents:write but NOT reviews:write) tries review.
    let output = env
        .but("--format json review request-changes feat -m 'please fix'")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "review denial must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_steering_envelope(&output, "AC-2 review denial");

    // Legacy fields.
    assert_eq!(require_str(&envelope, "code", "AC-2"), "perm.denied");
    assert!(
        !require_str(&envelope, "message", "AC-2").is_empty(),
        "AC-2 message must be non-empty"
    );

    // The newly-added remediation_hint.
    assert!(
        !require_str(&envelope, "remediation_hint", "AC-2").is_empty(),
        "AC-2 remediation_hint must be non-empty (newly added by STEER-005)"
    );

    // The four steering fields.
    assert_eq!(
        require_str(&envelope, "class", "AC-2"),
        "actor_correctable",
        "AC-2 class must be the stable token"
    );
    assert!(
        !require_array(&envelope, "held_permissions", "AC-2").is_empty(),
        "AC-2 held_permissions must be non-empty (implementer holds authorities)"
    );
    // authorized_actions is present (the field exists on the serialized
    // envelope); STEER-004 enriches it per-route, but the review route's
    // enrichment is separate from the serialization work (STEER-005).
    assert!(
        envelope.contains_key("authorized_actions"),
        "AC-2 authorized_actions field must be present on the envelope"
    );

    println!("AC-2: review_gate_cli_error carries the four steering fields + remediation_hint");
    Ok(())
}

/// AC-3 / TC-4 — merge_gate_cli_error adds the four steering fields while
/// preserving remediation_hint + unmet, exit 1 unchanged.
#[test]
#[serial_test::serial]
fn steer_cli_serde_merge_gate_carries_unmet_and_steering() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;

    // Implementer (lacks merge authority) tries to merge PR 77.
    let output = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "merge denial must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_steering_envelope(&output, "AC-3 merge denial");

    // Legacy fields.
    assert_eq!(require_str(&envelope, "code", "AC-3"), "perm.denied");
    assert!(
        !require_str(&envelope, "message", "AC-3").is_empty(),
        "AC-3 message must be non-empty"
    );

    // The merge site's existing remediation_hint is preserved.
    assert!(
        !require_str(&envelope, "remediation_hint", "AC-3").is_empty(),
        "AC-3 remediation_hint must be non-empty (preserved)"
    );

    // The merge-site-only `unmet` key is retained.
    assert!(
        envelope.contains_key("unmet"),
        "AC-3 unmet must be present (merge-site-only, preserved)"
    );

    // The four steering fields.
    assert_eq!(
        require_str(&envelope, "class", "AC-3"),
        "actor_correctable",
        "AC-3 class must be the stable token"
    );
    assert!(
        envelope.contains_key("held_permissions"),
        "AC-3 held_permissions must be present"
    );
    assert!(
        envelope.contains_key("authorized_actions"),
        "AC-3 authorized_actions must be present"
    );

    println!(
        "AC-3: merge_gate_cli_error carries steering fields + preserves unmet + remediation_hint"
    );
    Ok(())
}

/// AC-4 / TC-5, TC-6 — governance_cli_error (admin-write) emits the four
/// steering fields + remediation_hint + the admin-write affordance row,
/// exit 1 unchanged.
#[test]
#[serial_test::serial]
fn steer_cli_serde_governance_carries_admin_steering() -> anyhow::Result<()> {
    // The governed_loop_env fixture is optimized for commit/review/merge
    // flows; the `but perm` command resolves its target ref through the
    // workspace base-branch data, which requires a main-checkout fixture.
    let env = steer_governance_env()?;

    // Implementer (lacks administration:write) tries to grant a permission.
    let output = env
        .but("perm grant --principal reviewer reviews:write")
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;

    assert_eq!(
        output.status.code(),
        Some(1),
        "governance (admin-write) denial must exit 1; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_steering_envelope(&output, "AC-4 governance denial");

    // Legacy fields.
    assert_eq!(require_str(&envelope, "code", "AC-4"), "perm.denied");
    assert!(
        require_str(&envelope, "message", "AC-4").contains("administration:write"),
        "AC-4 message must name administration:write"
    );

    // The newly-added remediation_hint.
    assert!(
        !require_str(&envelope, "remediation_hint", "AC-4").is_empty(),
        "AC-4 remediation_hint must be non-empty (newly added by STEER-005)"
    );

    // The four steering fields.
    assert_eq!(
        require_str(&envelope, "class", "AC-4"),
        "actor_correctable",
        "AC-4 admin-write denial must be actor_correctable"
    );
    assert!(
        envelope.contains_key("held_permissions"),
        "AC-4 held_permissions must be present"
    );
    // The admin-write affordance row surfaces in authorized_actions.
    assert!(
        envelope.contains_key("authorized_actions"),
        "AC-4 authorized_actions must be present"
    );

    println!(
        "AC-4: governance_cli_error carries class=actor_correctable + steering fields + remediation_hint"
    );
    Ok(())
}

/// Fixture for governance (admin-write) CLI serializer tests.
///
/// Mirrors the `perm_env` pattern from `perm.rs` but lives in the
/// governed_loop module so the STEER-005 serializer tests share helpers.
fn steer_governance_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write", "merge"]

[[principal]]
id = "implementer"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "reviewer"
permissions = ["contents:read"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
"#,
    );
    env.but("setup").assert().success();
    Ok(env)
}

/// AC-5 / TC-7, TC-8 — serialized `class` is a stable enum STRING token
/// branchable without parsing `message`.
#[test]
#[serial_test::serial]
fn steer_cli_serde_class_token_branchable() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;

    // An actor_correctable denial: reviewer (holds reviews:write but NOT
    // contents:write) commits to feat → perm.denied, actor_correctable.
    env.file("class-token-actor.txt", "actor\n");
    let actor_output = env
        .but("--format json commit feat -m class-token-actor")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .output()?;
    assert_eq!(
        actor_output.status.code(),
        Some(1),
        "AC-5 actor_correctable denial must exit 1"
    );
    let actor_env = parse_steering_envelope(&actor_output, "AC-5 actor_correctable");
    let actor_class = require_str(&actor_env, "class", "AC-5 actor_correctable");
    assert_eq!(
        actor_class, "actor_correctable",
        "AC-5 TC-7: class must be the stable token `actor_correctable` (a JSON string), \
         not a nested object or PascalCase Debug form"
    );

    // An operator_required denial: unset BUT_AGENT_HANDLE → ghost principal
    // → perm.denied with class=operator_required.
    env.file("class-token-operator.txt", "operator\n");
    let operator_output = env
        .but("--format json commit feat -m class-token-operator")
        .allow_json()
        .env_remove("BUT_AGENT_HANDLE")
        .output()?;
    // The ghost-handle denial should exit 1 with operator_required class.
    if operator_output.status.code() == Some(1) {
        let operator_env = parse_steering_envelope(&operator_output, "AC-5 operator_required");
        let operator_class = require_str(&operator_env, "class", "AC-5 operator_required");
        assert_eq!(
            operator_class, "operator_required",
            "AC-5 TC-8: class must be the stable token `operator_required` \
             (the class dimension is not collapsed)"
        );
        // Prove the two denials carry DIFFERENT class tokens — an orchestrator
        // can branch on `class` without parsing `message`.
        assert_ne!(
            actor_class, operator_class,
            "AC-5: the two denial classes must differ (branchable without message)"
        );
    }

    println!(
        "AC-5: serialized class is the stable token `{actor_class}` (branchable without parsing message)"
    );
    Ok(())
}

/// AC-6 / TC-9, TC-10 — best-effort fail-closed via the
/// BUT_STEER_FORCE_SERIALIZATION_FAULT seam: a forced steering-payload
/// serialization fault still denies with code/message/remediation_hint +
/// exit 1.
#[test]
#[serial_test::serial]
fn steer_cli_serde_fault_still_emits_code_message_exit1() -> anyhow::Result<()> {
    let env = governed_loop_env("feat", REVIEW_ID)?;
    let repo = env.open_repo()?;
    let feat_before = ref_id(&repo, "refs/heads/feat")?;

    // Reviewer commits to feat (perm.denied) WITH the fault seam forced.
    env.file("fault-test.txt", "fault seam\n");
    let output = env
        .but("--format json commit feat -m fault-test")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "reviewer")
        .env("BUT_STEER_FORCE_SERIALIZATION_FAULT", "1")
        .output()?;

    // TC-10: the fault must NOT flip deny→allow — exit is still 1.
    assert_eq!(
        output.status.code(),
        Some(1),
        "AC-6 a serialization fault must still deny (exit 1), never allow (exit 0); \
         stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_steering_envelope(&output, "AC-6 fault denial");

    // TC-9: the fault still emits code/message/remediation_hint.
    assert!(
        envelope.contains_key("code"),
        "AC-6 fault must still emit `code`: {envelope:?}"
    );
    assert!(
        envelope.contains_key("message"),
        "AC-6 fault must still emit `message`: {envelope:?}"
    );
    assert!(
        envelope.contains_key("remediation_hint"),
        "AC-6 fault must still emit `remediation_hint`: {envelope:?}"
    );
    let code = require_str(&envelope, "code", "AC-6");
    assert_eq!(
        code, "perm.denied",
        "AC-6 fault must preserve the exact denial code"
    );

    // The steering fields are absent in the fault fallback (minimal envelope).
    // This proves the seam activated: the full envelope would have class etc.
    assert!(
        !envelope.contains_key("class"),
        "AC-6 fault fallback must emit the minimal envelope (no class): {envelope:?}"
    );

    assert_eq!(
        ref_id(&repo, "refs/heads/feat")?,
        feat_before,
        "faulted denial must leave feat unchanged"
    );

    println!(
        "AC-6: BUT_STEER_FORCE_SERIALIZATION_FAULT seam → still denies with code/message/remediation_hint + exit 1"
    );
    Ok(())
}

fn governed_loop_env(branch_name: &str, review_id: usize) -> anyhow::Result<Sandbox> {
    let env = Sandbox::init_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(format!(
        r#"
git remote set-url origin https://github.com/gitbutler/governed-loop-fixture.git
git branch -f main origin/main
git branch -m A {branch_name}
write_governance_commit() {{
    target_ref="$1"
    base=$(git rev-parse "$target_ref")
    index=$(mktemp)
    export GIT_INDEX_FILE="$index"
    git read-tree "$base"
    permissions_blob=$(git hash-object -w --stdin <<'EOF'
[[principal]]
id = "implementer"
permissions = ["contents:write", "pull_requests:write"]

[[principal]]
id = "reviewer"
groups = ["code-reviewers"]

[[principal]]
id = "maintainer"
groups = ["maintainers"]

[[group]]
name = "code-reviewers"
permissions = ["reviews:write", "comments:write"]
members = ["reviewer"]

[[group]]
name = "maintainers"
permissions = ["merge"]
members = ["maintainer"]
EOF
)
    gates_main_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
EOF
)
    gates_branch_blob=$(git hash-object -w --stdin <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "{branch_name}"
protected = false
EOF
)
    git update-index --add --cacheinfo 100644 "$permissions_blob" .gitbutler/permissions.toml
    if test "$target_ref" = "refs/heads/main"
    then
        git update-index --add --cacheinfo 100644 "$gates_main_blob" .gitbutler/gates.toml
    else
        git update-index --add --cacheinfo 100644 "$gates_branch_blob" .gitbutler/gates.toml
    fi
    tree=$(git write-tree)
    commit=$(printf 'governance config\n' | git commit-tree "$tree" -p "$base")
    git update-ref "$target_ref" "$commit"
    rm "$index"
    unset GIT_INDEX_FILE
}}
write_governance_commit refs/heads/main
write_governance_commit refs/heads/{branch_name}
git checkout {branch_name}
"#
    ));
    env.but("setup").assert().success();
    env.set_target_sha("refs/heads/main")?;
    env.setup_metadata(&[branch_name])?;
    env.but(format!("apply {branch_name}")).assert().success();
    attach_review_id(&env, branch_name, review_id)?;
    upsert_cached_review(&env, branch_name, review_id)?;
    Ok(env)
}

fn assert_merge_denied_for_implementer(env: &Sandbox) -> anyhow::Result<()> {
    let output = env
        .but("--format json pr merge 77 --method squash")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "implementer")
        .output()?;
    assert_denial(
        &output,
        "perm.denied",
        "merge",
        Some(&["reviewed merge"]),
        "implementer lacks merge authority, so explicit merge must be denied",
    );
    Ok(())
}

#[derive(Debug, Clone)]
struct CliErrorEnvelope {
    code: String,
    message: String,
    remediation_hint: String,
    /// STEER-004: steering classification — "actor_correctable" or
    /// "operator_required". Parsed from the CLI envelope; STEER-005
    /// serializes it at CLI sites. Retained for forward-compat assertions.
    #[allow(dead_code)]
    class: String,
}

fn assert_denial(
    output: &std::process::Output,
    code: &str,
    expected_message_text: &str,
    expected_hint_terms: Option<&[&str]>,
    reason: &str,
) -> CliErrorEnvelope {
    assert_eq!(
        output.status.code(),
        Some(1),
        "{reason}; denial must exit with code 1. stdout: {}; stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let envelope = parse_cli_error_envelope(output, reason);
    assert_eq!(
        envelope.code, code,
        "{reason}; expected exact denial code {code}"
    );
    assert!(
        !envelope.message.trim().is_empty(),
        "{reason}; structured error.message must be non-empty"
    );
    assert!(
        envelope.message.contains(expected_message_text),
        "{reason}; expected error.message to contain {expected_message_text:?}, got: {}",
        envelope.message
    );

    if let Some(expected_hint_terms) = expected_hint_terms {
        assert!(
            !envelope.remediation_hint.trim().is_empty(),
            "{reason}; structured error.remediation_hint must be non-empty"
        );
        for expected_hint_term in expected_hint_terms {
            assert!(
                envelope.remediation_hint.contains(expected_hint_term),
                "{reason}; expected error.remediation_hint to contain {expected_hint_term:?}, got: {}",
                envelope.remediation_hint
            );
        }
    }

    envelope
}

fn assert_no_governance_denial(output: &std::process::Output, label: &str) {
    assert!(
        output.status.code() != Some(1) || parse_cli_error_envelope_opt(output).is_none(),
        "{label} must not return a structured governance denial envelope: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains(r#""code":"perm.denied""#)
            && !stderr.contains(r#""code":"branch.protected""#)
            && !stderr.contains(r#""code":"gate.review_required""#),
        "{label} must not fail with a governance denial: {stderr}"
    );
}

fn assert_pr_new_reaches_forge_boundary(output: &std::process::Output, label: &str) {
    assert!(
        !output.status.success(),
        "{label} runs in a local fixture without forge credentials, so it must fail after the PR gate at the forge boundary"
    );
    assert_no_governance_denial(output, label);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No authenticated forge users found")
            || stderr.contains("No authenticated GitHub users found")
            || stderr.contains("Failed to create pull request")
            || stderr.contains("No forge could be determined"),
        "{label} must reach the downstream forge/provider boundary after pull_requests:write authorization, got: {stderr}"
    );
}

fn assert_forge_boundary_after_gate(output: &std::process::Output, review_id: usize) {
    assert!(
        !output.status.success(),
        "local fixture has no forge credentials, so the permitted merge should fail at the forge boundary"
    );
    assert_no_governance_denial(output, "authorized maintainer merge");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!(
            "forge merge_review boundary rejected review {review_id}"
        )),
        "authorized merge must reach the forge merge_review boundary, got: {stderr}"
    );
}

fn parse_cli_error_envelope(output: &std::process::Output, reason: &str) -> CliErrorEnvelope {
    parse_cli_error_envelope_opt(output).unwrap_or_else(|| {
        panic!(
            "{reason}; stderr must contain a parseable CLI JSON error envelope, got: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn parse_cli_error_envelope_opt(output: &std::process::Output) -> Option<CliErrorEnvelope> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let json = stderr.lines().find_map(json_object_from_line)?;
    let value = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let error = value.get("error")?;
    let code = error.get("code")?.as_str()?.to_owned();
    let message = error.get("message")?.as_str()?.to_owned();
    let remediation_hint = error
        .get("remediation_hint")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned();
    Some(CliErrorEnvelope {
        code,
        message,
        remediation_hint,
        class: error
            .get("class")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("actor_correctable")
            .to_owned(),
    })
}

fn json_object_from_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    (start <= end).then_some(&trimmed[start..=end])
}

fn attach_review_id(env: &Sandbox, branch_name: &str, review_id: usize) -> anyhow::Result<()> {
    let mut meta = env.meta()?;
    let ref_name = FullName::try_from(format!("refs/heads/{branch_name}"))?;
    let mut branch = meta.branch(ref_name.as_ref())?;
    branch.review.pull_request = Some(review_id);
    meta.set_branch(&branch)?;
    Ok(())
}

fn update_cached_review_head(
    env: &Sandbox,
    branch_name: &str,
    review_id: usize,
) -> anyhow::Result<()> {
    upsert_cached_review(env, branch_name, review_id)
}

fn upsert_cached_review(env: &Sandbox, branch_name: &str, review_id: usize) -> anyhow::Result<()> {
    let repo = env.open_repo()?;
    let head = ref_id(&repo, &format!("refs/heads/{branch_name}"))?;
    let ctx = env.context()?;
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: format!(
                "https://github.com/gitbutler/governed-loop-fixture/pull/{review_id}"
            ),
            number: review_id.try_into()?,
            title: format!("Governed loop {branch_name}"),
            body: None,
            author: Some("implementer".to_owned()),
            labels: "[]".to_owned(),
            draft: false,
            source_branch: branch_name.to_owned(),
            target_branch: "main".to_owned(),
            sha: head.to_string(),
            created_at: None,
            modified_at: None,
            merged_at: None,
            closed_at: None,
            repository_ssh_url: None,
            repository_https_url: Some(
                "https://github.com/gitbutler/governed-loop-fixture.git".to_owned(),
            ),
            repo_owner: Some("gitbutler".to_owned()),
            head_repo_is_fork: false,
            reviewers: "[]".to_owned(),
            unit_symbol: "#".to_owned(),
            last_sync_at: fixed_time(0),
            struct_version: but_forge::ForgeReview::struct_version(),
        })?;
    Ok(())
}

fn object_count(env: &Sandbox) -> usize {
    env.invoke_git("rev-list --objects --all").lines().count()
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
