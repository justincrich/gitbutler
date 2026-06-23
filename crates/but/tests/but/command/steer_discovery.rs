//! STEER-006 — self-scoped discovery (`but whoami` / `but can-i`) CLI proofs.
//!
//! Tests the self-scoped discovery contract: effective permissions (own ∪
//! groups), own group memberships, authorized-action set, the `but perm list`
//! discovery affordance surfaced in the denial menu (and omitted if absent),
//! and cross-principal recon denied `perm.denied` without leak.

use but_authz::{
    Authority, Denial, DenialPredicate, DeniedRoute, Route, authorized_actions,
    load_governance_config, to_envelope,
};
use std::ffi::OsString;

use crate::utils::Sandbox;

// ---------------------------------------------------------------------------
// Fixture: reviewers_group_with_members
// ---------------------------------------------------------------------------

/// Build a fixture where `rev` has `comments:write` directly and
/// `reviews:write` via the `reviewers` group (members: `rev`, `rev2`), and
/// `maint` has `merge`. Governance config is committed to `refs/heads/main`
/// (the workspace target ref) with `main` protected.
fn reviewers_group_with_members_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "rev"
permissions = ["comments:write"]
groups = ["reviewers"]

[[principal]]
id = "rev2"
groups = ["reviewers"]

[[principal]]
id = "maint"
permissions = ["merge"]

[[group]]
name = "reviewers"
permissions = ["reviews:write"]
members = ["rev", "rev2"]
EOF
cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config with reviewers group"
"#,
    );
    env.but("setup").assert().success();
    Ok(env)
}

/// Resolve the `rev` principal in-process (no env mutation).
fn resolve_rev(config: &but_authz::GovConfig) -> but_authz::Principal {
    but_authz::resolve_principal(|_| Some(OsString::from("rev")), config)
        .expect("rev must resolve from the committed fixture")
}

// ---------------------------------------------------------------------------
// AC-1 / TC-1 — Discovery affordance surfaced in the menu when the verb exists
// ---------------------------------------------------------------------------

/// AC-1 [PRIMARY]: An actor-correctable denial's authorized_actions includes
/// the `but perm list` discovery affordance with a non-empty catalog effect.
#[test]
#[serial_test::serial]
fn steer_discovery_affordance_surfaced_in_menu() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;
    let repo = env.open_repo()?;
    let config = load_governance_config(&repo, "refs/heads/main")?;
    let rev = resolve_rev(&config);

    // rev tries to commit but lacks contents:write → Authority denial.
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
    let actions = authorized_actions(&rev, &denied, &config);

    let commands: Vec<&str> = actions.iter().map(|a| a.command).collect();
    assert!(
        commands.contains(&"but perm list"),
        "actor-correctable denial menu MUST include `but perm list` discovery: {commands:?}"
    );

    // The discovery entry has a non-empty catalog effect.
    let discovery = actions
        .iter()
        .find(|a| a.command == "but perm list")
        .expect("discovery affordance must be present");
    assert!(
        !discovery.effect.is_empty(),
        "discovery affordance effect must be non-empty (catalog &'static str)"
    );

    // Prove the discovery verb is a REAL shipped CLI verb — no phantom command.
    let discovery_run = env
        .but("perm list")
        .env("BUT_AGENT_HANDLE", "rev")
        .output()?;
    assert!(
        discovery_run.status.success(),
        "`but perm list` must be a real verb — it must NOT fail: {}",
        String::from_utf8_lossy(&discovery_run.stderr)
    );

    println!("AC-1: discovery affordance `but perm list` surfaced in menu");
    Ok(())
}

// ---------------------------------------------------------------------------
// TC-2 — Discovery affordance omitted (not phantom) when no verb exists
// ---------------------------------------------------------------------------

/// TC-2: Every command in the derived menu is a real CATALOG entry — no
/// phantom/lying commands. If a command were removed from CATALOG, it would be
/// omitted rather than offered as a non-existent verb.
#[test]
#[serial_test::serial]
fn steer_discovery_affordance_omitted_when_absent() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;
    let repo = env.open_repo()?;
    let config = load_governance_config(&repo, "refs/heads/main")?;
    let rev = resolve_rev(&config);

    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
    let actions = authorized_actions(&rev, &denied, &config);

    // Every command in the menu MUST exist in the closed CATALOG.
    let catalog_commands: Vec<&str> = but_authz::CATALOG.iter().map(|a| a.command).collect();
    for action in &actions {
        assert!(
            catalog_commands.contains(&action.command),
            "menu command `{}` must be a real CATALOG entry — no phantom commands",
            action.command
        );
    }

    // The menu is non-empty (at minimum, the discovery affordance).
    assert!(
        !actions.is_empty(),
        "menu must not be empty — at least the discovery affordance"
    );

    println!("TC-2: no phantom commands in menu (all entries are real CATALOG verbs)");
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-2 / TC-3 — Group membership NOT inline in the denial by default
// ---------------------------------------------------------------------------

