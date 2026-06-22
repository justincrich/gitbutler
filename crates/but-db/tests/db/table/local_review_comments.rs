use but_db::LocalReviewComment;

use crate::table::in_memory_db;

/// AC-3: insert persists comment (resolved=false, created_at set, file+line Some/None round-trip);
/// list_by_thread + list_by_target both return it.
#[test]
fn local_review_comments_insert_and_list() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let code_comment = comment(
        "c1",
        "refs/heads/feat",
        "rev",
        "fix this",
        Some("f.rs".to_string()),
        Some(12),
        "t1",
        1_000_000,
    );
    let pr_level = comment(
        "c2",
        "refs/heads/feat",
        "rev",
        "general remark",
        None,
        None,
        "t2",
        1_000_001,
    );

    db.local_review_comments_mut()
        .insert(code_comment.clone())?;
    db.local_review_comments_mut().insert(pr_level.clone())?;

    let by_thread = db
        .local_review_comments()
        .list_by_thread("refs/heads/feat", "t1")?;
    println!("insert_and_list by_thread(t1)={by_thread:?}");
    assert_eq!(
        by_thread,
        vec![code_comment.clone()],
        "list_by_thread(t1) should return only C1 with resolved=false, file=Some(f.rs), line=Some(12) — \
         a missing thread_id filter would leak C2"
    );
    assert!(
        !by_thread[0].resolved,
        "resolved must be false on insert, not defaulted to true"
    );

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
        by_target[0], code_comment,
        "first comment should round-trip with file=Some(f.rs), line=Some(12)"
    );
    assert_eq!(
        by_target[1], pr_level,
        "PR-level comment should round-trip file=None && line=None"
    );

    Ok(())
}

/// AC-4: set_resolved(thread_id, resolved) flips every comment in a thread,
/// leaves another thread's comments untouched.
#[test]
fn local_review_comments_set_resolved_scopes_to_thread() -> anyhow::Result<()> {
    let mut db = in_memory_db();

    let t1a = comment(
        "c1a",
        "refs/heads/feat",
        "rev",
        "first in t1",
        None,
        None,
        "t1",
        1_000_000,
    );
    let t1b = comment(
        "c1b",
        "refs/heads/feat",
        "rev",
        "second in t1",
        None,
        None,
        "t1",
        1_000_001,
    );
    let t2 = comment(
        "c2",
        "refs/heads/feat",
        "rev",
        "first in t2",
        None,
        None,
        "t2",
        1_000_002,
    );

    db.local_review_comments_mut().insert(t1a)?;
    db.local_review_comments_mut().insert(t1b)?;
    db.local_review_comments_mut().insert(t2)?;

    db.local_review_comments_mut().set_resolved("t1", true)?;

    let rows = db
        .local_review_comments()
        .list_by_target("refs/heads/feat")?;
    println!("set_resolved_scopes_to_thread rows={rows:?}");

    let by_thread: std::collections::HashMap<&str, Vec<bool>> = {
        let mut map: std::collections::HashMap<&str, Vec<bool>> = std::collections::HashMap::new();
        for r in &rows {
            map.entry(r.thread_id.as_str())
                .or_default()
                .push(r.resolved);
        }
        map
    };

    let t1_resolved = by_thread.get("t1").expect("t1 comments must exist");
    assert!(
        t1_resolved.iter().all(|&r| r),
        "every comment in t1 (C1a, C1b) should be resolved — got {t1_resolved:?}"
    );
    assert_eq!(
        t1_resolved.len(),
        2,
        "both t1 comments should be present — a LIMIT 1 bug would only flip one"
    );

    let t2_resolved = by_thread.get("t2").expect("t2 comment must exist");
    assert!(
        t2_resolved.iter().all(|&r| !r),
        "C2 (thread t2) should remain unresolved after set_resolved(t1, true) — got {t2_resolved:?}"
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
