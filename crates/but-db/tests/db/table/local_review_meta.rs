use but_db::LocalReviewMeta;

use crate::table::in_memory_db;

/// AC-5: upsert_if_absent writes opener row; second upsert_if_absent with same (target,key)
/// but different value is a NO-OP (write-once). `get` on missing key returns None.
#[test]
fn local_review_meta_opener_is_write_once_per_target_key() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let opener = meta("refs/heads/feat", "opener_principal", "agent-A", 1_000_000);
    db.local_review_meta_mut()
        .upsert_if_absent(opener.clone())?;

    let fetched = db
        .local_review_meta()
        .get("refs/heads/feat", "opener_principal")?;
    println!("opener_after_first_insert={fetched:?}");
    assert_eq!(
        fetched.as_ref(),
        Some(&opener),
        "first write should be persisted — a stub would return None"
    );

    // Same (target, key) but a different value — must be a NO-OP.
    let impostor = meta("refs/heads/feat", "opener_principal", "impostor", 1_000_001);
    db.local_review_meta_mut().upsert_if_absent(impostor)?;

    let after_second = db
        .local_review_meta()
        .get("refs/heads/feat", "opener_principal")?;
    println!("opener_after_second_insert={after_second:?}");
    assert_eq!(
        after_second.as_ref(),
        Some(&opener),
        "second upsert_if_absent with same (target, key) must be a no-op — \
         value must still be agent-A, NOT impostor (a DO UPDATE impl would overwrite and break the R23 narrowing)"
    );

    let missing = db
        .local_review_meta()
        .get("refs/heads/feat", "missing-key")?;
    println!("missing_key={missing:?}");
    assert!(
        missing.is_none(),
        "a missing (target, key) should return None, not an error or a fabricated row"
    );

    Ok(())
}

fn meta(target: &str, key: &str, value: &str, created_at_secs: i64) -> LocalReviewMeta {
    LocalReviewMeta {
        target: target.to_string(),
        key: key.to_string(),
        value: value.to_string(),
        created_at: chrono::DateTime::from_timestamp(created_at_secs, 0)
            .expect("fixed timestamp is valid")
            .naive_utc(),
    }
}
