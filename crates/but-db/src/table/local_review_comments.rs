#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120100,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_comments`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`target` TEXT NOT NULL,
	`author_principal` TEXT NOT NULL,
	`body` TEXT NOT NULL,
	`file` TEXT,
	`line` INTEGER,
	`thread_id` TEXT NOT NULL,
	`resolved` BOOL NOT NULL,
	`created_at` TIMESTAMP NOT NULL
);

CREATE INDEX `idx_local_review_comments_target_thread`
ON `local_review_comments`(`target`, `thread_id`);",
)];

/// Tests are in `but-db/tests/db/table/local_review_comments.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewComment {
    pub id: String,
    pub target: String,
    pub author_principal: String,
    pub body: String,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub thread_id: String,
    pub resolved: bool,
    pub created_at: chrono::NaiveDateTime,
}

impl DbHandle {
    pub fn local_review_comments(&self) -> LocalReviewCommentsHandle<'_> {
        LocalReviewCommentsHandle { conn: &self.conn }
    }

    pub fn local_review_comments_mut(&mut self) -> LocalReviewCommentsHandleMut<'_> {
        LocalReviewCommentsHandleMut { conn: &self.conn }
    }
}

impl<'conn> Transaction<'conn> {
    pub fn local_review_comments(&self) -> LocalReviewCommentsHandle<'_> {
        LocalReviewCommentsHandle { conn: self.inner() }
    }

    pub fn local_review_comments_mut(&mut self) -> LocalReviewCommentsHandleMut<'_> {
        LocalReviewCommentsHandleMut { conn: self.inner() }
    }
}

pub struct LocalReviewCommentsHandle<'conn> {
    conn: &'conn rusqlite::Connection,
}

pub struct LocalReviewCommentsHandleMut<'conn> {
    conn: &'conn rusqlite::Connection,
}

impl LocalReviewCommentsHandle<'_> {
    /// List local review comments for a target, ordered by creation time then id.
    pub fn list_by_target(&self, target: &str) -> rusqlite::Result<Vec<LocalReviewComment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, author_principal, body, file, line, thread_id, resolved, created_at \
             FROM local_review_comments WHERE target = ?1 \
             ORDER BY created_at ASC, id ASC",
        )?;

        let results = stmt.query_map([target], |row| {
            Ok(LocalReviewComment {
                id: row.get(0)?,
                target: row.get(1)?,
                author_principal: row.get(2)?,
                body: row.get(3)?,
                file: row.get(4)?,
                line: row.get(5)?,
                thread_id: row.get(6)?,
                resolved: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }

    /// List local review comments for a thread scoped to a target,
    /// ordered by creation time then id.
    pub fn list_by_thread(
        &self,
        target: &str,
        thread_id: &str,
    ) -> rusqlite::Result<Vec<LocalReviewComment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, author_principal, body, file, line, thread_id, resolved, created_at \
             FROM local_review_comments WHERE target = ?1 AND thread_id = ?2 \
             ORDER BY created_at ASC, id ASC",
        )?;

        let results = stmt.query_map(rusqlite::params![target, thread_id], |row| {
            Ok(LocalReviewComment {
                id: row.get(0)?,
                target: row.get(1)?,
                author_principal: row.get(2)?,
                body: row.get(3)?,
                file: row.get(4)?,
                line: row.get(5)?,
                thread_id: row.get(6)?,
                resolved: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }
}

impl LocalReviewCommentsHandleMut<'_> {
    /// Enable read-only access functions.
    pub fn to_ref(&self) -> LocalReviewCommentsHandle<'_> {
        LocalReviewCommentsHandle { conn: self.conn }
    }

    /// Insert a local review comment.
    pub fn insert(&mut self, row: LocalReviewComment) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO local_review_comments \
             (id, target, author_principal, body, file, line, thread_id, resolved, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                row.id,
                row.target,
                row.author_principal,
                row.body,
                row.file,
                row.line,
                row.thread_id,
                row.resolved,
                row.created_at,
            ],
        )?;
        Ok(())
    }

    /// Set the `resolved` flag on every comment in a thread scoped to a target.
    ///
    /// Other threads on the same target are left untouched.
    pub fn set_resolved(
        &mut self,
        target: &str,
        thread_id: &str,
        resolved: bool,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE local_review_comments SET resolved = ?1 \
             WHERE target = ?2 AND thread_id = ?3",
            rusqlite::params![resolved, target, thread_id],
        )?;
        Ok(())
    }
}
