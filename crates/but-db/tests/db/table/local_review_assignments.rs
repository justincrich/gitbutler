use but_db::LocalReviewAssignment;

use crate::table::in_memory_db;

/// AC-1: upsert inserts then idempotently updates per (target, reviewer_principal).
/// A second upsert with same target+reviewer UPDATES the row (does NOT duplicate).
#[test]
fn local_review_assignments_upsert_and_list_by_target() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let first = assignment("a1", "refs/heads/feat", "rev", "pending", 1_000_000);
    // Same (target, reviewer_principal), different state/assigned_at — must UPDATE, not duplicate.
    let updated = assignment("a1-after", "refs/heads/feat", "rev", "approved", 1_000_100);

    db.local_review_assignments_mut().upsert(first.clone())?;
    db.local_review_assignments_mut().upsert(updated.clone())?;

    let rows = db
        .local_review_assignments()
        .list_by_target("refs/heads/feat")?;
    println!("upsert_and_list rows={rows:?}");

    assert_eq!(
        rows.len(),
        1,
        "second upsert with same (target, reviewer) must update, not duplicate — \
         a non-idempotent upsert would return 2 rows"
    );
    assert_eq!(
        rows[0].state, "approved",
        "state should be updated to the second upsert value (approved), not still pending"
    );
    assert_eq!(
        rows[0].reviewer_principal, "rev",
        "reviewer_principal round-trips on the returned row"
    );
    assert_eq!(
        rows[0].target, "refs/heads/feat",
        "target round-trips on the returned row"
    );
    assert_eq!(
        rows[0].assigned_at, updated.assigned_at,
        "assigned_at should be updated to the second upsert value"
    );

    Ok(())
}

/// AC-2: set_state(target, reviewer_principal, state) flips one reviewer's state and
/// leaves other assignments on the same target untouched.
#[test]
fn local_review_assignments_set_state_targets_one_reviewer() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let rev = assignment("a1", "refs/heads/feat", "rev", "pending", 1_000_000);
    let rev2 = assignment("a2", "refs/heads/feat", "rev2", "pending", 1_000_001);

    db.local_review_assignments_mut().upsert(rev)?;
    db.local_review_assignments_mut().upsert(rev2)?;

    db.local_review_assignments_mut()
        .set_state("refs/heads/feat", "rev", "changes_requested")?;

    let rows = db
        .local_review_assignments()
        .list_by_target("refs/heads/feat")?;
    println!("set_state_targets_one_reviewer rows={rows:?}");

    assert_eq!(
        rows.len(),
        2,
        "both assignments must remain after a single set_state"
    );

    let by_reviewer: std::collections::HashMap<&str, &str> = rows
        .iter()
        .map(|r| (r.reviewer_principal.as_str(), r.state.as_str()))
        .collect();
    assert_eq!(
        by_reviewer.get("rev"),
        Some(&"changes_requested"),
        "the targeted reviewer (rev) should be changes_requested"
    );
    assert_eq!(
        by_reviewer.get("rev2"),
        Some(&"pending"),
        "the other reviewer (rev2) on the same target must remain pending — \
         set_state scopes to the (target, reviewer_principal) pair, not the whole target"
    );

    Ok(())
}

/// AC-6: All three tables exist on a fresh migrated DbHandle, all are SchemaVersion::Zero,
/// and the three migrations are registered with the three highest ids
/// (`20260621120000`, `20260621120100`, `20260621120200`).
#[test]
fn local_review_tables_migrations_registered_zero() -> anyhow::Result<()> {
    let db = in_memory_db();

    // Each table must be queryable on a fresh DbHandle. A missing MIGRATIONS entry
    // would cause a "no such table" error here.
    let assignments = db
        .local_review_assignments()
        .list_by_target("refs/heads/feat")?;
    let comments = db
        .local_review_comments()
        .list_by_target("refs/heads/feat")?;
    let meta = db.local_review_meta().get("refs/heads/feat", "any-key")?;
    println!(
        "fresh_db queryable assignments={} comments={} meta_is_some={}",
        assignments.len(),
        comments.len(),
        meta.is_some()
    );
    assert!(
        assignments.is_empty(),
        "fresh DbHandle should have no assignments"
    );
    assert!(
        comments.is_empty(),
        "fresh DbHandle should have no comments"
    );
    assert!(meta.is_none(), "fresh DbHandle should have no meta row");

    // Verify all three migrations are registered with SchemaVersion::Zero by inspecting
    // the `M` Debug output (private fields are still observable through Debug).
    let debug_lines: Vec<String> = but_db::migration::ours()
        .map(|m| format!("{m:?}"))
        .collect();

    for (id, label) in [
        (20260621120000u64, "local_review_assignments"),
        (20260621120100u64, "local_review_comments"),
        (20260621120200u64, "local_review_meta"),
    ] {
        let needle = format!("up_created_at: {id}");
        let matched = debug_lines
            .iter()
            .find(|line| line.contains(&needle))
            .unwrap_or_else(|| {
                panic!("{label} migration with id {id} must be registered; got: {debug_lines:?}")
            });
        assert!(
            matched.contains("schema_version: 0"),
            "{label} migration must declare SchemaVersion::Zero; got: {matched}"
        );
    }

    // Sanity: the three new ids are the highest in the registered set.
    let mut all_ids: Vec<u64> = but_db::migration::ours()
        .map(|m| format!("{m:?}"))
        .filter_map(|line| {
            line.split("up_created_at: ")
                .nth(1)
                .and_then(|rest| rest.split(',').next())
                .and_then(|s| s.trim().parse::<u64>().ok())
        })
        .collect();
    all_ids.sort_unstable_by(|a, b| b.cmp(a));
    println!(
        "highest_migration_ids={:?}",
        &all_ids[..all_ids.len().min(5)]
    );
    assert!(
        all_ids.starts_with(&[20260621120200, 20260621120100, 20260621120000]),
        "the three new migrations should be the three highest ids in order; got: {all_ids:?}"
    );

    Ok(())
}

fn assignment(
    id: &str,
    target: &str,
    reviewer_principal: &str,
    state: &str,
    assigned_at_secs: i64,
) -> LocalReviewAssignment {
    LocalReviewAssignment {
        id: id.to_string(),
        target: target.to_string(),
        reviewer_principal: reviewer_principal.to_string(),
        state: state.to_string(),
        assigned_at: chrono::DateTime::from_timestamp(assigned_at_secs, 0)
            .expect("fixed timestamp is valid")
            .naive_utc(),
    }
}
