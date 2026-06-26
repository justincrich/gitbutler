use std::{
    ffi::OsStr,
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use but_api::commit::create::gate::{CommitGateTarget, enforce_commit_gate_for_target};
use but_api::legacy::{
    config_mutate::enforce_administration_write_gate,
    forge::authorize_branch_action,
    governance::{
        branch_gates_read_with_repo, can_i_with_repo, governance_status_read, group_list_with_repo,
        perm_list_with_repo, whoami_with_repo,
    },
    merge_gate::enforce_merge_gate,
    rules::{create_workspace_rule, list_workspace_rules_scoped_for_caller},
};
use but_db::ForgeReview;
use but_rules::{Action, CreateRuleRequest, Filter, ImplicitOperation, Trigger, WorkspaceRule};

const FEAT_REF: &str = "refs/heads/feat";
const FEAT_REMOTE_REF: &str = "refs/remotes/origin/feat";
const REVIEW_ID: usize = 1;

#[test]
#[serial_test::serial]
fn commit_gate_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            perm_list_with_repo(&repo, FEAT_REF, None).map(|_| ())
        })
    })
}

#[test]
#[serial_test::serial]
fn governance_status_read_registered_then_unregistered_empty() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;
    let ctx = context_with_target_ref(&repo, FEAT_REMOTE_REF)?;

    with_registry_only(&registry_path, || {
        let registered = governance_status_read(&ctx)?;
        assert!(
            registered
                .authorities
                .iter()
                .any(|authority| authority == "contents:write"),
            "registered runtime process must resolve to the dev principal's real authorities"
        );
        assert!(
            !registered.not_configured,
            "governed target ref must report configured governance"
        );
        assert_eq!(registered.target_ref, FEAT_REMOTE_REF);

        write_process_registry(&registry_path, false)?;
        let unregistered = governance_status_read(&ctx)?;
        assert!(
            unregistered.authorities.is_empty(),
            "unregistered runtime process must get the governance status read-only empty-authority shape"
        );
        assert!(
            !unregistered.not_configured,
            "unregistered caller resolution must not masquerade as unconfigured governance"
        );
        assert_eq!(unregistered.target_ref, FEAT_REMOTE_REF);
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn governance_status_read_malformed_registry_propagates_instead_of_empty() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    fs::write(&registry_path, "not valid toml = [")?;
    let ctx = context_with_target_ref(&repo, FEAT_REMOTE_REF)?;

    with_registry_only(&registry_path, || {
        let error = governance_status_read(&ctx)
            .expect_err("malformed registry must not return empty governance status");
        assert!(
            error.downcast_ref::<but_authz::Denial>().is_none(),
            "malformed registry must propagate as registry corruption, not a permission denial"
        );
        let message = format!("{error:#}");
        assert!(
            message.contains("parsing registry") || message.contains("loading registry"),
            "malformed registry error must retain registry parse/load context, got: {message}"
        );
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn workspace_rules_scoped_for_caller_registered_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;
    let mut ctx = context_with_target_ref(&repo, FEAT_REMOTE_REF)?;
    let seeded = seed_workspace_rules(&mut ctx)?;

    with_registry_only(&registry_path, || {
        let registered = list_workspace_rules_scoped_for_caller(&ctx, Some("dev"))?;
        assert_eq!(
            rule_ids(&registered),
            vec![seeded.dev.id()],
            "registered runtime process must be allowed to read its own scoped rules"
        );

        write_process_registry(&registry_path, false)?;
        assert_perm_denied(list_workspace_rules_scoped_for_caller(&ctx, Some("dev")).map(|_| ()))
    })
}

#[test]
#[serial_test::serial]
fn admin_write_gate_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;
    let ctx = context_with_review(&repo)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || enforce_merge_gate(&ctx, REVIEW_ID))
    })
}

