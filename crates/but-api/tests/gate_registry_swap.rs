use std::{ffi::OsStr, fs, path::Path};

use but_api::commit::create::gate::{CommitGateTarget, enforce_commit_gate_for_target};
use but_api::legacy::{
    config_mutate::enforce_administration_write_gate,
    forge::authorize_branch_action,
    governance::{branch_gates_read_with_repo, group_list_with_repo, perm_list_with_repo},
    merge_gate::enforce_merge_gate,
};
use but_db::ForgeReview;

const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

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
fn merge_gate_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;
    let ctx = context_with_review(&repo)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || enforce_merge_gate(&ctx, REVIEW_ID))
    })
}

#[test]
#[serial_test::serial]
fn forge_review_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            let principal =
                authorize_branch_action(&repo, "feat", but_authz::Authority::ReviewsWrite)?;
            let principal = principal.ok_or_else(|| {
                anyhow::anyhow!(
                    "feat branch carries governance config and must resolve a principal"
                )
            })?;
            assert_eq!(
                principal.id().as_str(),
                "dev",
                "registry entry must resolve the runtime process as the governed dev principal"
            );
            Ok(())
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
#[serial_test::serial]
fn unreadable_registry_falls_through_to_structured_denial() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    fs::create_dir(&registry_path)?;

    with_registry_only(&registry_path, || {
        let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
        assert_perm_denied(enforce_commit_gate_for_target(&repo, &target))
    })
}

#[test]
#[serial_test::serial]
fn env_fallback_still_allowed_when_registry_unreadable() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agent-registry.toml");
    fs::create_dir(&registry_path)?;

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
                    "unreadable registry must fall through to explicit env fallback: {err:#}"
                )
            })
        },
    )
}

#[test]
fn production_sources_do_not_use_legacy_env_resolver() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    for path in rust_sources(&manifest_dir.join("src"))? {
        let source = fs::read_to_string(&path)?;
        let relative = path.strip_prefix(manifest_dir)?.display();
        assert!(
            !source.contains("resolve_principal_from_env("),
            "{relative} must resolve through the runtime registry helper, not the env-only resolver"
        );
        for (line_idx, line) in source.lines().enumerate() {
            let reads_agent_handle = line.contains("BUT_AGENT_HANDLE")
                && (line.contains("env::var")
                    || line.contains("env::var_os")
                    || line.contains("std::env::var")
                    || line.contains("std::env::var_os"));
            assert!(
                !reads_agent_handle,
                "{relative}:{} must not read BUT_AGENT_HANDLE directly",
                line_idx + 1
            );
        }
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
    assert_perm_denied(action())
}

fn assert_perm_denied(result: anyhow::Result<()>) -> anyhow::Result<()> {
    let denial = match result {
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

fn rust_sources(root: &Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
    let mut sources = Vec::new();
    collect_rust_sources(root, &mut sources)?;
    sources.sort();
    Ok(sources)
}

fn collect_rust_sources(root: &Path, sources: &mut Vec<std::path::PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, sources)?;
        } else if path.extension() == Some(OsStr::new("rs")) {
            sources.push(path);
        }
    }
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

fn context_with_review(repo: &gix::Repository) -> anyhow::Result<but_ctx::Context> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    let head = ref_id(repo, FEAT_REF)?;
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: "https://github.com/gitbutler/registry-swap/pull/1".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Registry swap fixture".to_owned(),
            body: None,
            author: Some("dev".to_owned()),
            labels: "[]".to_owned(),
            draft: false,
            source_branch: "feat".to_owned(),
            target_branch: "main".to_owned(),
            sha: head.to_string(),
            created_at: None,
            modified_at: None,
            merged_at: None,
            closed_at: None,
            repository_ssh_url: None,
            repository_https_url: Some("https://github.com/gitbutler/registry-swap.git".to_owned()),
            repo_owner: Some("gitbutler".to_owned()),
            head_repo_is_fork: false,
            reviewers: "[]".to_owned(),
            unit_symbol: "#".to_owned(),
            last_sync_at: fixed_time(),
            struct_version: but_forge::ForgeReview::struct_version(),
        })?;
    Ok(ctx)
}

fn fixed_time() -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
