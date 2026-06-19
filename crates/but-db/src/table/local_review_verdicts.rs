#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260619120000,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_verdicts`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`target` TEXT NOT NULL,
	`principal_id` TEXT NOT NULL,
	`verdict` TEXT NOT NULL,
	`head_oid` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL
);

CREATE INDEX `idx_local_review_verdicts_target_created_at`
ON `local_review_verdicts`(`target`, `created_at`);",
)];

/// Tests are in `but-db/tests/db/table/local_review_verdicts.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewVerdict {
    pub id: String,
    pub target: String,
    pub principal_id: String,
    pub verdict: String,
    pub head_oid: String,
    pub created_at: chrono::NaiveDateTime,
}

impl DbHandle {
    pub fn local_review_verdicts(&self) -> LocalReviewVerdictsHandle<'_> {
        LocalReviewVerdictsHandle { conn: &self.conn }
    }

    pub fn local_review_verdicts_mut(&mut self) -> LocalReviewVerdictsHandleMut<'_> {
        LocalReviewVerdictsHandleMut { conn: &self.conn }
    }
}

impl<'conn> Transaction<'conn> {
    pub fn local_review_verdicts(&self) -> LocalReviewVerdictsHandle<'_> {
        LocalReviewVerdictsHandle { conn: self.inner() }
    }

    pub fn local_review_verdicts_mut(&mut self) -> LocalReviewVerdictsHandleMut<'_> {
        LocalReviewVerdictsHandleMut { conn: self.inner() }
    }
}

pub struct LocalReviewVerdictsHandle<'conn> {
    conn: &'conn rusqlite::Connection,
}

pub struct LocalReviewVerdictsHandleMut<'conn> {
    conn: &'conn rusqlite::Connection,
}

impl LocalReviewVerdictsHandle<'_> {
    /// List local review verdicts for a target, ordered by creation time.
    pub fn list_by_target(&self, target: &str) -> rusqlite::Result<Vec<LocalReviewVerdict>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, principal_id, verdict, head_oid, created_at \
             FROM local_review_verdicts WHERE target = ?1 \
             ORDER BY created_at ASC, id ASC",
        )?;

        let results = stmt.query_map([target], |row| {
            Ok(LocalReviewVerdict {
                id: row.get(0)?,
                target: row.get(1)?,
                principal_id: row.get(2)?,
                verdict: row.get(3)?,
                head_oid: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }
}

impl LocalReviewVerdictsHandleMut<'_> {
    /// Enable read-only access functions.
    pub fn to_ref(&self) -> LocalReviewVerdictsHandle<'_> {
        LocalReviewVerdictsHandle { conn: self.conn }
    }

    /// Insert a local review verdict.
    pub fn insert(&mut self, row: LocalReviewVerdict) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO local_review_verdicts \
             (id, target, principal_id, verdict, head_oid, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                row.id,
                row.target,
                row.principal_id,
                row.verdict,
                row.head_oid,
                row.created_at,
            ],
        )?;
        Ok(())
    }
}
