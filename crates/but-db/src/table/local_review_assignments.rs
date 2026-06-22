#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

use crate::{DbHandle, M, SchemaVersion, Transaction};

pub(crate) const M: &[M<'static>] = &[M::up(
    20260621120000,
    SchemaVersion::Zero,
    "CREATE TABLE `local_review_assignments`(
	`id` TEXT NOT NULL PRIMARY KEY,
	`target` TEXT NOT NULL,
	`reviewer_principal` TEXT NOT NULL,
	`state` TEXT NOT NULL,
	`assigned_at` TIMESTAMP NOT NULL
);

-- UNIQUE so that `upsert` can use ON CONFLICT(target, reviewer_principal) DO UPDATE
-- for idempotent per-(target, reviewer_principal) writes.
CREATE UNIQUE INDEX `idx_local_review_assignments_target_reviewer`
ON `local_review_assignments`(`target`, `reviewer_principal`);",
)];

/// Tests are in `but-db/tests/db/table/local_review_assignments.rs`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalReviewAssignment {
    pub id: String,
    pub target: String,
    pub reviewer_principal: String,
    pub state: String,
    pub assigned_at: chrono::NaiveDateTime,
}

impl DbHandle {
    pub fn local_review_assignments(&self) -> LocalReviewAssignmentsHandle<'_> {
        LocalReviewAssignmentsHandle { conn: &self.conn }
    }

    pub fn local_review_assignments_mut(&mut self) -> LocalReviewAssignmentsHandleMut<'_> {
        LocalReviewAssignmentsHandleMut { conn: &self.conn }
    }
}

impl<'conn> Transaction<'conn> {
    pub fn local_review_assignments(&self) -> LocalReviewAssignmentsHandle<'_> {
        LocalReviewAssignmentsHandle { conn: self.inner() }
    }

    pub fn local_review_assignments_mut(&mut self) -> LocalReviewAssignmentsHandleMut<'_> {
        LocalReviewAssignmentsHandleMut { conn: self.inner() }
    }
}

pub struct LocalReviewAssignmentsHandle<'conn> {
    conn: &'conn rusqlite::Connection,
}

pub struct LocalReviewAssignmentsHandleMut<'conn> {
    conn: &'conn rusqlite::Connection,
}

impl LocalReviewAssignmentsHandle<'_> {
    /// List local review assignments for a target, ordered by assignment time then id.
    pub fn list_by_target(&self, target: &str) -> rusqlite::Result<Vec<LocalReviewAssignment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, reviewer_principal, state, assigned_at \
             FROM local_review_assignments WHERE target = ?1 \
             ORDER BY assigned_at ASC, id ASC",
        )?;

        let results = stmt.query_map([target], |row| {
            Ok(LocalReviewAssignment {
                id: row.get(0)?,
                target: row.get(1)?,
                reviewer_principal: row.get(2)?,
                state: row.get(3)?,
                assigned_at: row.get(4)?,
            })
        })?;

        results.collect::<Result<Vec<_>, _>>()
    }
}

impl LocalReviewAssignmentsHandleMut<'_> {
    /// Enable read-only access functions.
    pub fn to_ref(&self) -> LocalReviewAssignmentsHandle<'_> {
        LocalReviewAssignmentsHandle { conn: self.conn }
    }

    /// Insert or update a local review assignment.
    ///
    /// Idempotent per `(target, reviewer_principal)`: a second call with the same
    /// target+reviewer updates the existing row's `state` and `assigned_at` rather
    /// than inserting a duplicate.
    pub fn upsert(&mut self, row: LocalReviewAssignment) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO local_review_assignments \
             (id, target, reviewer_principal, state, assigned_at) \
             VALUES (?1, ?2, ?3, ?4, ?5) \
             ON CONFLICT(target, reviewer_principal) DO UPDATE SET \
                 state = excluded.state, \
                 assigned_at = excluded.assigned_at",
            rusqlite::params![
                row.id,
                row.target,
                row.reviewer_principal,
                row.state,
                row.assigned_at,
            ],
        )?;
        Ok(())
    }

    /// Flip the `state` of the assignment identified by `(target, reviewer_principal)`.
    ///
    /// Other assignments on the same target are left untouched — the UPDATE scopes to
    /// the `(target, reviewer_principal)` pair, not the whole target.
    pub fn set_state(
        &mut self,
        target: &str,
        reviewer_principal: &str,
        state: &str,
    ) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE local_review_assignments SET state = ?1 \
             WHERE target = ?2 AND reviewer_principal = ?3",
            rusqlite::params![state, target, reviewer_principal],
        )?;
        Ok(())
    }
}