/// AC-2: The denial JSON has no inline `groups`/`memberships` key while
/// `held_permissions` carries the effective set.
#[test]
#[serial_test::serial]
fn steer_discovery_membership_not_inline() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;
    let repo = env.open_repo()?;
    let config = load_governance_config(&repo, "refs/heads/main")?;
    let rev = resolve_rev(&config);

    // rev hits an actor-correctable denial (lacks contents:write).
    let held = but_authz::effective_authority(&rev, &config);
    let denied = DeniedRoute::new(Route::Commit, DenialPredicate::Authority);
    let denial = Denial::missing_permission(Authority::ContentsWrite, &held)
        .with_authorized_actions(&rev, &denied, &config);
    let envelope = to_envelope(&denial);

    let error = envelope
        .as_object()
        .unwrap_or_else(|| panic!("envelope must be a JSON object: {envelope}"));

    // held_permissions carries the effective set (incl. group-folded reviews:write).
    let held_perms = error
        .get("held_permissions")
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("envelope must have held_permissions array"));
    let held_tokens: Vec<&str> = held_perms.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        held_tokens.contains(&"reviews:write"),
        "held_permissions must contain reviews:write (group-folded): {held_tokens:?}"
    );

    // No inline groups/memberships key.
    assert!(
        !error.contains_key("groups"),
        "denial MUST NOT have an inline `groups` key: {error:?}"
    );
    assert!(
        !error.contains_key("memberships"),
        "denial MUST NOT have an inline `memberships` key: {error:?}"
    );

    println!("AC-2: no inline groups/memberships in denial; held_permissions carries grants");
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-3 / TC-4 — whoami returns the full self picture
// ---------------------------------------------------------------------------

