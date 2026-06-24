//! LPR-009 — Safe-seam invariant: proves the merge gate reads ONLY
//! `local_review_verdicts` at head — never reads `local_review_assignments`,
//! `local_review_comments`, or `local_review_meta` (including the
//! `opener_principal` meta row).
//!
//! Invariant (verbatim from tech-delta §E):
//! "Gate gates (verdict-at-head, untouched); new tables drive (orchestration)."
//!
//! Five tests:
//! 1. `forged_drive_metadata_with_no_verdict_is_blocked` — a fully forged
//!    drive layer (all assignments `approved`, all comments `resolved`, opener
//!    meta forged) with NO verdict at head is BLOCKED (`gate.review_required`).
//! 2. `only_verdict_at_head_flips_gate` — a single approved verdict at head
//!    with NO drive rows makes the gate PROCEED.
//! 3. `safe_seam_forged_drive_equals_empty_drive` — bidirectional
//!    equivalence: forged drive rows AND empty drive rows produce IDENTICAL
//!    denial decisions for both satisfied and unsatisfied verdicts.
//! 4. `safe_seam_only_verdict_at_head_flips_the_land` — 3-step chained
//!    capstone: Step1 blocked → Step2 forged drive still blocked with
//!    IDENTICAL decision → Step3 approved verdict proceeds.
//! 5. `safe_seam_drive_rows_have_no_effect_on_satisfied_merge` — pending
//!    assignment + changes_requested assignment + unresolved comment don't
//!    change a satisfied merge gate.
//!
//! R6/R18 honesty: the verdict store itself is forgeable by a direct DB write
//! (accepted leak — this task does NOT prove the verdict store is unforgeable).
//! These tests prove the DRIVE/GATE SEPARATION: the three drive tables never
//! participate in the land decision.

use but_api::legacy::merge_gate::{MergeGateError, enforce_merge_gate};
use but_db::ForgeReview;

const FEAT_REF: &str = "refs/heads/feat";
const REVIEW_ID: usize = 1;

/// The safe-seam invariant, verbatim — embedded as a message arg so the proof
/// is self-documenting.
const SAFE_SEAM_INVARIANT: &str =
    "Gate gates (verdict-at-head, untouched); new tables drive (orchestration).";

// =========================================================================
// Test 1: Forged drive metadata with no verdict is blocked (AC-5)
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
// Test 2: Only verdict at head flips gate (AC-3)
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
// =========================================================================
// Test 3: Forged drive equals empty drive (bidirectional equivalence)
// =========================================================================

/// Forged drive rows AND empty drive rows produce IDENTICAL denial decisions
/// when no verdict is at head. The gate ignores drive metadata entirely — the
/// decision is identical regardless of whether drive rows exist.
#[tokio::test]
#[serial_test::serial]
async fn safe_seam_forged_drive_equals_empty_drive() -> anyhow::Result<()> {
    // Repo A: WITH forged drive rows (no verdict).
    let (repo_a, _tmp_a) = safe_seam_gated_repo()?;
    let head_a = ref_id(&repo_a, FEAT_REF)?;

    let ctx_with_drive = context_with_review(&repo_a, head_a)?;
    {
        let mut db = ctx_with_drive.db.get_cache_mut()?;
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: "forged-a1".to_owned(),
                target: "feat".to_owned(),
                reviewer_principal: "reviewer".to_owned(),
                state: "approved".to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
        db.local_review_comments_mut()
            .insert(but_db::LocalReviewComment {
                id: "forged-c1".to_owned(),
                target: "feat".to_owned(),
                author_principal: "reviewer".to_owned(),
                body: "forged resolved thread".to_owned(),
                file: None,
                line: None,
                thread_id: "forged-thread".to_owned(),
                resolved: true,
                created_at: chrono::Utc::now().naive_utc(),
            })?;
        db.local_review_meta_mut()
            .upsert_if_absent(but_db::LocalReviewMeta {
                target: "feat".to_owned(),
                key: "opener_principal".to_owned(),
                value: "forged-agent".to_owned(),
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }

    // Verify drive rows present, no verdict.
    {
        let db = ctx_with_drive.db.get_cache()?;
        assert!(
            !db.local_review_assignments()
                .list_by_target("feat")?
                .is_empty(),
            "forged assignments must be present"
        );
        assert!(
            !db.local_review_comments()
                .list_by_target("feat")?
                .is_empty(),
            "forged comments must be present"
        );
        assert!(
            db.local_review_meta()
                .get("feat", "opener_principal")?
                .is_some(),
            "forged opener meta must be present"
        );
        assert_eq!(
            db.local_review_verdicts().list_by_target("feat")?.len(),
            0,
            "no verdict at head"
        );
    }

    let drive_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx_with_drive, REVIEW_ID)
        })
        .await;

    // Repo B: WITHOUT drive rows (clean fixture, separate repo).
    let (repo_b, _tmp_b) = safe_seam_gated_repo()?;
    let head_b = ref_id(&repo_b, FEAT_REF)?;

    let ctx_empty = context_with_review(&repo_b, head_b)?;
    {
        let db = ctx_empty.db.get_cache()?;
        assert_eq!(
            db.local_review_assignments().list_by_target("feat")?.len(),
            0,
            "empty: no drive assignment rows"
        );
        assert_eq!(
            db.local_review_comments().list_by_target("feat")?.len(),
            0,
            "empty: no drive comment rows"
        );
        assert_eq!(
            db.local_review_verdicts().list_by_target("feat")?.len(),
            0,
            "empty: no verdict at head"
        );
    }

    let empty_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx_empty, REVIEW_ID)
        })
        .await;

    // Both must produce IDENTICAL denial: same code.
    match (drive_result, empty_result) {
        (Err(drive_err), Err(empty_err)) => {
            let drive_gate = drive_err
                .downcast_ref::<MergeGateError>()
                .expect("should be MergeGateError");
            let empty_gate = empty_err
                .downcast_ref::<MergeGateError>()
                .expect("should be MergeGateError");
            assert_eq!(
                drive_gate.code, empty_gate.code,
                "{SAFE_SEAM_INVARIANT} — forged drive rows and empty drive must produce identical gate decision code"
            );
            assert_eq!(
                drive_gate.code, "gate.review_required",
                "{SAFE_SEAM_INVARIANT} — no verdict at head must be blocked regardless of drive rows"
            );
        }
        (Ok(()), _) => {
            panic!("{SAFE_SEAM_INVARIANT} — forged drive rows with no verdict must be blocked")
        }
        (_, Ok(())) => {
            panic!("{SAFE_SEAM_INVARIANT} — empty drive with no verdict must be blocked")
        }
    }

    Ok(())
}
// =========================================================================
// Test 4: Only verdict at head flips the land (3-step chained capstone)
// =========================================================================

