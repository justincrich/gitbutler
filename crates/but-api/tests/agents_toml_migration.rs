use but_api::commit::create::gate::{CommitGateTarget, enforce_commit_gate_for_target};
use but_authz::{Authority, PrincipalId, load_governance_config};
use serde::Serialize;

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";
const PERMISSIONS_FORMAT_REF: &str = "refs/heads/permissions-format";
const AGENTS_FORMAT_REF: &str = "refs/heads/agents-format";

#[test]
fn migrate_round_trip_is_byte_equivalent_gov_config() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let permissions_commit = ref_id(&repo, MAIN_REF)?;
    write_agents_toml_from_permissions_wire(&repo)?;

    but_testsupport::invoke_bash(
        r#"
git update-ref refs/heads/permissions-format HEAD
git add .gitbutler/agents.toml
git rm .gitbutler/permissions.toml
git commit -m "migrate governance agents"
git update-ref refs/heads/agents-format HEAD
"#,
        &repo,
    );
    let agents_commit = ref_id(&repo, AGENTS_FORMAT_REF)?;

    assert_ne!(
        permissions_commit, agents_commit,
        "migration round-trip must compare config loaded from two different commits"
    );

    let cfg_before = load_governance_config(&repo, PERMISSIONS_FORMAT_REF)?;
    let cfg_after = load_governance_config(&repo, AGENTS_FORMAT_REF)?;

    assert_eq!(
        cfg_before, cfg_after,
        "migration must be byte-equivalent at the GovConfig layer; [[principal]] -> [[agent]] is the only wire-level change"
    );
    assert_contents_write(&cfg_before, "dev")?;
    assert_contents_write(&cfg_after, "dev")?;
    assert_contents_read_only(&cfg_before, "ro")?;
    assert_contents_read_only(&cfg_after, "ro")?;

    Ok(())
}

#[test]
#[serial_test::serial]
fn legacy_permissions_only_repo_authorizes_via_env_fallback() -> anyhow::Result<()> {
    let (repo, _tmp) = legacy_permissions_only_repo();
    let target = CommitGateTarget::config_only(gix::refs::FullName::try_from(FEAT_REF)?);

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", Some("1")),
            ("BUT_AGENT_HANDLE", Some("dev")),
        ],
        || -> anyhow::Result<()> {
            enforce_commit_gate_for_target(&repo, &target)?;
            Ok(())
        },
    )?;

    temp_env::with_vars(
        [
            ("BUT_AUTHZ_ALLOW_ENV_HANDLE", None),
            ("BUT_AGENT_HANDLE", Some("dev")),
        ],
        || -> anyhow::Result<()> {
            let err = match enforce_commit_gate_for_target(&repo, &target) {
                Ok(()) => anyhow::bail!(
                    "legacy env fallback must deny when BUT_AUTHZ_ALLOW_ENV_HANDLE is unset"
                ),
                Err(err) => err,
            };
            let denial = err
                .downcast_ref::<but_authz::Denial>()
                .expect("env fallback rejection must be a structured authz denial");
            assert_eq!(
                denial.code, "perm.denied",
                "env fallback rejection must use the stable perm.denied code"
            );
            Ok(())
        },
    )?;

    Ok(())
}

fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("agents-toml-migration");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "permissions governance config"
"#,
        &repo,
    );
    (repo, tmp)
}

fn legacy_permissions_only_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("agents-toml-migration");
    but_testsupport::invoke_bash(
        r#"
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'
[[principal]]
id = "dev"
permissions = ["contents:write"]

[[principal]]
id = "ro"
permissions = ["contents:read"]
EOF

cat >.gitbutler/gates.toml <<'EOF'
[[branch]]
name = "main"
protected = true

[[branch]]
name = "feat"
protected = false
EOF

git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "legacy permissions governance config"
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

fn write_agents_toml_from_permissions_wire(repo: &gix::Repository) -> anyhow::Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must have a working tree"))?;
    let permissions_path = workdir.join(".gitbutler/permissions.toml");
    let permissions_text = std::fs::read_to_string(&permissions_path)?;
    let permissions = toml::from_str::<but_authz::PermissionsWire>(&permissions_text)?;
    let agents = AgentsWireForMigration::from(permissions);
    let agents_text = toml::to_string(&agents)?;

    std::fs::write(workdir.join(".gitbutler/agents.toml"), agents_text)?;
    Ok(())
}

#[derive(Serialize)]
struct AgentsWireForMigration {
    agent: Vec<AgentWireForMigration>,
    group: Vec<but_authz::GroupWire>,
}

impl From<but_authz::PermissionsWire> for AgentsWireForMigration {
    fn from(permissions: but_authz::PermissionsWire) -> Self {
        Self {
            agent: permissions
                .principal
                .into_iter()
                .map(AgentWireForMigration::from)
                .collect(),
            group: permissions.group,
        }
    }
}

#[derive(Serialize)]
struct AgentWireForMigration {
    id: String,
    permissions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<String>,
    groups: Vec<String>,
}

impl From<but_authz::PrincipalWire> for AgentWireForMigration {
    fn from(principal: but_authz::PrincipalWire) -> Self {
        Self {
            id: principal.id,
            permissions: principal.permissions,
            role: principal.role,
            groups: principal.groups,
        }
    }
}

fn assert_contents_write(config: &but_authz::GovConfig, principal_id: &str) -> anyhow::Result<()> {
    let authorities = config
        .principal_authorities(&PrincipalId::new(principal_id))
        .ok_or_else(|| {
            anyhow::anyhow!("{principal_id} principal must load from governance config")
        })?;
    assert!(
        authorities.contains(Authority::ContentsWrite),
        "{principal_id} must resolve to contents:write after migration round-trip"
    );
    Ok(())
}

fn assert_contents_read_only(
    config: &but_authz::GovConfig,
    principal_id: &str,
) -> anyhow::Result<()> {
    let authorities = config
        .principal_authorities(&PrincipalId::new(principal_id))
        .ok_or_else(|| {
            anyhow::anyhow!("{principal_id} principal must load from governance config")
        })?;
    assert!(
        authorities.contains(Authority::ContentsRead),
        "{principal_id} must resolve to contents:read after migration round-trip"
    );
    assert!(
        !authorities.contains(Authority::ContentsWrite),
        "{principal_id} must not gain contents:write during migration round-trip"
    );
    Ok(())
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
