//! STEER-007 — denial-steering telemetry event integration proofs.
//!
//! Proves the structured `tracing` event fires once per governed denial,
//! carrying the four aggregate metrics (`code`, `class`, `had_lateral_action`,
//! `menu_length`) operators use to measure whether steering reduces
//! hard-quits and loops. Captured from a REAL `tracing_subscriber` layer
//! installed in-process — no mock.
//!
//! See `.spec/prds/governance/tasks/sprint-08-steer-capability-aware-denials/
//! STEER-007-denial-steering-telemetry.md` for the task contract.

use std::sync::{Arc, Mutex};

use but_core::{DiffSpec, DryRun};
use but_rebase::graph_rebase::mutate::{InsertSide, RelativeTo};
use tracing::Subscriber;
use tracing::field::{Field, Visit};
use tracing_subscriber::{
    Layer,
    layer::{Context, SubscriberExt},
};

const MAIN_REF: &str = "refs/heads/main";
const FEAT_REF: &str = "refs/heads/feat";

// ---------------------------------------------------------------------------
// Capturing subscriber layer — the REAL `tracing_subscriber` sink the daemon
// and CLI use (crates/but/src/trace.rs), installed in-process for the test.
// ---------------------------------------------------------------------------

/// A single captured `tracing` event with its named fields extracted to
/// `(name, value_string)` pairs.
#[derive(Debug, Clone, Default)]
struct CapturedEvent {
    fields: Vec<(String, String)>,
}

impl CapturedEvent {
    /// Return the recorded value for the named field, if any.
    ///
    /// `tracing` records `&str` field values via `record_str`, but a `Display`
    /// shorthand (`field = %value`) falls back to `record_debug` (which adds
    /// surrounding `"`). Both forms are normalized here so callers compare
    /// against the raw value.
    fn field(&self, name: &str) -> Option<String> {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| normalize_captured_value(v))
    }
}

/// Strip the surrounding `"` added by `record_debug` so a `&str` recorded via
/// `record_str` and one recorded via the `%` Display shorthand compare equal.
fn normalize_captured_value(v: &str) -> String {
    if v.len() >= 2 && v.starts_with('"') && v.ends_with('"') {
        v[1..v.len() - 1].to_owned()
    } else {
        v.to_owned()
    }
}

/// Thread-safe sink of captured events shared between the layer and the test.
#[derive(Default, Debug)]
struct CapturingSink {
    events: Mutex<Vec<CapturedEvent>>,
}

/// A `tracing_subscriber` `Layer` that records every event's named fields
/// into a [`CapturingSink`].
struct CapturingLayer {
    sink: Arc<CapturingSink>,
}

impl CapturingLayer {
    fn new(sink: Arc<CapturingSink>) -> Self {
        Self { sink }
    }
}

impl<S> Layer<S> for CapturingLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = FieldCollector::default();
        event.record(&mut visitor);
        if let Ok(mut events) = self.sink.events.lock() {
            events.push(CapturedEvent {
                fields: visitor.fields,
            });
        }
    }
}

/// `tracing::field::Visit` collector that records every field as a
/// `(name, stringified_value)` pair. Covers the five `record_*` variants
/// tracing emits for `&str`/`bool`/`usize`/`Display`/`Debug` field values.
#[derive(Default)]
struct FieldCollector {
    fields: Vec<(String, String)>,
}

