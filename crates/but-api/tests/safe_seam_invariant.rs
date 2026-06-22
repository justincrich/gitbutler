//! LPR-009 — Safe-seam invariant: proves the merge gate reads ONLY
//! `local_review_verdicts` at head — never reads `local_review_assignments`,
//! `local_review_comments`, or `local_review_meta` (including the
//! `opener_principal` meta row).
//!
//! Invariant (verbatim from tech-delta §E):
//! "Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."
//!
//! Three tests:
//! 1. `merge_gate_path_has_zero_references_to_drive_tables` — build-gate
//!    honesty grep: the gate path source (`merge_gate.rs` +
//!    `review_requirement.rs`) contains zero references to drive tables.
//! 2. `forged_drive_metadata_with_no_verdict_is_blocked` — a fully forged
//!    drive layer (all assignments `approved`, all comments `resolved`, opener
//!    meta forged) with NO verdict at head is BLOCKED (`gate.review_required`).
//! 3. `only_verdict_at_head_flips_gate` — a single approved verdict at head
//!    with NO drive rows makes the gate PROCEED.
//!
//! R6/R18 honesty: the verdict store itself is forgeable by a direct DB write
//! (accepted leak — this task does NOT prove the verdict store is unforgeable).
//! These tests prove the DRIVE/GATE SEPARATION: the three drive tables never
//! participate in the land decision.

use std::{
    fs,
    path::{Path, PathBuf},
};

use but_api::legacy::merge_gate::{MergeGateError, enforce_merge_gate};
use but_db::ForgeReview;

const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

/// The safe-seam invariant, verbatim — embedded as a message arg so the proof
/// is self-documenting.
const SAFE_SEAM_INVARIANT: &str =
    "Gate gates (verdict-at-head, untouched); new tables drive (orchestration).";

// =========================================================================
// Test 1: Build-gate honesty grep (AC-1)
// =========================================================================

/// AC-1: the merge-gate code path (`merge_gate.rs` + `review_requirement.rs`)
/// contains zero case-insensitive references to `local_review_assignments`,
/// `local_review_comments`, or `local_review_meta`. The gate reads ONLY
/// `local_review_verdicts` at head — the drive/gate separation enforced at
/// build time, not by convention.
#[test]
fn merge_gate_path_has_zero_references_to_drive_tables() -> anyhow::Result<()> {
    let workspace_root = workspace_root()?;
    let gate_paths = [
        "crates/but-api/src/legacy/merge_gate.rs",
        "crates/but-api/src/legacy/review_requirement.rs",
    ];

    let drive_tables = [
        "local_review_assignments",
        "local_review_comments",
        "local_review_meta",
    ];

    let mut combined_source = String::new();
    for relative in &gate_paths {
        let path = workspace_root.join(relative);
        let source = fs::read_to_string(&path).map_err(|err| {
            anyhow::anyhow!("failed to read gate-path file {}: {err}", path.display())
        })?;
        combined_source.push_str(&source);
        combined_source.push('\n');
    }

    let lower = combined_source.to_lowercase();
    let mut violations: Vec<&str> = Vec::new();
    for table in &drive_tables {
        if lower.contains(table) {
            violations.push(table);
        }
    }

    assert!(
        violations.is_empty(),
        "{SAFE_SEAM_INVARIANT} — the merge-gate path must reference NONE of the drive tables, \
		 but found references to: {violations:?} in {}",
        gate_paths.join(" + "),
    );

    // Sanity: confirm the gate path DOES reference the verdict store (the one
    // table it is supposed to read). Without this, the grep test would pass
    // vacuously if the gate path were empty or pointed at the wrong files.
    assert!(
        lower.contains("local_review_verdicts"),
        "sanity check: the gate path must reference local_review_verdicts (the one table it reads)"
    );

    println!(
        "{SAFE_SEAM_INVARIANT}\n  gate path: {}\n  drive tables checked: {drive_tables:?}\n  \
		 violations: {violations:?}\n  verdict store reference: present",
        gate_paths.join(" + "),
    );

    Ok(())
}

// =========================================================================
// Test 2: Forged drive metadata with no verdict is blocked (AC-5)
// =========================================================================