#[test]
#[serial_test::serial]
fn whoami_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            let outcome = whoami_with_repo(&repo, FEAT_REF, None)?;
            assert_eq!(outcome.principal, "dev");
            assert!(
                outcome
                    .authorities
                    .iter()
                    .any(|authority| authority == "contents:write"),
                "registered runtime process must receive its own effective authority set"
            );
            Ok(())
        })
    })
}

#[test]
#[serial_test::serial]
fn can_i_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, true)?;

    with_registry_only(&registry_path, || {
        registered_then_unregistered_denied(&registry_path, || {
            let outcome = can_i_with_repo(&repo, FEAT_REF, "contents:write", None)?;
            assert_eq!(outcome.principal, "dev");
            assert_eq!(outcome.authority, "contents:write");
            assert!(
                outcome.held,
                "registered runtime process must resolve before answering held=true"
            );
            Ok(())
        })
    })
}

#[test]
#[serial_test::serial]
fn forge_review_registered_process_allowed_then_unregistered_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, false)?;
    assert_eq!(
        but_authz::Registry::load(&registry_path)?.len(),
        0,
        "env fallback fixture must use an empty runtime registry"
    );

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
            })?;
            println!(
                "governed commit accepted BUT_AGENT_HANDLE=dev only with BUT_AUTHZ_ALLOW_ENV_HANDLE=1 and runtime registry has 0 entries"
            );
            Ok(())
        },
    )
}

#[test]
#[serial_test::serial]
fn merge_gate_env_only_without_flag_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    write_process_registry(&registry_path, false)?;
    let ctx = context_with_review(&repo)?;

    temp_env::with_vars(
        [
            ("BUT_AGENT_REGISTRY_PATH", Some(registry_path.as_os_str())),
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", None),
            ("BUT_AGENT_HANDLE", Some(OsStr::new("maint"))),
        ],
        || -> anyhow::Result<()> {
            let denial = assert_perm_denied_with_message(
                enforce_merge_gate(&ctx, REVIEW_ID),
                "merge env-only without flag",
            )?;
            assert!(
                denial.message.contains("pid ") && denial.message.contains("start_time"),
                "merge denial must identify the unregistered current process: {}",
                denial.message
            );
            println!(
                "merge env-only BUT_AGENT_HANDLE=maint without BUT_AUTHZ_ALLOW_ENV_HANDLE denied with literal `perm.denied`: {}",
                denial.message
            );
            Ok(())
        },
    )
}