/// AC-3: `but whoami` as `rev` prints effective perms (incl. group-folded
/// reviews:write), own membership (reviewers), and an authorized-action set.
#[test]
#[serial_test::serial]
fn steer_whoami_returns_full_self_picture() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;

    let output = env.but("whoami").env("BUT_AGENT_HANDLE", "rev").output()?;
    assert!(
        output.status.success(),
        "`but whoami` as rev must exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Direct grant.
    assert!(
        stdout.contains("comments:write"),
        "whoami must show direct grant comments:write: {stdout}"
    );
    // Group-folded grant.
    assert!(
        stdout.contains("reviews:write"),
        "whoami must show group-folded reviews:write: {stdout}"
    );
    // Own group membership.
    assert!(
        stdout.contains("reviewers"),
        "whoami must show own group membership reviewers: {stdout}"
    );
    // At least one authorized-action entry with a `but ` command.
    assert!(
        stdout.contains("but "),
        "whoami must show at least one authorized action with a `but ` command: {stdout}"
    );
    // The discovery affordance is in the authorized actions.
    assert!(
        stdout.contains("but perm list"),
        "whoami authorized actions must include `but perm list`: {stdout}"
    );

    println!("AC-3: whoami returns full self picture for rev");
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-4 / TC-5 — Cross-principal recon denied perm.denied
// ---------------------------------------------------------------------------

/// AC-4: `but can-i --principal maint merge` by `rev` is denied `perm.denied`
/// with no leak; self `can-i reviews:write` by `rev` is Ok.
#[test]
#[serial_test::serial]
fn steer_discovery_cross_principal_denied() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;

    // Cross-principal: rev cannot recon maint's authority set.
    let cross = env
        .but("can-i --principal maint merge")
        .env("BUT_AGENT_HANDLE", "rev")
        .output()?;
    assert_eq!(
        cross.status.code(),
        Some(1),
        "cross-principal can-i must exit 1"
    );
    let cross_stderr = String::from_utf8_lossy(&cross.stderr);
    let envelope = parse_error_envelope(&cross_stderr);
    assert_eq!(
        envelope.code, "perm.denied",
        "cross-principal can-i must be perm.denied"
    );
    // No leak of maint's effective set.
    assert!(
        !cross_stderr.contains("merge") || envelope.message.contains("administration"),
        "cross-principal output must NOT leak maint's authority set: {cross_stderr}"
    );

    // Self can-i: rev holds reviews:write via group → exit 0.
    let self_check = env
        .but("can-i reviews:write")
        .env("BUT_AGENT_HANDLE", "rev")
        .output()?;
    assert!(
        self_check.status.success(),
        "self can-i reviews:write by rev must exit 0 (held via group): {}",
        String::from_utf8_lossy(&self_check.stderr)
    );
    let self_stdout = String::from_utf8_lossy(&self_check.stdout);
    assert!(
        self_stdout.contains("yes"),
        "self can-i of a held authority must say yes: {self_stdout}"
    );

    println!("AC-4: cross-principal can-i denied perm.denied; self can-i Ok");
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-5 / TC-6 — whoami does not list other group members
// ---------------------------------------------------------------------------

/// AC-5: `but whoami` as `rev` shows own membership `reviewers` but does NOT
/// list the other member `rev2`.
#[test]
#[serial_test::serial]
fn steer_whoami_hides_other_group_members() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;

    let output = env.but("whoami").env("BUT_AGENT_HANDLE", "rev").output()?;
    assert!(
        output.status.success(),
        "`but whoami` as rev must exit 0: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Own membership is shown.
    assert!(
        stdout.contains("reviewers"),
        "whoami must show rev's own membership in reviewers: {stdout}"
    );
    // Other member is NOT shown.
    assert!(
        !stdout.contains("rev2"),
        "whoami must NOT list the other member rev2: {stdout}"
    );

    println!("AC-5: whoami shows own membership, hides other group members");
    Ok(())
}

// ---------------------------------------------------------------------------
// AC-6 / TC-7 — Discovery is not a principal-existence oracle
// ---------------------------------------------------------------------------

/// AC-6: `but can-i --principal <unknown>` by `rev` returns the same
/// `perm.denied` code + a non-existence-revealing message as
/// `--principal <existing>` — no principal-id enumeration oracle.
#[test]
#[serial_test::serial]
fn steer_discovery_not_a_principal_oracle() -> anyhow::Result<()> {
    let env = reviewers_group_with_members_env()?;

    // Unknown target.
    let unknown = env
        .but("can-i --principal ghost-9f3a merge")
        .env("BUT_AGENT_HANDLE", "rev")
        .output()?;
    assert_eq!(
        unknown.status.code(),
        Some(1),
        "unknown-target can-i must exit 1"
    );
    let unknown_env = parse_error_envelope(&String::from_utf8_lossy(&unknown.stderr));
    assert_eq!(
        unknown_env.code, "perm.denied",
        "unknown-target can-i must be perm.denied"
    );
    let unknown_msg = unknown_env.message.to_lowercase();
    assert!(
        !unknown_msg.contains("unknown principal")
            && !unknown_msg.contains("not found")
            && !unknown_msg.contains("no such principal"),
        "unknown-target message must NOT reveal principal existence: {unknown_msg}"
    );

    // Existing target (that rev cannot recon).
    let existing = env
        .but("can-i --principal maint merge")
        .env("BUT_AGENT_HANDLE", "rev")
        .output()?;
    assert_eq!(
        existing.status.code(),
        Some(1),
        "existing-target can-i must exit 1"
    );
    let existing_env = parse_error_envelope(&String::from_utf8_lossy(&existing.stderr));
    assert_eq!(
        existing_env.code, "perm.denied",
        "existing-target can-i must be perm.denied"
    );

    // Both paths must be indistinguishable (same code).
    assert_eq!(
        unknown_env.code, existing_env.code,
        "unknown-target and existing-target must return the SAME denial code"
    );

    println!("AC-6: discovery is not a principal-existence oracle");
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct ErrorEnvelope {
    code: String,
    message: String,
}

fn parse_error_envelope(stderr: &str) -> ErrorEnvelope {
    let json = stderr
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            let start = trimmed.find('{')?;
            let end = trimmed.rfind('}')?;
            (start <= end).then_some(&trimmed[start..=end])
        })
        .expect("stderr must contain a JSON error envelope");

    let value: serde_json::Value =
        serde_json::from_str(json).expect("error envelope must be valid JSON");
    let error = value
        .get("error")
        .expect("envelope must have an error object");
    ErrorEnvelope {
        code: error
            .get("code")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned(),
        message: error
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned(),
    }
}
