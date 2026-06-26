//! Command implementation for runtime agent registration.
//!
//! IDENT-006 wires this module into the top-level CLI dispatch.

#![allow(dead_code)]

use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use but_ctx::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{CliError, args::agent::Subcommands, bad_input, utils::OutputChannel};

const AGENTS_PATH: &str = ".gitbutler/agents.toml";
const REGISTRY_FILE_NAME: &str = "agents-runtime.toml";

/// Execute `but agent`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    cmd: Option<Subcommands>,
) -> Result<(), CliError> {
    match cmd.unwrap_or(Subcommands::List { committed: false }) {
        Subcommands::Register {
            pid,
            start_time,
            r#as,
            ttl,
            by,
        } => {
            let (pid, start_time) = resolve_process_key(pid, start_time)?;
            let roster = load_committed_agents(ctx)?;
            if !roster.contains(r#as.as_str()) {
                return Err(bad_input(format!("unknown agent_id: {as}", as = r#as))
                    .arg_name("--as")
                    .arg_value(r#as)
                    .into());
            }

            let registry_path = resolve_registry_path(ctx).map_err(CliError::from)?;
            let mut registry =
                but_authz::Registry::load(&registry_path.path).map_err(CliError::from)?;
            registry
                .register(pid, start_time, r#as.clone(), ttl, by)
                .map_err(CliError::from)?;
            let expires_at = start_time.checked_add(ttl).ok_or_else(|| {
                CliError::from(anyhow::anyhow!("registration expires_at overflow"))
            })?;
            write_registry_or_exit(&registry, &registry_path);
            write_registration(out, "registered", pid, start_time, &r#as, expires_at)
                .map_err(CliError::from)
        }
        Subcommands::Unregister { pid, start_time } => {
            let registry_path = resolve_registry_path(ctx).map_err(CliError::from)?;
            let mut registry =
                but_authz::Registry::load(&registry_path.path).map_err(CliError::from)?;
            let removed = match start_time {
                Some(start_time) => registry
                    .unregister((pid, start_time))
                    .map(|registration| RemovedRegistration {
                        pid,
                        start_time,
                        agent_id: registration.agent_id.as_str().to_owned(),
                        expires_at: registration.expires_at,
                    })
                    .into_iter()
                    .collect::<Vec<_>>(),
                None => registry
                    .registrations()
                    .iter()
                    .filter_map(|(&(entry_pid, entry_start_time), registration)| {
                        (entry_pid == pid).then(|| RemovedRegistration {
                            pid: entry_pid,
                            start_time: entry_start_time,
                            agent_id: registration.agent_id.as_str().to_owned(),
                            expires_at: registration.expires_at,
                        })
                    })
                    .collect::<Vec<_>>(),
            };

            for removed in &removed {
                registry.unregister((removed.pid, removed.start_time));
            }

            if !removed.is_empty() {
                write_registry_or_exit(&registry, &registry_path);
            }
            write_unregister(out, pid, start_time, &removed).map_err(CliError::from)
        }
        Subcommands::List { committed: true } => {
            let roster = load_committed_agents(ctx)?;
            write_committed_list(out, &roster).map_err(CliError::from)
        }
        Subcommands::List { committed: false } => {
            let registry_path = resolve_registry_path(ctx).map_err(CliError::from)?;
            let registry =
                but_authz::Registry::load(&registry_path.path).map_err(CliError::from)?;
            let live = registry
                .registrations()
                .iter()
                .map(|(&(pid, start_time), registration)| LiveRegistration {
                    pid,
                    start_time,
                    agent_id: registration.agent_id.as_str().to_owned(),
                    registered_at: registration.registered_at,
                    expires_at: registration.expires_at,
                    registered_by: registration.registered_by.as_str().to_owned(),
                })
                .collect::<Vec<_>>();
            write_live_list(out, &live).map_err(CliError::from)
        }
        Subcommands::Whoami => {
            let pid = but_authz::current_pid();
            let start_time = but_authz::process_start_time(pid).map_err(CliError::from)?;
            let registry_path = resolve_registry_path(ctx).map_err(CliError::from)?;
            let registry =
                but_authz::Registry::load(&registry_path.path).map_err(CliError::from)?;
            let Some(agent_id) = registry.resolve((pid, start_time)) else {
                return Err(bad_input(format!(
                    "no agent registration for pid {pid} start_time {start_time}"
                ))
                .into());
            };
            write_whoami(out, pid, start_time, agent_id.as_str()).map_err(CliError::from)
        }
    }
}

fn resolve_process_key(pid: Option<u32>, start_time: Option<u64>) -> Result<(u32, u64), CliError> {
    let current_pid = but_authz::current_pid();
    let pid = pid.unwrap_or(current_pid);
    let start_time = match start_time {
        Some(start_time) => start_time,
        None if pid == current_pid => but_authz::process_start_time(pid).map_err(CliError::from)?,
        None => {
            return Err(bad_input(format!(
                "--start-time is required when registering another process pid {pid}"
            ))
            .arg_name("--start-time")
            .hint("pass the target process start time as Unix seconds")
            .into());
        }
    };
    Ok((pid, start_time))
}

fn resolve_registry_path(ctx: &mut Context) -> anyhow::Result<RegistryPath> {
    if let Some(path) = env::var_os("BUT_AGENT_REGISTRY_PATH") {
        let path = PathBuf::from(path);
        tracing::debug!(path = %path.display(), "using BUT_AGENT_REGISTRY_PATH for agent registry");
        return Ok(RegistryPath {
            path,
            create_parent: false,
        });
    }

    let repo = ctx.repo.get()?;
    let repo_hash = repo_hash(&repo)?;
    if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir)
            .join("gitbutler")
            .join(repo_hash)
            .join(REGISTRY_FILE_NAME);
        tracing::debug!(path = %path.display(), "using XDG_RUNTIME_DIR agent registry path");
        return Ok(RegistryPath {
            path,
            create_parent: true,
        });
    }

    let workdir = repo
        .workdir()
        .context("worktree is required for default agent registry path without XDG_RUNTIME_DIR")?;
    let path = workdir.join(".gitbutler").join(REGISTRY_FILE_NAME);
    tracing::debug!(path = %path.display(), "using worktree agent registry path");
    Ok(RegistryPath {
        path,
        create_parent: true,
    })
}

fn repo_hash(repo: &gix::Repository) -> anyhow::Result<String> {
    let path = gix::path::realpath(repo.git_dir()).unwrap_or_else(|_| repo.git_dir().to_owned());
    let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());
    Ok(digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn write_registry_or_exit(registry: &but_authz::Registry, registry_path: &RegistryPath) {
    if registry_path.create_parent
        && let Some(parent) = registry_path.path.parent()
        && let Err(error) = fs::create_dir_all(parent)
    {
        exit_registry_write_error(&registry_path.path, error.into());
    }

    if let Err(error) = registry.write(&registry_path.path) {
        exit_registry_write_error(&registry_path.path, error);
    }
}

fn exit_registry_write_error(path: &Path, error: anyhow::Error) -> ! {
    eprintln!(
        "failed to write agent registry {}: {error:#}",
        path.display()
    );
    std::process::exit(2);
}

fn load_committed_agents(ctx: &mut Context) -> Result<CommittedAgents, CliError> {
    let target_ref = resolve_target_ref(ctx).map_err(CliError::from)?;
    let repo = ctx.repo.get().map_err(CliError::from)?;
    let content = read_committed_blob(&repo, &target_ref, AGENTS_PATH).map_err(|err| {
        bad_input(format!(
            "invalid committed {AGENTS_PATH} at {target_ref}: {err:#}"
        ))
    })?;
    let file = toml::from_str::<AgentsFile>(&content).map_err(|err| {
        bad_input(format!(
            "invalid committed {AGENTS_PATH} at {target_ref}: {err}"
        ))
    })?;
    CommittedAgents::from_file(file).map_err(|err| {
        bad_input(format!(
            "invalid committed {AGENTS_PATH} at {target_ref}: {err:#}"
        ))
        .into()
    })
}

fn resolve_target_ref(ctx: &mut Context) -> anyhow::Result<String> {
    let target = {
        let mut guard = ctx.exclusive_worktree_access();
        but_api::legacy::virtual_branches::get_base_branch_data(ctx, guard.write_permission())?
    };
    let target = target.context("target ref is required to load committed agents")?;
    let repo = ctx.repo.get()?;
    for candidate in target_ref_candidates(&target) {
        if repo.try_find_reference(&candidate)?.is_some() {
            return Ok(candidate);
        }
    }
    Ok(target.branch_name)
}

fn target_ref_candidates(target: &gitbutler_branch_actions::BaseBranch) -> Vec<String> {
    let mut candidates = Vec::new();
    if !target.short_name.is_empty() {
        candidates.push(format!("refs/heads/{}", target.short_name));
    }
    candidates.push(target.branch_name.clone());
    if !target.branch_name.starts_with("refs/") {
        candidates.push(format!("refs/remotes/{}", target.branch_name));
        candidates.push(format!("refs/heads/{}", target.branch_name));
    }
    candidates
}

fn read_committed_blob(
    repo: &gix::Repository,
    target_ref: &str,
    path: &str,
) -> anyhow::Result<String> {
    let mut reference = repo
        .find_reference(target_ref)
        .with_context(|| format!("resolving target ref {target_ref}"))?;
    let commit = reference
        .peel_to_commit()
        .with_context(|| format!("peeling {target_ref} to a commit"))?;
    let tree = commit
        .tree()
        .with_context(|| format!("reading tree for {target_ref}"))?;
    let entry = tree
        .lookup_entry_by_path(Path::new(path))
        .with_context(|| format!("looking up {path} in {target_ref}"))?
        .ok_or_else(|| anyhow::anyhow!("missing {path} at {target_ref}"))?;
    let blob = repo
        .find_blob(entry.id())
        .with_context(|| format!("reading {path} blob at {target_ref}"))?;
    let content = std::str::from_utf8(&blob.data)
        .with_context(|| format!("decoding {path} at {target_ref} as UTF-8"))?;
    Ok(content.to_owned())
}

fn write_registration(
    out: &mut OutputChannel,
    verb: &str,
    pid: u32,
    start_time: u64,
    agent_id: &str,
    expires_at: u64,
) -> anyhow::Result<()> {
    let registration = RegistrationOutput {
        pid,
        start_time,
        agent_id,
        expires_at,
    };
    if let Some(out) = out.for_human_or_shell() {
        writeln!(
            out,
            "{verb}: pid={pid} start_time={start_time} agent_id={agent_id} expires_at={expires_at}"
        )?;
    } else if let Some(out) = out.for_json() {
        out.write_value(registration)?;
    }
    Ok(())
}

fn write_unregister(
    out: &mut OutputChannel,
    pid: u32,
    start_time: Option<u64>,
    removed: &[RemovedRegistration],
) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        if removed.is_empty() {
            match start_time {
                Some(start_time) => writeln!(out, "not found: pid={pid} start_time={start_time}")?,
                None => writeln!(out, "not found: pid={pid}")?,
            }
        } else if let Some(start_time) = start_time {
            writeln!(out, "removed: pid={pid} start_time={start_time}")?;
        } else {
            writeln!(out, "removed: pid={pid} count={}", removed.len())?;
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(UnregisterOutput {
            pid,
            start_time,
            removed_count: removed.len(),
            removed,
        })?;
    }
    Ok(())
}

fn write_committed_list(out: &mut OutputChannel, roster: &CommittedAgents) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        for agent_id in &roster.ids {
            writeln!(out, "{agent_id}")?;
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(roster)?;
    }
    Ok(())
}

fn write_live_list(out: &mut OutputChannel, live: &[LiveRegistration]) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        for registration in live {
            writeln!(
                out,
                "pid={} start_time={} agent_id={} expires_at={}",
                registration.pid,
                registration.start_time,
                registration.agent_id,
                registration.expires_at
            )?;
        }
    } else if let Some(out) = out.for_json() {
        out.write_value(LiveListOutput {
            registrations: live,
        })?;
    }
    Ok(())
}