/// AC-5 (inverse): a fully forged drive layer — all assignments `approved`,
/// all comments `resolved`, opener meta row forged — written DIRECTLY via the
/// LPR-001 Handles (the adversarial fixture), with NO `local_review_verdicts`
/// row at head, is BLOCKED with `gate.review_required`. Drive metadata alone
/// never satisfies the gate; only an approved verdict-at-head can.
#[tokio::test]
#[serial_test::serial]
async fn forged_drive_metadata_with_no_verdict_is_blocked() -> anyhow::Result<()> {
    let (repo, _tmp) = safe_seam_gated_repo()?;
    let head = ref_id(&repo, FEAT_REF)?;
    let ctx = context_with_review(&repo, head)?;

    // Forge a fully "satisfied" drive layer via DIRECT DB writes (not through
    // governed verbs — this is the adversarial fixture simulating a forged DB).
    {
        let mut db = ctx.db.get_cache_mut()?;

        // Forged assignment: "approved" state (as if a reviewer approved).
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: "forged-assignment-1".to_owned(),
                target: "feat".to_owned(),
                reviewer_principal: "reviewer".to_owned(),
                state: "approved".to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;

        // Forged comment: "resolved" thread (as if review feedback was addressed).
        db.local_review_comments_mut()
            .insert(but_db::LocalReviewComment {
                id: "forged-comment-1".to_owned(),
                target: "feat".to_owned(),
                author_principal: "reviewer".to_owned(),
                body: "forged resolved thread".to_owned(),
                file: None,
                line: None,
                thread_id: "forged-thread".to_owned(),
                resolved: true,
                created_at: chrono::Utc::now().naive_utc(),
            })?;

        // Forged opener meta row (the R23 accepted-leak forgeable row).
        db.local_review_meta_mut()
            .upsert_if_absent(but_db::LocalReviewMeta {
                target: "feat".to_owned(),
                key: "opener_principal".to_owned(),
                value: "forged-agent".to_owned(),
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }

    // Critically: NO local_review_verdicts row at head.

    // Verify the forged drive rows are actually present (otherwise the test is vacuous).
    {
        let db = ctx.db.get_cache()?;
        assert_eq!(
            db.local_review_assignments().list_by_target("feat")?.len(),
            1,
            "forged assignment must be present — otherwise the test is vacuous"
        );
        assert_eq!(
            db.local_review_comments().list_by_target("feat")?.len(),
            1,
            "forged comment must be present — otherwise the test is vacuous"
        );
        assert!(
            db.local_review_meta()
                .get("feat", "opener_principal")?
                .is_some(),
            "forged opener meta must be present — otherwise the test is vacuous"
        );
        assert_eq!(
            db.local_review_verdicts().list_by_target("feat")?.len(),
            0,
            "no verdict at head — the gate decision must rest on this absence"
        );
    }

    // Run the governed gate under a principal with merge authority.
    let gate_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx, REVIEW_ID)
        })
        .await;

    match gate_result {
        Ok(()) => panic!(
            "{SAFE_SEAM_INVARIANT} — forged drive metadata alone must NOT satisfy the gate; \
			 merge proceeded despite no approved verdict at head (this would be a safe-seam break)"
        ),
        Err(err) => {
            let gate_error = err
                .downcast_ref::<MergeGateError>()
                .expect("blocked merge should produce a MergeGateError");
            assert_eq!(
                gate_error.code, "gate.review_required",
                "{SAFE_SEAM_INVARIANT} — forged drive metadata with no verdict must be blocked \
				 with gate.review_required, got code `{}`: {}",
                gate_error.code, gate_error.message,
            );
            assert!(
                gate_error.unmet.iter().any(|entry| entry == "no_approval"),
                "the block should report no_approval (the verdict-at-head absence), got unmet: {:?}",
                gate_error.unmet,
            );
        }
    }

    Ok(())
}

// =========================================================================
// Test 3: Only verdict at head flips gate (AC-3)
// =========================================================================