#[test]
#[serial_test::serial]
fn expired_current_process_registry_entry_denied() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    let (pid, start_time) = write_expired_current_process_registry(&registry_path)?;

    with_registry_only(&registry_path, || {
        let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
        let denial = assert_perm_denied_with_message(
            enforce_commit_gate_for_target(&repo, &target),
            "expired current process registry entry",
        )?;
        assert!(
            denial.message.contains("pid ") && denial.message.contains("start_time"),
            "expired-entry denial must identify the current process: {}",
            denial.message
        );
        let loaded = but_authz::Registry::load(&registry_path)?;
        assert_eq!(
            loaded.len(),
            0,
            "runtime registry load path must prune expired entries before resolution"
        );
        println!(
            "expired entry denial code is literal `perm.denied`; runtime registry load path pruned pid={pid} start_time={start_time} before resolution"
        );
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn current_pid_wrong_start_time_denied_at_commit_gate() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    let (pid, observed_start_time, registered_start_time) =
        write_current_pid_wrong_start_time_registry(&registry_path)?;

    with_registry_only(&registry_path, || {
        let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
        let denial = assert_perm_denied_with_message(
            enforce_commit_gate_for_target(&repo, &target),
            "current pid wrong start_time",
        )?;
        assert!(
            denial.message.contains("pid ") && denial.message.contains("start_time"),
            "wrong-start-time denial must identify the current process: {}",
            denial.message
        );
        assert!(
            denial.message.contains(&observed_start_time.to_string())
                && denial.message.contains(&registered_start_time.to_string()),
            "wrong-start-time denial must name observed and registered start_time values: {}",
            denial.message
        );
        println!(
            "wrong-start-time denial code is literal `perm.denied`; registry entry key uses tuple ({pid}, {registered_start_time}) while current start_time is {observed_start_time}"
        );
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn wrong_pid_current_start_time_denied_at_commit_gate() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    let (current_pid, wrong_pid, current_start_time) =
        write_wrong_pid_current_start_time_registry(&registry_path)?;

    with_registry_only(&registry_path, || {
        let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
        let denial = assert_perm_denied_with_message(
            enforce_commit_gate_for_target(&repo, &target),
            "wrong pid current start_time",
        )?;
        assert!(
            denial.message.contains("pid ") && denial.message.contains("start_time"),
            "wrong-pid denial must identify the current process: {}",
            denial.message
        );
        assert!(
            denial.message.contains(&current_pid.to_string())
                && denial.message.contains(&current_start_time.to_string()),
            "wrong-pid denial must name the current pid/start_time, not the spoofed key: {}",
            denial.message
        );
        println!(
            "wrong-pid denial code is literal `perm.denied`; registry entry key uses tuple ({wrong_pid}, {current_start_time}) while current pid is {current_pid}"
        );
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn malformed_registry_propagates_instead_of_empty() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
    fs::write(&registry_path, "not valid toml = [")?;

    with_registry_only(&registry_path, || {
        let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);
        let error = enforce_commit_gate_for_target(&repo, &target)
            .expect_err("malformed registry must not be treated as an empty registry");
        assert!(
            error.downcast_ref::<but_authz::Denial>().is_none(),
            "malformed registry must propagate as registry corruption, not a permission denial"
        );
        let message = format!("{error:#}");
        assert!(
            message.contains("parsing registry"),
            "malformed registry error must retain the parser context, got: {message}"
        );
        Ok(())
    })
}

#[test]
#[serial_test::serial]
fn unreadable_registry_falls_through_to_structured_denial() -> anyhow::Result<()> {
    let (repo, tmp) = governed_repo();
    let registry_path = tmp.path().join("agents-runtime.toml");
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
    let registry_path = tmp.path().join("agents-runtime.toml");
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

#[test]
fn all_but_agent_handle_env_helpers_are_flag_gated() -> anyhow::Result<()> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let test_root = manifest_dir.join("tests");
    let mut violations = Vec::new();

    for path in rust_sources(&test_root)? {
        if is_dedicated_env_fallback_gate_test(&path) {
            continue;
        }
        let source = fs::read_to_string(&path)?;
        let relative = path.strip_prefix(manifest_dir)?.display().to_string();
        let helper_scopes = temp_env_helper_scopes(&source);

        for scope in helper_scopes
            .iter()
            .filter(|scope| scope.contains_agent_handle)
        {
            let paired = scope.contains_allow_env_handle
                || helper_scopes.iter().any(|candidate| {
                    candidate.contains_allow_env_handle
                        && candidate.start <= scope.start
                        && scope.end <= candidate.end
                });
            if !paired {
                violations.push(format!(
                    "{relative}:{} unpaired {helper} scope sets BUT_AGENT_HANDLE without BUT_AUTHZ_ALLOW_ENV_HANDLE=1",
                    scope.line,
                    helper = scope.helper
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "every temp_env BUT_AGENT_HANDLE helper scope must also enable BUT_AUTHZ_ALLOW_ENV_HANDLE=1:\n{}",
        violations.join("\n")
    );
    Ok(())
}

fn is_dedicated_env_fallback_gate_test(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(OsStr::to_str),
        Some("agents_toml_migration.rs" | "gate_registry_swap.rs")
    )
}

#[derive(Debug)]
struct TempEnvScope {
    helper: &'static str,
    start: usize,
    end: usize,
    line: usize,
    contains_agent_handle: bool,
    contains_allow_env_handle: bool,
}

fn temp_env_helper_scopes(source: &str) -> Vec<TempEnvScope> {
    const HELPERS: [(&str, &str); 3] = [
        ("temp_env::with_var(", "temp_env::with_var"),
        ("temp_env::with_vars(", "temp_env::with_vars"),
        ("temp_env::async_with_vars(", "temp_env::async_with_vars"),
    ];

    let mut scopes = Vec::new();
    for (needle, helper) in HELPERS {
        for start in helper_starts(source, needle) {
            let open_paren = start + needle.len() - 1;
            let Some(close_paren) = find_matching_paren(source, open_paren) else {
                break;
            };
            let scope_source = &source[start..=close_paren];
            scopes.push(TempEnvScope {
                helper,
                start,
                end: close_paren,
                line: source[..start].matches('\n').count() + 1,
                contains_agent_handle: scope_source.contains("\"BUT_AGENT_HANDLE\""),
                contains_allow_env_handle: scope_source.contains("\"BUT_AUTHZ_ALLOW_ENV_HANDLE\"")
                    && (scope_source.contains("Some(\"1\")")
                        || scope_source.contains("Some(OsStr::new(\"1\"))")),
            });
        }
    }
    scopes.sort_by_key(|scope| (scope.start, scope.end));
    scopes
}

fn helper_starts(source: &str, needle: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut starts = Vec::new();
    let mut idx = 0;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let byte = bytes[idx];
        let next = bytes.get(idx + 1).copied();

        if in_line_comment {
            in_line_comment = byte != b'\n';
            idx += 1;
            continue;
        }
        if in_block_comment {
            if byte == b'*' && next == Some(b'/') {
                in_block_comment = false;
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }
        if in_char {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\'' {
                in_char = false;
            }
            idx += 1;
            continue;
        }

        if bytes[idx..].starts_with(needle.as_bytes()) {
            starts.push(idx);
            idx += needle.len();
        } else if byte == b'/' && next == Some(b'/') {
            in_line_comment = true;
            idx += 2;
        } else if byte == b'/' && next == Some(b'*') {
            in_block_comment = true;
            idx += 2;
        } else if byte == b'"' {
            in_string = true;
            idx += 1;
        } else if byte == b'\'' {
            in_char = true;
            idx += 1;
        } else {
            idx += 1;
        }
    }

    starts
}

fn find_matching_paren(source: &str, open_paren: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut idx = open_paren;
    let mut depth = 0usize;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while idx < bytes.len() {
        let byte = bytes[idx];
        let next = bytes.get(idx + 1).copied();

        if in_line_comment {
            in_line_comment = byte != b'\n';
            idx += 1;
            continue;
        }
        if in_block_comment {
            if byte == b'*' && next == Some(b'/') {
                in_block_comment = false;
                idx += 2;
            } else {
                idx += 1;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            idx += 1;
            continue;
        }
        if in_char {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'\'' {
                in_char = false;
            }
            idx += 1;
            continue;
        }

        if byte == b'/' && next == Some(b'/') {
            in_line_comment = true;
            idx += 2;
        } else if byte == b'/' && next == Some(b'*') {
            in_block_comment = true;
            idx += 2;
        } else if byte == b'"' {
            in_string = true;
            idx += 1;
        } else if byte == b'\'' {
            in_char = true;
            idx += 1;
        } else if byte == b'(' {
            depth += 1;
            idx += 1;
        } else if byte == b')' {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(idx);
            }
            idx += 1;
        } else {
            idx += 1;
        }
    }
    None
}

struct SeededRules {
    dev: WorkspaceRule,
}

fn seed_workspace_rules(ctx: &mut but_ctx::Context) -> anyhow::Result<SeededRules> {
    Ok(SeededRules {
        dev: create_rule(ctx, Some("dev"))?,
    })
}

fn create_rule(
    ctx: &mut but_ctx::Context,
    session_id: Option<&str>,
) -> anyhow::Result<WorkspaceRule> {
    let mut filters = Vec::new();
    if let Some(session_id) = session_id {
        filters.push(Filter::ClaudeCodeSessionId(session_id.to_owned()));
    }

    create_workspace_rule(
        ctx,
        CreateRuleRequest {
            trigger: Trigger::ClaudeCodeHook,
            filters,
            action: Action::Implicit(ImplicitOperation::AssignToAppropriateBranch),
        },
    )
}

fn rule_ids(rules: &[WorkspaceRule]) -> Vec<String> {
    rules.iter().map(WorkspaceRule::id).collect()
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
    assert_perm_denied_with_message(result, "unregistered runtime process").map(|_| ())
}

fn assert_perm_denied_with_message(
    result: anyhow::Result<()>,
    context: &str,
) -> anyhow::Result<but_authz::Denial> {
    let denial = match result {
        Ok(()) => anyhow::bail!("{context} must be denied when env fallback is disabled"),
        Err(err) => err
            .downcast::<but_authz::Denial>()
            .map_err(|err| anyhow::anyhow!("{context} denial should be structured: {err:#}"))?,
    };

    assert_eq!(
        denial.code, "perm.denied",
        "{context} must deny with the stable perm.denied code"
    );
    Ok(denial)
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

fn write_expired_current_process_registry(path: &Path) -> anyhow::Result<(u32, u64)> {
    let mut registry = but_authz::Registry::empty();
    let pid = but_authz::current_pid();
    let start_time = but_authz::process_start_time(pid)?;
    registry.register(pid, start_time, "dev", 0, "operator")?;
    registry.write(path)?;
    wait_until_after(start_time)?;
    Ok((pid, start_time))
}

fn write_current_pid_wrong_start_time_registry(path: &Path) -> anyhow::Result<(u32, u64, u64)> {
    let mut registry = but_authz::Registry::empty();
    let pid = but_authz::current_pid();
    let observed_start_time = but_authz::process_start_time(pid)?;
    let registered_start_time = observed_start_time.checked_add(1).ok_or_else(|| {
        anyhow::anyhow!("cannot construct wrong start_time for pid {pid}: u64 overflow")
    })?;
    registry.register(pid, registered_start_time, "dev", 14_400, "operator")?;
    registry.write(path)?;
    Ok((pid, observed_start_time, registered_start_time))
}

fn write_wrong_pid_current_start_time_registry(path: &Path) -> anyhow::Result<(u32, u32, u64)> {
    let mut registry = but_authz::Registry::empty();
    let current_pid = but_authz::current_pid();
    let wrong_pid = current_pid
        .checked_add(1)
        .or_else(|| current_pid.checked_sub(1))
        .ok_or_else(|| anyhow::anyhow!("cannot construct wrong pid for {current_pid}"))?;
    let current_start_time = but_authz::process_start_time(current_pid)?;
    registry.register(wrong_pid, current_start_time, "dev", 14_400, "operator")?;
    registry.write(path)?;
    Ok((current_pid, wrong_pid, current_start_time))
}

fn wait_until_after(timestamp: u64) -> anyhow::Result<()> {
    while unix_time_seconds()? <= timestamp {
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

fn unix_time_seconds() -> anyhow::Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
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

[[principal]]
id = "maint"
permissions = ["merge"]
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
git update-ref refs/remotes/origin/feat refs/heads/feat
git checkout main
"#,
        &repo,
    );
    (repo, tmp)
}

fn context_with_target_ref(
    repo: &gix::Repository,
    target_ref: &str,
) -> anyhow::Result<but_ctx::Context> {
    let ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    let mut project_meta = ctx.project_meta()?;
    project_meta.target_ref = Some(target_ref.try_into()?);
    project_meta.target_commit_id = Some(ref_id(repo, target_ref)?);
    ctx.set_project_meta(project_meta)?;
    Ok(ctx)
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