fn write_whoami(
    out: &mut OutputChannel,
    pid: u32,
    start_time: u64,
    agent_id: &str,
) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(out, "{agent_id}")?;
    } else if let Some(out) = out.for_json() {
        out.write_value(WhoamiOutput {
            pid,
            start_time,
            agent_id,
        })?;
    }
    Ok(())
}

#[derive(Debug)]
struct RegistryPath {
    path: PathBuf,
    create_parent: bool,
}

#[derive(Debug, Serialize)]
struct RegistrationOutput<'a> {
    pid: u32,
    start_time: u64,
    agent_id: &'a str,
    expires_at: u64,
}

#[derive(Debug, Serialize)]
struct RemovedRegistration {
    pid: u32,
    start_time: u64,
    agent_id: String,
    expires_at: u64,
}

#[derive(Debug, Serialize)]
struct UnregisterOutput<'a> {
    pid: u32,
    start_time: Option<u64>,
    removed_count: usize,
    removed: &'a [RemovedRegistration],
}

#[derive(Debug, Serialize)]
struct LiveRegistration {
    pid: u32,
    start_time: u64,
    agent_id: String,
    registered_at: u64,
    expires_at: u64,
    registered_by: String,
}

#[derive(Debug, Serialize)]
struct LiveListOutput<'a> {
    registrations: &'a [LiveRegistration],
}

#[derive(Debug, Serialize)]
struct WhoamiOutput<'a> {
    pid: u32,
    start_time: u64,
    agent_id: &'a str,
}

#[derive(Debug, Serialize)]
struct CommittedAgents {
    ids: Vec<String>,
}

impl CommittedAgents {
    fn from_file(file: AgentsFile) -> anyhow::Result<Self> {
        let mut ids = BTreeSet::new();
        for agent in file.agent {
            if agent.id.trim().is_empty() {
                anyhow::bail!("agent id must not be empty");
            }
            if !ids.insert(agent.id.clone()) {
                anyhow::bail!("duplicate agent id {}", agent.id);
            }
        }
        Ok(Self {
            ids: ids.into_iter().collect(),
        })
    }

    fn contains(&self, id: &str) -> bool {
        self.ids.iter().any(|known| known == id)
    }
}

#[derive(Debug, Deserialize)]
struct AgentsFile {
    #[serde(default)]
    agent: Vec<AgentWire>,
}

#[derive(Debug, Deserialize)]
struct AgentWire {
    id: String,
}
