use but_db::LocalReviewComment;

use crate::table::in_memory_db;

/// AC-3: insert persists comment (resolved=false, created_at set, file+line Some/None round-trip);
/// list_by_thread + list_by_target both return it.
#[test]
fn local_review_comments_insert_and_list() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let with_anchor = comment(
        "c1",
        "refs/heads/feat",
        "rust-reviewer",
        "nits to fix",
        Some("src/lib.rs".to_string()),
        Some(42),
        "thread-1",
        1_000_000,
    );
    let without_anchor = comment(
        "c2",
        "refs/heads/feat",
        "justin",
        "general remark",
        None,
        None,
        "thread-2",
        1_000_001,
    );

    db.local_review_comments_mut().insert(with_anchor.clone())?;
    db.local_review_comments_mut()
        .insert(without_anchor.clone())?;

    let by_target = db
        .local_review_comments()
        .list_by_target("refs/heads/feat")?;
    println!("insert_and_list by_target={by_target:?}");

    assert_eq!(
        by_target.len(),
        2,
        "both comments should be listed under the target"
    );
    assert_eq!(
        by_target[0], with_anchor,
        "first comment should round-trip with file+line anchor"
    );
    assert_eq!(
        by_target[1], without_anchor,
        "second comment should round-trip without anchor"
    );

    let thread1 = db
        .local_review_comments()
        .list_by_thread("refs/heads/feat", "thread-1")?;
    println!("insert_and_list thread-1={thread1:?}");
    assert_eq!(
        thread1,
        vec![with_anchor.clone()],
        "thread-1 should return only its own comment"
    );

    let thread2 = db
        .local_review_comments()
        .list_by_thread("refs/heads/feat", "thread-2")?;
    println!("insert_and_list thread-2={thread2:?}");
    assert_eq!(
        thread2,
        vec![without_anchor],
        "thread-2 should return only its own comment"
    );

    Ok(())
}

/// AC-4: set_resolved flips every comment in a thread, leaves other thread untouched.
#[test]
fn local_review_comments_set_resolved_scopes_to_thread() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let t1_first = comment(
        "c1",
        "refs/heads/feat",
        "rust-reviewer",
        "first in thread 1",
        None,
        None,
        "thread-1",
        1_000_000,
    );
    let t1_second = comment(
        "c2",
        "refs/heads/feat",
        "justin",
        "second in thread 1",
        None,
        None,
        "thread-1",
        1_000_001,
    );
    let t2_first = comment(
        "c3",
        "refs/heads/feat",
        "rust-reviewer",
        "first in thread 2",
        None,
        None,
        "thread-2",
        1_000_002,
    );

    db.local_review_comments_mut().insert(t1_first)?;
    db.local_review_comments_mut().insert(t1_second)?;
    db.local_review_comments_mut().insert(t2_first)?;

    db.local_review_comments_mut()
        .set_resolved("refs/heads/feat", "thread-1", true)?;

    let thread1 = db
        .local_review_comments()
        .list_by_thread("refs/heads/feat", "thread-1")?;
    println!("set_resolved thread-1={thread1:?}");
    assert!(
        thread1.iter().all(|c| c.resolved),
        "every comment in thread-1 should be resolved"
    );

    let thread2 = db
        .local_review_comments()
        .list_by_thread("refs/heads/feat", "thread-2")?;
    println!("set_resolved thread-2={thread2:?}");
    assert!(
        thread2.iter().all(|c| !c.resolved),
        "thread-2 should remain unresolved after thread-1 was toggled"
    );

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn comment(
    id: &str,
    target: &str,
    author_principal: &str,
    body: &str,
    file: Option<String>,
    line: Option<i64>,
    thread_id: &str,
    created_at_secs: i64,
) -> LocalReviewComment {
    LocalReviewComment {
        id: id.to_string(),
        target: target.to_string(),
        author_principal: author_principal.to_string(),
        body: body.to_string(),
        file,
        line,
        thread_id: thread_id.to_string(),
        resolved: false,
        created_at: chrono::DateTime::from_timestamp(created_at_secs, 0)
            .expect("fixed timestamp is valid")
            .naive_utc(),
    }
}