/// 3-step chained capstone:
///   Step1 — merge with no verdict → blocked (gate.review_required)
///   Step2 — insert forged drive rows (no verdict) → still blocked with
///           IDENTICAL decision
///   Step3 — insert real approved verdict at head → PROCEEDS
/// This proves the verdict-at-head is the ONLY signal that flips the gate;
/// drive rows never participate in the land decision.
#[tokio::test]
#[serial_test::serial]

async fn safe_seam_only_verdict_at_head_flips_the_land() -> anyhow::Result<()> {
    let (repo, _tmp) = safe_seam_gated_repo()?;
    let head = ref_id(&repo, FEAT_REF)?;
    let head_oid = head.to_string();

    // Step 1: merge with NO verdict → blocked.
    let ctx_step1 = context_with_review(&repo, head)?;
    {
        let db = ctx_step1.db.get_cache()?;
        assert_eq!(
            db.local_review_verdicts().list_by_target("feat")?.len(),
            0,
            "Step1: no verdict at head"
        );
    }
    let step1_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx_step1, REVIEW_ID)
        })
        .await;
    let step1_code = match &step1_result {
        Err(err) => {
            err.downcast_ref::<MergeGateError>()
                .expect("Step1 blocked should be MergeGateError")
                .code
        }
        Ok(()) => panic!("Step1: no verdict must be blocked"),
    };
    assert_eq!(step1_code, "gate.review_required");

    // Step 2: insert forged drive rows (no verdict) → still blocked with IDENTICAL decision.
    let ctx_step2 = context_with_review(&repo, head)?;
    {
        let mut db = ctx_step2.db.get_cache_mut()?;
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: "step2-assignment".to_owned(),
                target: "feat".to_owned(),
                reviewer_principal: "reviewer".to_owned(),
                state: "approved".to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
        db.local_review_comments_mut()
            .insert(but_db::LocalReviewComment {
                id: "step2-comment".to_owned(),
                target: "feat".to_owned(),
                author_principal: "reviewer".to_owned(),
                body: "step2 forged".to_owned(),
                file: None,
                line: None,
                thread_id: "step2-thread".to_owned(),
                resolved: true,
                created_at: chrono::Utc::now().naive_utc(),
            })?;
        db.local_review_meta_mut()
            .upsert_if_absent(but_db::LocalReviewMeta {
                target: "feat".to_owned(),
                key: "opener_principal".to_owned(),
                value: "forged-agent".to_owned(),
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }
    let step2_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx_step2, REVIEW_ID)
        })
        .await;
    match &step2_result {
        Err(err) => {
            let gate_err = err
                .downcast_ref::<MergeGateError>()
                .expect("Step2 blocked should be MergeGateError");
            assert_eq!(
                gate_err.code, step1_code,
                "{SAFE_SEAM_INVARIANT} — Step2 forged drive rows must produce IDENTICAL decision to Step1 (no verdict)"
            );
        }
        Ok(()) => {
            panic!("{SAFE_SEAM_INVARIANT} — Step2 forged drive with no verdict must be blocked")
        }
    }

    // Step 3: insert real approved verdict at head (no drive rows) → PROCEEDS.
    let ctx_step3 = context_with_review(&repo, head)?;
    {
        let mut db = ctx_step3.db.get_cache_mut()?;
        db.local_review_verdicts_mut()
            .insert(but_db::LocalReviewVerdict {
                id: "step3-verdict".to_owned(),
                target: "feat".to_owned(),
                principal_id: "reviewer".to_owned(),
                verdict: "approved".to_owned(),
                head_oid,
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }
    let step3_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx_step3, REVIEW_ID)
        })
        .await;
    assert!(
        step3_result.is_ok(),
        "{SAFE_SEAM_INVARIANT} — Step3 approved verdict at head must PROCEED: {:?}",
        step3_result.as_ref().err().map(|e| e.to_string()),
    );

    Ok(())
}

