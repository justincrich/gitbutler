//! GATES AC-5 anti-bypass build-gate: the commit gate must be mechanism-agnostic.
//!
//! The commit gate is enforced at API/CLI/agent entry points rather than at the
//! engine narrow waist `but_workspace::commit::commit_create` (the waist takes a
//! trait-erased selector and cannot name the destination branch to evaluate
//! branch protection — see `crates/but-authz/src/commit_gate.rs`). That design
//! is only sound if EVERY caller of the waist either runs the ref-aware commit
//! gate first or is an explicitly-justified internal lifecycle commit. Without a
//! guard, a future commit-producing mechanism could reach the engine ungoverned
//! (exactly the bypass UC-GATES-01 AC-5 forbids).
//!
//! This test dynamically enumerates every non-test caller of the waist and
//! asserts each caller file is EITHER gated (`enforce_commit_gate*`) OR carries
//! a `GATE-EXEMPT:` allowlist marker. A new ungated caller — in any crate —
//! fails this build-gate. The structural `assert_gate_helper_call_count` in
//! `commit_gate.rs` complements this by pinning specific gate sites; this test
//! provides the exhaustive anti-bypass sweep.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};
use but_testsupport::gix_testtools::tempfile::TempDir;

/// Matches a call to the engine waist `but_workspace::commit::commit_create(`.
/// (The waist *definition* writes `pub fn commit_create(`, which this does not
/// match, so the defining crate is not mistaken for a caller.)
const WAIST_CALL_PATTERN: &str = r"but_workspace::commit::commit_create\(";

/// A caller is governed if it runs the ref-aware commit gate before the waist.
/// `enforce_commit_gate` (the Context-aware resolver) and
/// `enforce_commit_gate_for_target` (the but-authz primitive) both satisfy this.
const GATE_MARKER: &str = "enforce_commit_gate";

/// An internal below-feature-branch lifecycle commit (unapply/stash/branch
/// removal/transaction plumbing) is exempt via an explicit, justified marker.
const EXEMPT_MARKER: &str = "GATE-EXEMPT";

/// The engine waist definition itself — not a caller.
const WAIST_DEFINITION: &str = "crates/but-workspace/src/commit/commit_create.rs";

#[test]
fn every_commit_create_caller_is_gated_or_exempt() -> anyhow::Result<()> {
    let root = workspace_root()?;
    let callers = waist_caller_files(&root)?;

    // Positive control: the grep is wired and finds real callers, including the
    // gated `but-api` commit entry point.
    assert!(
        callers.len() >= 4,
        "expected several commit_create callers across the workspace, found {}: {callers:#?}",
        callers.len()
    );
    assert!(
        callers
            .iter()
            .any(|path| path.ends_with("crates/but-api/src/commit/create.rs")),
        "anti-bypass audit must observe the but-api commit entry point; enumerated callers: {callers:#?}"
    );

    // Every caller must be gated or carry the exemption marker.
    let mut ungated = Vec::new();
    for path in &callers {
        if !is_gated_or_exempt(path)? {
            ungated.push(path.display().to_string());
        }
    }
    assert!(
        ungated.is_empty(),
        "GATES AC-5 (mechanism-agnostic commit gate): the following callers reach the \
         `but_workspace::commit::commit_create` engine waist with NO commit gate and NO \
         `GATE-EXEMPT:` marker — a new ungated commit mechanism that bypasses governance.\n\
         Gate the call (run `enforce_commit_gate*` before it) or, if it is an internal \
         lifecycle commit, add a justified `GATE-EXEMPT:` marker:\n{}",
        ungated.join("\n")
    );

    assert_seeded_controls_fire(&root)?;
    Ok(())
}

/// Teeth: prove the per-file predicate (and hence the audit) actually fails on
/// an ungated caller, and passes a gated/exempt one. A vacuous always-true
/// predicate would be worthless.
fn assert_seeded_controls_fire(_root: &Path) -> anyhow::Result<()> {
    let temp = TempDir::new().context("create seeded commit-gate control temp directory")?;

    let ungated = temp.path().join("ungated-bypass-violation.rs");
    fs::write(
        &ungated,
        "fn bypass() {\n    let _ = but_workspace::commit::commit_create(editor, changes, rel, side, msg, lines);\n}\n",
    )
    .with_context(|| format!("write {}", ungated.display()))?;
    assert!(
        !is_gated_or_exempt(&ungated)?,
        "seeded control: an ungated commit_create caller MUST be flagged (the audit has teeth)"
    );

    let exempt = temp.path().join("exempt-ok.rs");
    fs::write(
        &exempt,
        "// GATE-EXEMPT: seeded control\nfn ok() { let _ = but_workspace::commit::commit_create(); }\n",
    )
    .with_context(|| format!("write {}", exempt.display()))?;
    assert!(
        is_gated_or_exempt(&exempt)?,
        "seeded control: a GATE-EXEMPT-marked caller must pass"
    );

    let gated = temp.path().join("gated-ok.rs");
    fs::write(
        &gated,
        "fn ok() {\n    enforce_commit_gate_for_target(repo, &target)?;\n    let _ = but_workspace::commit::commit_create();\n}\n",
    )
    .with_context(|| format!("write {}", gated.display()))?;
    assert!(
        is_gated_or_exempt(&gated)?,
        "seeded control: an enforce_commit_gate-gated caller must pass"
    );

    Ok(())
}

fn is_gated_or_exempt(path: &Path) -> anyhow::Result<bool> {
    let content =
        fs::read_to_string(path).with_context(|| format!("read caller file {}", path.display()))?;
    Ok(content.contains(GATE_MARKER) || content.contains(EXEMPT_MARKER))
}

/// Enumerate every non-test source file under `crates/` that calls the engine
/// waist, excluding test trees and the waist definition itself.
fn waist_caller_files(root: &Path) -> anyhow::Result<Vec<PathBuf>> {
    let output = Command::new("grep")
        .current_dir(root)
        .args(["-rEl", WAIST_CALL_PATTERN, "crates"])
        .output()
        .with_context(|| format!("failed to run grep from {}", root.display()))?;

    match output.status.code() {
        Some(0) => {}
        Some(1) => bail!(
            "no commit_create callers found — the WAIST_CALL_PATTERN is likely stale (did the \
             engine waist move or get renamed?)"
        ),
        other => bail!(
            "grep failed with status {other:?}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ),
    }

    let stdout = String::from_utf8(output.stdout).context("grep output is not utf-8")?;
    let files = stdout
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        // Test trees may call the waist directly; they are not production mechanisms.
        .filter(|line| !line.contains("/tests/") && !line.ends_with("/tests.rs"))
        // The waist definition is not a caller.
        .filter(|line| *line != WAIST_DEFINITION)
        // This audit file references the pattern in string literals.
        .filter(|line| !line.ends_with("commit_create_caller_audit.rs"))
        .map(|line| root.join(line))
        .collect();
    Ok(files)
}

fn workspace_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("CARGO_MANIFEST_DIR must resolve from crates/but-api to the workspace root")
}
