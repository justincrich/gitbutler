use std::{ffi::OsStr, fs, path::Path};

use but_api::commit::create::gate::{CommitGateTarget, enforce_commit_gate_for_target};
use but_api::legacy::{
    config_mutate::enforce_administration_write_gate,
    governance::{branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo},
};

const FEAT_REF: &str = "refs/heads/feat";

#[test]
#[serial_test::serial]
fn commit_gate_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_REGISTRY_PATH", Some(registry_path.as_os_str())),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", None),
            ("BUT_AGENT_HANDLE", None),
        ],
        || -> anyhow::Result<()> {
            let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);

            registered_then_unregistered_denied(&registry_path, || {
                enforce_commit_gate_for_target(&repo, &target)
            })
        },
    )
}

#[test]
#[serial_test::serial]
fn branch_gates_read_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            branch_gates_read_with_repo(&repo, FEAT_REF).map(|_| ())
        })
    })
}

#[test]
#[serial_test::serial]
fn group_list_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            group_list_with_repo(&repo, FEAT_REF).map(|_| ())
        })
    })
}

#[test]
#[serial_test::serial]
fn perm_list_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            perm_list_with_repo(&repo, FEAT_REF, None).map(|_| ())
        })
    })
}

#[test]
#[serial_test::serial]
fn admin_write_gate_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            enforce_administration_write_gate(&repo, FEAT_REF)
        })
    })
}

#[test]
#[serial_test::serial]
fn env_fallback_still_allowed_on_registry_miss() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, false)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_REGISTRY_PATH", Some(registry_path.as_os_str())),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some(OsStr::new("1"))),
            ("BUT_AGENT_HANDLE", Some(OsStr::new("dev"))),
        ],
        || -> anyhow::Result<()> {
            let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
            enforce_commit_gate_for_target(&repo, &target).map_err(|err| {
                anyhow::anyhow!(
                    "explicit env fallback should still satisfy commit gate on registry miss: {err:#}"
                )
            })
        },
    )
}

#[test]
fn scoped_gate_sources_do_not_reference_legacy_handle() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let token = "BUT_AGENT_".to_owned() + "HANDLE";
    for relative in [
        "src/commit/gate.rs",
        "src/legacy/merge_gate.rs",
        "src/legacy/governance.rs",
        "src/legacy/forge.rs",
        "src/legacy/config_mutate.rs",
    ] {
        let source = fs::read_to_string(manifest_dir.join(relative))?;
        assert!(
            !source.contains(&token),
            "{relative} must not reference the legacy env handle directly"
        );
    }
    Ok(())
}

fn with_registry_only(
    registry_path: &Path,
    f: impl FnOnce() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    temp_env::with_vars(
        [
            ("BUT_AGENT_REGISTRY_PATH", Some(registry_path.as_os_str())),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", None),
            ("BUT_AGENT_HANDLE", None),
        ],
        f,
    )
}

fn registered_then_unregistered_denied(
    registry_path: &Path,
    action: impl Fn() -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    action()?;
    write_process_registry(registry_path, false)?;
    let denial = match action() {
        Ok(()) => anyhow::bail!(
            "unregistered runtime process must be denied when env fallback is disabled"
        ),
        Err(err) => err
            .downcast::<but_authz::Denial>()
            .map_err(|err| anyhow::anyhow!("gate denial should be structured: {err:#}"))?,
    };

    assert_eq!(
        denial.code, "perm.denied",
        "unregistered runtime process must deny with the stable perm.denied code"
    );
    Ok(())
}

fn write_process_registry(path: &Path, registered: bool) -> anyhow::Result<()> {
    let mut registry = but_authz::Registry::load(path)?;
    let pid = but_authz::current_pid();
    let start_time = but_authz::process_start_time(pid)?;
    if registered {
        registry.register(pid, start_time, "dev", 60, "dev")?;
    } else {
        registry.unregister((pid, start_time));
    }
    registry.write(path)
}

fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = [
    "contents:write",
    "merge",
    "reviews:write",
    "comments:write",
    "pull_requests:write",
    "administration:read",
    "administration:write",
]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "feat"
protected = false
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
echo feat-base >feat-base.txt
git add feat-base.txt
git commit -m "feat base"
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}
