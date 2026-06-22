//! STEER-006: `but whoami` / `but can-i` self-scoped discovery.

use crate::utils::{CommandExt as _, Sandbox};

/// AC-1 (PRIMARY): `but whoami` with a committed governance config returns the
/// caller's resolved principal handle, sorted effective authorities, group
/// memberships, and authorized actions (intersection with
/// ROUTE_AUTHORITY_TABLE) as JSON. The caller's own data is the only data
/// disclosed — group rosters stay gated by `administration:read`.
#[test]
#[serial_test::serial]
fn whoami_returns_self_scoped_principal_info_as_json() -> anyhow::Result<()> {
    let env = governed_env()?;

    // "rust-implementer" holds contents:write directly + reviews:write via the
    // "backend" group. Effective authorities must include both, sorted.
    let output = env
        .but("--format json whoami")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(
        output.status.success(),
        "whoami must succeed for a known principal; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["principal"], "rust-implementer", "principal handle");
    let authorities = json["authorities"]
        .as_array()
        .expect("authorities is an array");
    let authority_tokens = authorities
        .iter()
        .map(|v| v.as_str().expect("authority token is a string").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        authority_tokens,
        vec!["contents:write", "reviews:write"],
        "effective authorities must be sorted and include direct + group-inherited tokens"
    );
    let groups = json["groups"].as_array().expect("groups is an array");
    let group_names = groups
        .iter()
        .map(|v| v.as_str().expect("group is a string").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        group_names,
        vec!["backend"],
        "group memberships must list the caller's own groups only"
    );
    let authorized_actions = json["authorized_actions"]
        .as_array()
        .expect("authorized_actions is an array");
    let action_commands = authorized_actions
        .iter()
        .map(|v| v.as_str().expect("action is a string").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        action_commands,
        vec!["commit", "request-changes"],
        "authorized actions must be the intersection of held authorities with ROUTE_AUTHORITY_TABLE, in table order"
    );
    // CRITICAL: the caller must NOT see group rosters (members of "backend").
    // Only group names are disclosed — not who else is in them.
    let raw = serde_json::to_string(&json)?;
    assert!(
        !raw.contains("rust-reviewer") && !raw.contains("admin"),
        "whoami must not disclose other principals in the caller's groups: {raw}"
    );
    assert_eq!(
        json["not_configured"], false,
        "not_configured must be false when a committed config exists"
    );

    Ok(())
}

/// AC-2: `but whoami` without BUT_AGENT_HANDLE exits 1 with a structured
/// `perm.denied` error envelope, matching the existing CLI denial shape so
/// steering consumers can route it uniformly.
#[test]
#[serial_test::serial]
fn whoami_without_handle_exits_one_with_perm_denied() -> anyhow::Result<()> {
    let env = governed_env()?;

    let output = env
        .but("--format json whoami")
        .allow_json()
        .env_remove("BUT_AGENT_HANDLE")
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "whoami must exit 1 when BUT_AGENT_HANDLE is unset"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(r#""code":"perm.denied""#),
        "whoami denial must be a structured perm.denied envelope, got: {stderr}"
    );
    assert!(
        stderr.contains("BUT_AGENT_HANDLE"),
        "whoami denial must name the missing BUT_AGENT_HANDLE handle, got: {stderr}"
    );

    Ok(())
}

/// AC-3: `but whoami` runs without a git repo and prints the caller's handle
/// with empty authorities/groups/actions and `not_configured: true`. These are
/// discovery commands, not repo-scoped ones.
#[test]
#[serial_test::serial]
fn whoami_outside_repo_prints_handle_with_not_configured() -> anyhow::Result<()> {
    let env = Sandbox::empty()?;

    let output = env
        .but("--format json whoami")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "lonely-agent")
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "whoami must succeed outside a repo when BUT_AGENT_HANDLE is set; stderr: {stderr}, stdout: {stdout}"
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow::anyhow!("invalid JSON stdout: {stdout:?} — parse error: {e}"))?;
    assert_eq!(json["principal"], "lonely-agent");
    assert_eq!(json["not_configured"], true);
    assert!(
        json["authorities"].as_array().unwrap().is_empty(),
        "authorities must be empty outside a repo"
    );
    assert!(
        json["groups"].as_array().unwrap().is_empty(),
        "groups must be empty outside a repo"
    );
    assert!(
        json["authorized_actions"].as_array().unwrap().is_empty(),
        "authorized_actions must be empty outside a repo"
    );

    Ok(())
}

