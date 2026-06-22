use but_db::LocalReviewAssignment;

use crate::table::in_memory_db;

/// AC-1: upsert inserts then idempotently updates per (target, reviewer_principal).
/// A second upsert with same target+reviewer UPDATES the row (does NOT duplicate).
#[test]
fn local_review_assignments_upsert_and_list_by_target() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let first = assignment(
        "a1",
        "refs/heads/feat",
        "rust-reviewer",
        "assigned",
        1_000_000,
    );
    // Same (target, reviewer_principal), different id/state/assigned_at — must UPDATE, not duplicate.
    let updated = assignment(
        "a1-after",
        "refs/heads/feat",
        "rust-reviewer",
        "accepted",
        1_000_100,
    );

    db.local_review_assignments_mut().upsert(first.clone())?;
    db.local_review_assignments_mut().upsert(updated.clone())?;

    let rows = db
        .local_review_assignments()
        .list_by_target("refs/heads/feat")?;
    println!("upsert_and_list rows={rows:?}");

    assert_eq!(
        rows.len(),
        1,
        "second upsert with same (target, reviewer) must update, not duplicate"
    );
    assert_eq!(
        rows[0].state, updated.state,
        "state should be updated to the second upsert value"
    );
    assert_eq!(
        rows[0].assigned_at, updated.assigned_at,
        "assigned_at should be updated to the second upsert value"
    );

    Ok(())
}

/// AC-2: set_state flips one assignment's state, leaves other assignments on same target untouched.
#[test]
fn local_review_assignments_set_state_targets_one_reviewer() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let reviewer_a = assignment(
        "a1",
        "refs/heads/feat",
        "rust-reviewer",
        "assigned",
        1_000_000,
    );
    let reviewer_b = assignment(
        "a2",
        "refs/heads/feat",
        "tui-reviewer",
        "assigned",
        1_000_001,
    );

    db.local_review_assignments_mut().upsert(reviewer_a)?;
    db.local_review_assignments_mut().upsert(reviewer_b)?;

    db.local_review_assignments_mut()
        .set_state("a1", "accepted")?;

    let rows = db
        .local_review_assignments()
        .list_by_target("refs/heads/feat")?;
    println!("set_state_targets_one rows={rows:?}");

    assert_eq!(
        rows.len(),
        2,
        "both assignments must remain after a single set_state"
    );
    let by_id: std::collections::HashMap<&str, &str> = rows
        .iter()
        .map(|r| (r.id.as_str(), r.state.as_str()))
        .collect();
    assert_eq!(
        by_id.get("a1"),
        Some(&"accepted"),
        "the targeted reviewer should be accepted"
    );
    assert_eq!(
        by_id.get("a2"),
        Some(&"assigned"),
        "the other reviewer on the same target must remain assigned"
    );

    Ok(())
}

/// AC-6: All three tables exist on a fresh migrated DbHandle, all are SchemaVersion::Zero,
/// and the three migrations are registered with the three highest ids
/// (`20260621120000`, `20260621120100`, `20260621120200`).
#[test]
fn local_review_tables_migrations_registered_zero() -> anyhow::Result<()> {
    let db = in_memory_db();

    // Each table must be queryable on a fresh DbHandle. A missing table would error.
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