impl Visit for FieldCollector {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.fields
            .push((field.name().to_owned(), value.to_owned()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .push((field.name().to_owned(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .push((field.name().to_owned(), value.to_string()));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .push((field.name().to_owned(), value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .push((field.name().to_owned(), format!("{value:?}")));
    }
}

/// Install a capturing subscriber for the duration of `f`, returning every
/// event captured on the current thread.
fn capture_events<R>(f: impl FnOnce() -> R) -> (R, Vec<CapturedEvent>) {
    let sink = Arc::new(CapturingSink::default());
    let subscriber = tracing_subscriber::registry().with(CapturingLayer::new(sink.clone()));
    let result = tracing::subscriber::with_default(subscriber, f);
    let events = sink
        .events
        .lock()
        .map(|mut guard| std::mem::take(&mut *guard))
        .unwrap_or_default();
    (result, events)
}

/// Filter captured events to those whose `message` field (or any field's
/// value) names "denial steering telemetry" — the stable event label emitted
/// at the denial-payload construction boundary.
fn denial_steering_events(events: &[CapturedEvent]) -> Vec<CapturedEvent> {
    events
        .iter()
        .filter(|event| {
            event.fields.iter().any(|(_, v)| {
                let normalized = normalize_captured_value(v);
                normalized.contains("denial steering telemetry")
            })
        })
        .cloned()
        .collect()
}

// ---------------------------------------------------------------------------
// AC-1 / TC-1 — Actor-correctable denial emits an event with all four fields
// ---------------------------------------------------------------------------

/// AC-1 / TC-1 — an actor-correctable denial (`branch.protected` for a holder
/// of `contents:write`) emits exactly one denial-steering event carrying
/// `code="branch.protected"`, `class="actor_correctable"`,
/// `had_lateral_action=true`, and `menu_length>=1`.
///
/// GIVEN the `governed_repo` fixture and a capturing tracing layer; WHEN the
/// `dev` principal (holds `contents:write`) attempts a direct commit to
/// protected `main`, THEN the captured tracing events contain exactly one
/// denial-steering event with the four required fields populated.
#[test]
#[serial_test::serial]
fn steer_telemetry_actor_correctable_event_fields() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let (gate_result, events) = temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || {
        capture_events(|| -> anyhow::Result<()> {
            checkout(&repo, "main");
            write_file(&repo, "main.txt", "main\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            match commit_to_ref(&mut ctx, MAIN_REF, "direct main commit", DryRun::No) {
                Ok(_) => anyhow::bail!("protected main direct commit should be denied"),
                Err(err) => {
                    assert_eq!(
                        but_api::commit::create::gate::classify_error(&err)
                            .expect("denial should classify")
                            .code,
                        "branch.protected",
                        "branch.protected code must remain unchanged"
                    );
                }
            }
            Ok(())
        })
    });
    gate_result?;

    let steering_events = denial_steering_events(&events);
    assert_eq!(
        steering_events.len(),
        1,
        "exactly one denial-steering event must fire per governed denial (got {}): {:?}",
        steering_events.len(),
        events,
    );

    let event = &steering_events[0];
    let code = event
        .field("code")
        .expect("event must carry a `code` field");
    assert!(!code.is_empty(), "`code` must be non-empty (got {code:?})");
    assert_eq!(
        code, "branch.protected",
        "`code` must be the stable denial code"
    );

    let class = event
        .field("class")
        .expect("event must carry a `class` field");
    assert_eq!(
        class, "actor_correctable",
        "`class` must be the stable snake_case DenialClass name (got {class:?})"
    );

    let had_lateral_action = event
        .field("had_lateral_action")
        .expect("event must carry a `had_lateral_action` field");
    assert_eq!(
        had_lateral_action, "true",
        "`had_lateral_action` must be true for a menu that offers a lateral move (got {had_lateral_action:?})"
    );

    let menu_length: usize = event
        .field("menu_length")
        .expect("event must carry a `menu_length` field")
        .parse()
        .expect("`menu_length` must parse as a usize");
    assert!(
        menu_length >= 1,
        "`menu_length` must be >= 1 for an actor-correctable denial that has a lateral menu (got {menu_length})"
    );

    // Observation-only: the ref must NOT advance.
    assert_eq!(
        ref_id(&repo, MAIN_REF)?,
        main_before,
        "branch.protected must leave main unchanged (event is observation-only)"
    );

    println!(
        "AC-1: branch.protected → code={code}, class={class}, had_lateral_action={had_lateral_action}, menu_length={menu_length}"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// AC-2 / TC-2 — Operator-required (config.invalid) emits empty menu
// ---------------------------------------------------------------------------

/// AC-2 / TC-2 — an operator-required `config.invalid` denial (malformed
/// committed `gates.toml`) emits exactly one denial-steering event with
/// `class="operator_required"`, `had_lateral_action=false`, and
/// `menu_length=0`.
///
/// GIVEN the `governed_repo` fixture mutated to commit a malformed
/// `gates.toml` at the target ref; WHEN a gated action runs against the
/// malformed config, THEN the captured event carries `operator_required`,
/// `had_lateral_action=false`, and `menu_length=0`.
#[test]
#[serial_test::serial]
fn steer_telemetry_operator_required_empty_menu() -> anyhow::Result<()> {
    let (gate_result, events) = temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || {
        capture_events(|| -> anyhow::Result<()> {
            let (repo, _tmp) = governed_repo();
            checkout(&repo, "feat");
            but_testsupport::invoke_bash(
                r#"
cat >.gitbutler/gates.toml <<'EOF'
[[branch]
name = "feat"
protected = nope
EOF
git add .gitbutler/gates.toml
git commit -m "malformed feat gates"
"#,
                &repo,
            );
            write_file(&repo, "malformed.txt", "malformed\n")?;
            let feat_before = ref_id(&repo, FEAT_REF)?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            match commit_to_ref(&mut ctx, FEAT_REF, "malformed config", DryRun::No) {
                Ok(_) => anyhow::bail!("malformed-config commit should be denied"),
                Err(err) => {
                    assert_eq!(
                        but_api::commit::create::gate::classify_error(&err)
                            .expect("config.invalid should classify")
                            .code,
                        "config.invalid",
                        "config.invalid code must be surfaced"
                    );
                }
            }
            assert_eq!(
                ref_id(&repo, FEAT_REF)?,
                feat_before,
                "config.invalid must leave feat unchanged"
            );
            Ok(())
        })
    });
    gate_result?;

    let steering_events = denial_steering_events(&events);
    assert_eq!(
        steering_events.len(),
        1,
        "exactly one denial-steering event must fire on the config.invalid path (got {})",
        steering_events.len(),
    );

    let event = &steering_events[0];
    assert_eq!(
        event.field("code").as_deref(),
        Some("config.invalid"),
        "config.invalid event must carry the stable code"
    );
    assert_eq!(
        event.field("class").as_deref(),
        Some("operator_required"),
        "config.invalid MUST be operator_required"
    );
    assert_eq!(
        event.field("had_lateral_action").as_deref(),
        Some("false"),
        "config.invalid MUST set had_lateral_action=false (no lateral action)"
    );
    let menu_length: usize = event
        .field("menu_length")
        .expect("config.invalid event must carry `menu_length`")
        .parse()
        .expect("`menu_length` must parse as usize");
    assert_eq!(
        menu_length, 0,
        "config.invalid MUST set menu_length=0 (empty menu)"
    );

    println!("AC-2: config.invalid → operator_required, had_lateral_action=false, menu_length=0");

    Ok(())
}

// ---------------------------------------------------------------------------
// AC-3 / TC-3 — Discovery-only menu sets had_lateral_action=false
// ---------------------------------------------------------------------------

/// AC-3 / TC-3 — a denial whose menu contains ONLY the discovery affordance
/// (`but perm list`) emits `had_lateral_action=false` with `menu_length=1`,
/// proving the metric distinguishes a real lateral move from the always-
/// appended discovery entry.
///
/// GIVEN the `ro` principal (holds only `contents:read`) is denied a commit
/// to feat, WHEN the denial is produced, THEN the event carries
/// `had_lateral_action=false` while `menu_length=1` (the discovery entry
/// only).
#[test]
#[serial_test::serial]
fn steer_telemetry_discovery_only_no_lateral() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let feat_before = ref_id(&repo, FEAT_REF)?;

    let (gate_result, events) = temp_env::with_var("BUT_AGENT_HANDLE", Some("ro"), || {
        capture_events(|| -> anyhow::Result<()> {
            checkout(&repo, "feat");
            write_file(&repo, "ro-discovery.txt", "ro\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            match commit_to_ref(&mut ctx, FEAT_REF, "ro discovery-only", DryRun::No) {
                Ok(_) => anyhow::bail!("ro commit should be denied with perm.denied"),
                Err(err) => {
                    let gate_error = but_api::commit::create::gate::classify_error(&err)
                        .expect("perm.denied should classify");
                    assert_eq!(
                        gate_error.code, "perm.denied",
                        "ro commit MUST deny with perm.denied"
                    );
                    // Sanity: the menu really is discovery-only so the AC-3
                    // assertion below is meaningful (proves the metric is not
                    // merely `menu_length > 0`).
                    let commands: Vec<&str> = gate_error
                        .authorized_actions
                        .iter()
                        .map(|a| a.command)
                        .collect();
                    assert_eq!(
                        commands,
                        &["but perm list"],
                        "AC-3 setup: ro menu MUST be discovery-only (got {commands:?})"
                    );
                }
            }
            Ok(())
        })
    });
    gate_result?;

    assert_eq!(
        ref_id(&repo, FEAT_REF)?,
        feat_before,
        "ro denial must leave feat unchanged"
    );

    let steering_events = denial_steering_events(&events);
    assert_eq!(
        steering_events.len(),
        1,
        "exactly one denial-steering event must fire on the ro perm.denied path (got {})",
        steering_events.len(),
    );

    let event = &steering_events[0];
    assert_eq!(
        event.field("had_lateral_action").as_deref(),
        Some("false"),
        "discovery-only menu MUST set had_lateral_action=false (the discovery entry is NOT a lateral action)"
    );
    let menu_length: usize = event
        .field("menu_length")
        .expect("discovery-only event must carry `menu_length`")
        .parse()
        .expect("`menu_length` must parse as usize");
    assert_eq!(
        menu_length, 1,
        "discovery-only menu MUST set menu_length=1 (just the discovery entry)"
    );

    println!("AC-3: discovery-only menu → had_lateral_action=false, menu_length=1");

    Ok(())
}

// ---------------------------------------------------------------------------
// AC-4 / TC-4 — Event is observation-only (no deny→allow flip)
// ---------------------------------------------------------------------------

/// AC-4 / TC-4 — the telemetry event does NOT alter the deny/allow decision
/// or advance the ref. The denial still returns the same stable code and
/// `refs/heads/main` is identical before and after.
///
/// GIVEN the `governed_repo` fixture and the `dev` principal attempting a
/// commit to protected main; WHEN the gate denies with the event emitted,
/// THEN `classify_error` still returns `branch.protected` and the `main` ref
/// is unchanged.
#[test]
#[serial_test::serial]
fn steer_telemetry_event_is_observation_only() -> anyhow::Result<()> {
    let (repo, _tmp) = governed_repo();
    let main_before = ref_id(&repo, MAIN_REF)?;

    let (gate_result, events) = temp_env::with_var("BUT_AGENT_HANDLE", Some("dev"), || {
        capture_events(|| -> anyhow::Result<()> {
            checkout(&repo, "main");
            write_file(&repo, "observation-only.txt", "denied\n")?;
            let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
            let result = commit_to_ref(&mut ctx, MAIN_REF, "observation only", DryRun::No);
            let err = match result {
                Ok(_) => anyhow::bail!(
                    "commit to protected main MUST be denied (no deny->allow flip from telemetry)"
                ),
                Err(err) => err,
            };
            let gate_error = but_api::commit::create::gate::classify_error(&err)
                .expect("denial must still classify after telemetry fires");
            assert_eq!(
                gate_error.code, "branch.protected",
                "telemetry MUST NOT change the stable denial code"
            );
            assert_eq!(
                gate_error.class,
                but_authz::DenialClass::ActorCorrectable,
                "telemetry MUST NOT change the class"
            );
            assert!(
                !gate_error.authorized_actions.is_empty(),
                "telemetry MUST NOT drop the authorized_actions menu"
            );
            Ok(())
        })
    });
    gate_result?;

    // The ref must NOT advance — observation-only.
    let main_after = ref_id(&repo, MAIN_REF)?;
    assert_eq!(
        main_after, main_before,
        "telemetry MUST NOT advance the ref (observation-only): before={main_before} after={main_after}"
    );

    // The event MUST have fired exactly once.
    let steering_events = denial_steering_events(&events);
    assert_eq!(
        steering_events.len(),
        1,
        "exactly one denial-steering event must fire on the branch.protected path (got {})",
        steering_events.len(),
    );

    println!(
        "AC-4: branch.protected code/class/menu preserved, main ref unchanged ({main_before} == {main_after})"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Fixtures + helpers (mirror crates/but-api/tests/commit_gate.rs)
// ---------------------------------------------------------------------------

fn governed_repo() -> (gix::Repository, tempfile::TempDir) {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");
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

fn commit_to_ref(
    ctx: &mut but_ctx::Context,
    ref_name: &str,
    message: &str,
    dry_run: DryRun,
) -> anyhow::Result<but_api::commit::types::CommitCreateResult> {
    let repo = ctx.repo.get()?.clone();
    let changes = worktree_changes_as_specs(&repo)?;
    but_api::commit::create::commit_create_only(
        ctx,
        RelativeTo::Reference(gix::refs::FullName::try_from(ref_name)?),
        InsertSide::Below,
        changes,
        message.to_owned(),
        dry_run,
    )
}

fn worktree_changes_as_specs(repo: &gix::Repository) -> anyhow::Result<Vec<DiffSpec>> {
    Ok(but_core::diff::worktree_changes(repo)?
        .changes
        .into_iter()
        .map(DiffSpec::from)
        .collect())
}

fn write_file(repo: &gix::Repository, path: &str, content: &str) -> anyhow::Result<()> {
    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("test repository must be non-bare"))?;
    std::fs::write(workdir.join(path), content)?;
    Ok(())
}

fn checkout(repo: &gix::Repository, branch_name: &str) {
    but_testsupport::invoke_bash(&format!("git checkout {branch_name}"), repo);
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
