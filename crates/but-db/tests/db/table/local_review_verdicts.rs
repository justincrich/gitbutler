use but_db::LocalReviewVerdict;

use crate::table::in_memory_db;

#[test]
fn insert_and_query_by_target() -> anyhow::Result<()> {
    let mut db = in_memory_db();
    let verdict = local_review_verdict(
        "v1",
        "refs/heads/feat",
        "rust-reviewer",
        "approved",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        1_000_000,
    );

    db.local_review_verdicts_mut().insert(verdict.clone())?;

    let rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/feat")?;
    println!("insert_and_query_by_target rows={rows:?}");

    assert_eq!(
        rows.len(),
        1,
        "the inserted target should return one verdict"
    );
    assert_eq!(
        rows.first(),
        Some(&verdict),
        "every stored field stays intact"
    );

    Ok(())
}

#[test]
fn head_pinning_distinguishes_heads() -> anyhow::Result<()> {
    let mut db = in_memory_db();
    let first_head = local_review_verdict(
        "h1",
        "refs/heads/feat",
        "rust-reviewer",
        "approved",
        "1111111111111111111111111111111111111111",
        1_000_000,
    );
    let second_head = local_review_verdict(
        "h2",
        "refs/heads/feat",
        "rust-reviewer",
        "approved",
        "2222222222222222222222222222222222222222",
        1_000_002,
    );

    db.local_review_verdicts_mut().insert(first_head.clone())?;
    db.local_review_verdicts_mut().insert(second_head.clone())?;

    let rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/feat")?;
    println!("head_pinning_distinguishes_heads rows={rows:?}");

    assert_eq!(rows.len(), 2, "two heads for one target must not collapse");
    assert!(
        rows.contains(&first_head),
        "the old-head verdict should remain queryable"
    );
    assert!(
        rows.contains(&second_head),
        "the new-head verdict should remain queryable"
    );

    Ok(())
}

#[test]
fn empty_target_returns_empty() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let initial_rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/feat")?;
    println!("empty_target_initial rows={initial_rows:?}");
    assert!(
        initial_rows.is_empty(),
        "an empty target should return an empty Vec, not an error"
    );

    let other_verdict = local_review_verdict(
        "o1",
        "refs/heads/other",
        "rust-reviewer",
        "approved",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        1_000_000,
    );
    db.local_review_verdicts_mut()
        .insert(other_verdict.clone())?;

    let other_rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/other")?;
    println!("empty_target_other rows={other_rows:?}");
    assert_eq!(
        other_rows.first(),
        Some(&other_verdict),
        "the unrelated target should be stored"
    );

    let feat_rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/feat")?;
    println!("empty_target_filtered rows={feat_rows:?}");
    assert!(
        feat_rows.is_empty(),
        "querying refs/heads/feat must not leak another target's verdict"
    );

    Ok(())
}

#[test]
fn distinct_principals_both_returned() -> anyhow::Result<()> {
    let mut db = in_memory_db();
    let reviewer_verdict = local_review_verdict(
        "v1",
        "refs/heads/feat",
        "rust-reviewer",
        "approved",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        1_000_000,
    );
    let justin_verdict = local_review_verdict(
        "v2",
        "refs/heads/feat",
        "justin",
        "approved",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        1_000_001,
    );

    db.local_review_verdicts_mut()
        .insert(reviewer_verdict.clone())?;
    db.local_review_verdicts_mut()
        .insert(justin_verdict.clone())?;

    let rows = db
        .local_review_verdicts()
        .list_by_target("refs/heads/feat")?;
    println!("distinct_principals_both_returned rows={rows:?}");

    assert_eq!(
        rows,
        vec![reviewer_verdict, justin_verdict],
        "distinct principals should both return in created_at order"
    );
    assert!(
        rows.iter().all(|row| row.verdict == "approved"),
        "both returned rows should preserve the approving verdict"
    );

    Ok(())
}

fn local_review_verdict(
    id: &str,
    target: &str,
    principal_id: &str,
    verdict: &str,
    head_oid: &str,
    created_at_secs: i64,
) -> LocalReviewVerdict {
    LocalReviewVerdict {
        id: id.to_string(),
        target: target.to_string(),
        principal_id: principal_id.to_string(),
        verdict: verdict.to_string(),
        head_oid: head_oid.to_string(),
        created_at: chrono::DateTime::from_timestamp(created_at_secs, 0)
            .expect("fixed timestamp is valid")
            .naive_utc(),
    }
}
