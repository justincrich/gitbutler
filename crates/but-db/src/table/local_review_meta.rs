#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120200,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_meta`(
	`target` TEXT NOT NULL,
	`key` TEXT NOT NULL,
	`value` TEXT NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	PRIMARY KEY (`target`, `key`)
);",
)];

/// Tests are in `but-db/tests/db/table/local_review_meta.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewMeta {
    pub target: String,
    pub key: String,
    pub value: String,
    pub created_at: chrono::NaiveDateTime,
}

impl DbHandle {
    pub fn local_review_meta(&self) -> LocalReviewMetaHandle<'_> {
        LocalReviewMetaHandle { conn: &self.conn }
    }

    pub fn local_review_meta_mut(&mut self) -> LocalReviewMetaHandleMut<'_> {
        LocalReviewMetaHandleMut { conn: &self.conn }
    }
}

impl<'conn> Transaction<'conn> {
    pub fn local_review_meta(&self) -> LocalReviewMetaHandle<'_> {
        LocalReviewMetaHandle { conn: self.inner() }
    }

    pub fn local_review_meta_mut(&mut self) -> LocalReviewMetaHandleMut<'_> {
        LocalReviewMetaHandleMut { conn: self.inner() }
    }
}

pub struct LocalReviewMetaHandle<'conn> {
    conn: &'conn rusqlite::Connection,
}

pub struct LocalReviewMetaHandleMut<'conn> {
    conn: &'conn rusqlite::Connection,
}

impl LocalReviewMetaHandle<'_> {
    /// Get the metadata row for a single `(target, key)`, or `None` if absent.
    pub fn get(&self, target: &str, key: &str) -> rusqlite::Result<Option<LocalReviewMeta>> {
        let mut stmt = self.conn.prepare(
            "SELECT target, key, value, created_at FROM local_review_meta \
             WHERE target = ?1 AND key = ?2",
        )?;

        let mut results = stmt.query_map(rusqlite::params![target, key], |row| {
            Ok(LocalReviewMeta {
                target: row.get(0)?,
                key: row.get(1)?,
                value: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        match results.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }
}

impl LocalReviewMetaHandleMut<'_> {
    /// Enable read-only access functions.
    pub fn to_ref(&self) -> LocalReviewMetaHandle<'_> {
        LocalReviewMetaHandle { conn: self.conn }
    }

    /// Insert a metadata row only if `(target, key)` is absent.
    ///
    /// Write-once per composite key: a second call with the same `(target, key)` —
    /// even with a different value — is a NO-OP.
    pub fn upsert_if_absent(&mut self, row: LocalReviewMeta) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO local_review_meta (target, key, value, created_at) \
             VALUES (?1, ?2, ?3, ?4) \
             ON CONFLICT(target, key) DO NOTHING",
            rusqlite::params![row.target, row.key, row.value, row.created_at,],
        )?;
        Ok(())
    }
}