// =========================================================================
// Test 5: Drive rows have no effect on satisfied merge
// =========================================================================

/// A satisfied merge gate (approved verdict at head) is NOT blocked by
/// drive metadata: pending assignment, changes_requested assignment, and
/// an unresolved comment all leave the gate decision unchanged.
#[tokio::test]
#[serial_test::serial]
async fn safe_seam_drive_rows_have_no_effect_on_satisfied_merge() -> anyhow::Result<()> {
    let (repo, _tmp) = safe_seam_gated_repo()?;
    let head = ref_id(&repo, FEAT_REF)?;
    let head_oid = head.to_string();
    let ctx = context_with_review(&repo, head)?;

    // Seed a satisfied merge: approved verdict at head from a reviewer
    // distinct from the author ("impl").
    {
        let mut db = ctx.db.get_cache_mut()?;
        db.local_review_verdicts_mut()
            .insert(but_db::LocalReviewVerdict {
                id: "satisfied-verdict".to_owned(),
                target: "feat".to_owned(),
                principal_id: "reviewer".to_owned(),
                verdict: "approved".to_owned(),
                head_oid: head_oid.clone(),
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }

    // Gate must proceed with clean state (no drive rows).
    let clean_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx, REVIEW_ID)
        })
        .await;
    assert!(
        clean_result.is_ok(),
        "{SAFE_SEAM_INVARIANT} — approved verdict at head with no drive rows must proceed: {:?}",
        clean_result.as_ref().err().map(|e| e.to_string()),
    );

    // Add "noisy" drive rows that should NOT affect the gate decision.
    {
        let mut db = ctx.db.get_cache_mut()?;
        // Pending assignment (not approved).
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: "pending-assignment".to_owned(),
                target: "feat".to_owned(),
                reviewer_principal: "another-reviewer".to_owned(),
                state: "pending".to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
        // Changes requested assignment.
        db.local_review_assignments_mut()
            .upsert(but_db::LocalReviewAssignment {
                id: "changes-requested-assignment".to_owned(),
                target: "feat".to_owned(),
                reviewer_principal: "third-reviewer".to_owned(),
                state: "changes_requested".to_owned(),
                assigned_at: chrono::Utc::now().naive_utc(),
            })?;
        // Unresolved comment.
        db.local_review_comments_mut()
            .insert(but_db::LocalReviewComment {
                id: "unresolved-comment".to_owned(),
                target: "feat".to_owned(),
                author_principal: "reviewer".to_owned(),
                body: "please fix this".to_owned(),
                file: Some("feat.txt".to_owned()),
                line: Some(1),
                thread_id: "unresolved-thread".to_owned(),
                resolved: false,
                created_at: chrono::Utc::now().naive_utc(),
            })?;
    }

    // Verify drive rows are present.
    {
        let db = ctx.db.get_cache()?;
        assert_eq!(
            db.local_review_assignments().list_by_target("feat")?.len(),
            2,
            "two assignments present (pending + changes_requested)"
        );
        assert_eq!(
            db.local_review_comments().list_by_target("feat")?.len(),
            1,
            "one unresolved comment present"
        );
    }

    // Run gate with noisy drive rows — must STILL proceed.
    let noisy_result: anyhow::Result<()> =
        temp_env::async_with_vars([("BUT_AGENT_HANDLE", Some("maint"))], async {
            enforce_merge_gate(&ctx, REVIEW_ID)
        })
        .await;
    assert!(
        noisy_result.is_ok(),
        "{SAFE_SEAM_INVARIANT} — drive rows (pending, changes_requested, unresolved) must not block a satisfied merge gate: {:?}",
        noisy_result.as_ref().err().map(|e| e.to_string()),
    );

    Ok(())
}

// // Shared fixture helpers (mirror crates/but-api/tests/merge_gate.rs)
// =========================================================================

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