/// AC-4 (PRIMARY): `but can-i <authority>` returns `{"authorized": true}` when
/// the caller holds the authority and `{"authorized": false}` when not. Both
/// cases exit 0 — this is a query, not an enforcement gate.
#[test]
#[serial_test::serial]
fn can_i_returns_authorized_bool_and_exits_zero() -> anyhow::Result<()> {
    let env = governed_env()?;

    // rust-implementer holds contents:write + reviews:write (via group).
    let yes = env
        .but("--format json can-i contents:write")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(
        yes.status.success(),
        "can-i must exit 0 even when checking a held authority; stderr: {}",
        String::from_utf8_lossy(&yes.stderr)
    );
    let yes_json: serde_json::Value = serde_json::from_slice(&yes.stdout)?;
    assert_eq!(yes_json["authorized"], true);
    assert_eq!(yes_json["action"], "contents:write");
    assert_eq!(yes_json["principal"], "rust-implementer");

    let no = env
        .but("--format json can-i merge")
        .allow_json()
        .env("BUT_AGENT_HANDLE", "rust-implementer")
        .output()?;
    assert!(
        no.status.success(),
        "can-i must exit 0 even when the authority is missing; stderr: {}",
        String::from_utf8_lossy(&no.stderr)
    );
    let no_json: serde_json::Value = serde_json::from_slice(&no.stdout)?;
    assert_eq!(no_json["authorized"], false);
    assert_eq!(no_json["action"], "merge");
    assert_eq!(no_json["principal"], "rust-implementer");

    Ok(())
}

/// AC-5: `but can-i` without BUT_AGENT_HANDLE prints
/// `{"authorized": false, "reason": "unresolved_principal"}` and exits 0 —
/// callers can pipe it into a query without distinguishing missing-principal
/// from denial.
#[test]
#[serial_test::serial]
fn can_i_without_handle_returns_unresolved_principal_and_exits_zero() -> anyhow::Result<()> {
    let env = governed_env()?;

    let output = env
        .but("--format json can-i merge")
        .allow_json()
        .env_remove("BUT_AGENT_HANDLE")
        .output()?;
    assert!(
        output.status.success(),
        "can-i must exit 0 even when the principal can't be resolved; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(json["authorized"], false);
    assert_eq!(json["reason"], "unresolved_principal");

    Ok(())
}

/// Build a sandbox with a committed governance config:
/// - `admin` holds `administration:write` directly.
/// - `rust-implementer` holds `contents:write` directly and inherits
///   `reviews:write` from the `backend` group.
/// - `rust-reviewer` is also in the `backend` group (used to assert that the
///   group roster is NOT disclosed to other members).
fn governed_env() -> anyhow::Result<Sandbox> {
    let env = Sandbox::open_scenario_with_target_and_default_settings("one-stack")?;
    env.invoke_bash(
        r#"
git branch -f main origin/main
git checkout main
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "admin"
permissions = ["administration:write"]

[[principal]]
id = "rust-implementer"
permissions = ["contents:write"]
groups = ["backend"]

[[principal]]
id = "rust-reviewer"
permissions = ["contents:read"]
groups = ["backend"]

[[group]]
name = "backend"
permissions = ["reviews:write"]
members = ["rust-implementer", "rust-reviewer"]
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
