//! Command implementation for the committed agent roster + legacy migration.
//!
//! IDENT-006 wired this module into the top-level CLI dispatch. The runtime PID
//! registry was superseded by environment-primary identity (`BUT_AGENT_HANDLE`,
//! set by the trusted harness wrapper); the `register`/`unregister`/`whoami`
//! runtime verbs were removed with it. See `crates/but-authz/README.md` and
//! `.spec/prds/governance/12-uc-agent-identity.md`.

#![allow(dead_code)]

use std::{collections::BTreeSet, fs, path::Path};

use anyhow::Context as _;
use but_ctx::Context;
use serde::{Deserialize, Serialize};

use crate::{CliError, args::agent::Subcommands, bad_input, utils::OutputChannel};

const AGENTS_PATH: &str = ".gitbutler/agents.toml";
const PERMISSIONS_PATH: &str = ".gitbutler/permissions.toml";
const MIGRATE_CAVEAT: &str = "agents.toml written to the working tree; inert until committed. Commit the add of .gitbutler/agents.toml and the delete of .gitbutler/permissions.toml together.";

/// Execute `but agent`.
pub async fn exec(
    ctx: &mut Context,
    out: &mut OutputChannel,
    cmd: Option<Subcommands>,
) -> Result<(), CliError> {
    match cmd.unwrap_or(Subcommands::List { committed: true }) {
        Subcommands::List { committed: _ } => {
            // The committed roster is the only supported source; identity is
            // resolved from `BUT_AGENT_HANDLE` at the gate, not from a runtime
            // registry, so there are no live registrations to list.
            let roster = load_committed_agents(ctx)?;
            write_committed_list(out, &roster).map_err(CliError::from)
        }
        Subcommands::Migrate => migrate(ctx, out).map_err(CliError::from),
    }
}

fn migrate(ctx: &mut Context, out: &mut OutputChannel) -> anyhow::Result<()> {
    let repo = ctx.repo.get()?;
    let workdir = repo
        .workdir()
        .context("working tree is required to migrate .gitbutler/permissions.toml")?;
    let permissions_path = workdir.join(PERMISSIONS_PATH);
    let agents_path = workdir.join(AGENTS_PATH);

    match fs::metadata(&agents_path) {
        Ok(metadata) if metadata.len() > 0 => {
            return write_migrate_already_done(out);
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| format!("reading metadata for {AGENTS_PATH}"));
        }
    }

    let contents = fs::read_to_string(&permissions_path)
        .with_context(|| format!("reading {PERMISSIONS_PATH} from working tree"))?;
    let rewritten = but_authz::rewrite_principals_to_agents(&contents);
    fs::write(&agents_path, rewritten)
        .with_context(|| format!("writing {AGENTS_PATH} to working tree"))?;

    write_migrate_success(out)
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

fn write_migrate_success(out: &mut OutputChannel) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(
            out,
            "migrated: {PERMISSIONS_PATH} -> {AGENTS_PATH}; {MIGRATE_CAVEAT}"
        )?;
    } else if let Some(out) = out.for_json() {
        out.write_value(MigrateOutput {
            status: "migrated",
            source: Some(PERMISSIONS_PATH),
            destination: AGENTS_PATH,
            caveat: Some(MIGRATE_CAVEAT),
        })?;
    }
    Ok(())
}

fn write_migrate_already_done(out: &mut OutputChannel) -> anyhow::Result<()> {
    if let Some(out) = out.for_human_or_shell() {
        writeln!(
            out,
            "already migrated; no change: {AGENTS_PATH} already exists"
        )?;
    } else if let Some(out) = out.for_json() {
        out.write_value(MigrateOutput {
            status: "already_migrated",
            source: None,
            destination: AGENTS_PATH,
            caveat: None,
        })?;
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct MigrateOutput<'a> {
    status: &'a str,
    source: Option<&'a str>,
    destination: &'a str,
    caveat: Option<&'a str>,
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
