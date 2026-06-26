use std::path::Path;

use but_api::commit::create::gate::{enforce_commit_gate_for_target, CommitGateTarget};

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

            enforce_commit_gate_for_target(&repo, &target).map_err(|err| {
                anyhow::anyhow!(
                    "registered runtime process should satisfy commit gate via registry, without BUT_AGENT_HANDLE fallback: {err:#}"
                )
            })?;

            write_process_registry(&registry_path, false)?;
            let denial = match enforce_commit_gate_for_target(&repo, &target) {
                Ok(()) => anyhow::bail!(
                    "unregistered runtime process must be denied when env fallback is disabled"
                ),
                Err(err) => err.downcast::<but_authz::Denial>().map_err(|err| {
                    anyhow::anyhow!("commit gate denial should be structured: {err:#}")
                })?,
            };

            assert_eq!(
                denial.code, "perm.denied",
                "unregistered runtime process must deny with the stable perm.denied code"
            );
            Ok(())
        },
    )
}

fn write_process_registry(path: &Path, registered: bool) -> anyhow::Result<()> {
    let mut registry = but_authz::Registry::empty();
    if registered {
        let pid = but_authz::current_pid();
        let start_time = but_authz::process_start_time(pid)?;
        registry.register(pid, start_time, "dev", 60, "dev")?;
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
permissions = ["contents:write"]
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