/// AC-3 (inverse): a single approved verdict at head, with NO drive rows (no
/// assignment, no comment, no meta), makes the gate PROCEED. This proves the
/// gate cares only about verdict-at-head — the presence/absence of drive
/// metadata is irrelevant to the land decision.
#[tokio::test]
#[serial_test::serial]
async fn only_verdict_at_head_flips_gate() -> anyhow::Result<()> {
    let (repo, _tmp) = safe_seam_gated_repo()?;
    let head = ref_id(&repo, FEAT_REF)?;
    let head_oid = head.to_string();
    let ctx = context_with_review(&repo, head)?;

    // Write NO drive rows. Write ONE approved verdict at the current head.
    // The verdict is written directly via the Handle mutator — this is the
    // R6/R18 accepted-leak (a direct DB write CAN forge an approval). The point
    // of this test is NOT that the verdict store is unforgeable, but that the
    // gate reads ONLY the verdict store and nothing else.
    {
        let mut db = ctx.db.get_cache_mut()?;
        db.local_review_verdicts_mut()
            .insert(but_db::LocalReviewVerdict {
                id: "verdict-at-head-1".to_owned(),
                target: "feat".to_owned(),
                principal_id: "reviewer".to_owned(),
                verdict: "approved".to_owned(),
                head_oid,
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }

    // Verify the drive tables are empty (otherwise the test is vacuous).
    {
        let db = ctx.db.get_cache()?;
        assert_eq!(
            db.local_review_assignments().list_by_target("feat")?.len(),
            0,
            "no drive rows — the gate decision must rest solely on the verdict"
        );
        assert_eq!(
            db.local_review_comments().list_by_target("feat")?.len(),
            0,
            "no drive rows"
        );
        assert_eq!(
            db.local_review_verdicts().list_by_target("feat")?.len(),
            1,
            "exactly one approved verdict at head"
        );
    }

    // Run the governed gate under a principal with merge authority.
    let gate_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx, REVIEW_ID)
        })
        .await;

    assert!(
        gate_result.is_ok(),
        "{SAFE_SEAM_INVARIANT} — a single approved verdict at head with no drive rows must \
		 PROCEED; the gate reads only verdicts at head. Got error: {:?}",
        gate_result.as_ref().err().map(|err| err.to_string()),
    );

    Ok(())
}

// =========================================================================
// Shared fixture helpers (mirror crates/but-api/tests/merge_gate.rs)
// =========================================================================

fn workspace_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map(PathBuf::from)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "CARGO_MANIFEST_DIR must resolve from crates/but-api to the workspace root"
            )
        })
}

/// Build a real governed repo with a protected `main` branch and a single
/// `min_approvals = 1, require_distinct_from_author = true` review requirement,
/// plus a `feat` branch. Mirrors the `GateConfig::Single` fixture from
/// `crates/but-api/tests/merge_gate.rs`.
#[allow(clippy::type_complexity)]
fn safe_seam_gated_repo() -> anyhow::Result<(gix::Repository, tempfile::TempDir)> {
    let (repo, tmp) = but_testsupport::writable_scenario("checkout-head-info");

    but_testsupport::invoke_bash(
        r#"
git remote add origin https://github.com/gitbutler/safe-seam-fixture.git
mkdir -p .gitbutler
cat >.gitbutler/permissions.toml <<'EOF'

[[principal]]
id = "impl"
permissions = ["contents:write", "pull_requests:write", "reviews:write"]

[[principal]]
id = "reviewer"
permissions = ["reviews:write"]

[[principal]]
id = "maint"
permissions = ["merge"]
EOF
cat >.gitbutler/gates.toml <<'EOF'

[[branch]]
name = "main"
protected = true

[[gate]]
branch = "main"
type = "review"
min_approvals = 1
require_distinct_from_author = true
EOF
git add .gitbutler/permissions.toml .gitbutler/gates.toml
git commit -m "governance config"
git checkout -b feat
echo feat >feat.txt
git add feat.txt
git commit -m "feat"
git checkout main
"#,
        &repo,
    );

    Ok((repo, tmp))
}

fn context_with_review(
    repo: &gix::Repository,
    head: gix::ObjectId,
) -> anyhow::Result<but_ctx::Context> {
    let mut ctx = but_ctx::Context::from_repo(repo.clone())?.with_memory_app_cache();
    seed_review(&mut ctx, head)?;
    Ok(ctx)
}

fn seed_review(ctx: &mut but_ctx::Context, head: gix::ObjectId) -> anyhow::Result<()> {
    ctx.db
        .get_cache_mut()?
        .forge_reviews_mut()?
        .upsert(ForgeReview {
            html_url: "https://github.com/gitbutler/safe-seam-fixture/pull/1".to_owned(),
            number: REVIEW_ID.try_into()?,
            title: "Safe-seam fixture".to_owned(),
            body: None,
            author: Some("impl".to_owned()),
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
            repository_https_url: Some(
                "https://github.com/gitbutler/safe-seam-fixture.git".to_owned(),
            ),
            repo_owner: Some("gitbutler".to_owned()),
            head_repo_is_fork: false,
            reviewers: "[]".to_owned(),
            unit_symbol: "#".to_owned(),
            last_sync_at: fixed_time(0),
            struct_version: but_forge::ForgeReview::struct_version(),
        })?;
    Ok(())
}

fn fixed_time(seconds: i64) -> chrono::NaiveDateTime {
    chrono::DateTime::from_timestamp(1_735_689_600 + seconds, 0)
        .expect("fixed timestamp is valid")
        .naive_utc()
}

fn ref_id(repo: &gix::Repository, ref_name: &str) -> anyhow::Result<gix::ObjectId> {
    Ok(repo.find_reference(ref_name)?.peel_to_id()?.detach())
}
