---
stability: CONSTITUTION
last_validated: 2026-06-20
prd_version: 1.0.0
---

# 03 — Data Schema

Three schema surfaces: the **verdict store** (`check_results`, a `but-db`
table), the **check definitions** (`.gitbutler/checks/*.toml`, ref-pinned), and
the **required-set policy** (`[[required_check]]` in `.gitbutler/gates.toml`,
ref-pinned). All three are modeled directly on shapes already in the codebase.

## §1 — `check_results` (the verdict store)

A **plain `but-db` table**. Modeled on `local_review_verdicts`
(`crates/but-db/src/table/local_review_verdicts.rs` — `target TEXT`,
`head_oid TEXT`, `principal_id TEXT`, `verdict TEXT`, `created_at TIMESTAMP`)
and `ci_checks` (`crates/but-db/src/table/ci_checks.rs` — `name TEXT`,
`head_sha TEXT`, `status_conclusion TEXT`). The store needs **no more protection
than `local_review_verdicts`**: governance already accepts that store as
forgeable-by-direct-DB-write (its R6); a check is a _reproducible_ deterministic
review — safer in detectability (a forged green is detectable post-merge), **not** strictly safer (no principal identity). There is **no signing, HMAC, or hardening** (see
01 §5, 08 R-FORGERY).

### Migration (mirror `local_review_verdicts` migration form)

```rust
// crates/but-db/src/table/check_results.rs
pub(crate) const M: &[M<'static>] = &[M::up(
    20260620120000,
    SchemaVersion::Zero,
    "CREATE TABLE `check_results`(
        `id`           TEXT NOT NULL PRIMARY KEY,
        `name`         TEXT NOT NULL,
        `head_oid`     TEXT NOT NULL,
        `conclusion`   TEXT NOT NULL,
        `recorded_at`  TIMESTAMP NOT NULL,
        `metadata`     TEXT NOT NULL,
        `signature`    TEXT
    );

    CREATE INDEX `idx_check_results_name_head_oid`
    ON `check_results`(`name`, `head_oid`);",
)];
```

### Row type

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckResult {
    pub id: String,                    // ULID/uuid string, like LocalReviewVerdict.id
    pub name: String,                  // matches a [[check]] name in .gitbutler/checks/*.toml
    pub head_oid: String,              // the OID the check ran against (gix::ObjectId.to_string())
    pub conclusion: String,            // serialized Conclusion (see §4); persisted as TEXT
    pub recorded_at: chrono::NaiveDateTime,
    pub metadata: String,             // JSON: { exit_code, duration_ms, runner_version, checkout_kind, truncated_output }
    pub signature: Option<String>,    // FORWARD-COMPAT SEAM ONLY — always None in v1; NOT a security claim
}
```

### Column rationale

| Column        | Type        | Why                                                                                                                                                                                                                                            |
| ------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `id`          | TEXT PK     | Per-run identity (mirrors `LocalReviewVerdict.id`). Append-only; never updated in place.                                                                                                                                                       |
| `name`        | TEXT        | Joins to the `[[check]]` `name` and to `[[required_check]].name`.                                                                                                                                                                              |
| `head_oid`    | TEXT        | **The SHA-binding key.** The exact OID the check ran against. The gate matches on `(name, head_oid == current_head)`.                                                                                                                          |
| `conclusion`  | TEXT        | Serialized `Conclusion` enum (§4). v1 producer emits `success` / `failure` / `timed_out`.                                                                                                                                                      |
| `recorded_at` | TIMESTAMP   | When the producer recorded the result. The gate may use it only for tie-breaking the latest row per `(name, head_oid)`.                                                                                                                        |
| `metadata`    | TEXT (JSON) | Observability payload: real `exit_code`, `duration_ms`, `runner_version`, `checkout_kind` (07), truncated tail of stdout/stderr. **Never** carries the conclusion itself — the conclusion is the typed column derived from the real exit code. |
| `signature`   | TEXT NULL   | **Forward-compat seam.** When producers eventually run off-host, a producer identity proof could live here. v1 writes `NULL` and the gate **ignores** it. Documented as NOT a v1 security control (01 §5).                                     |

> **`trigger` is NOT a `check_results` column.** A check's `trigger` (`on-commit` / `on-merge-attempt`) is a **definition-time** field in `.gitbutler/checks/*.toml`, never part of a result row. A consumer that displays a result's trigger (e.g. the desktop `CheckResultRow`, TR §10) **joins** the result's `name` to the check definition; it is never stored per-result.

### Access handle (mirror `LocalReviewVerdictsHandle`)

```rust
impl CheckResultsHandle<'_> {
    /// All results recorded for (name, head_oid), latest first.
    pub fn list_for(&self, name: &str, head_oid: &str) -> rusqlite::Result<Vec<CheckResult>>;
}
impl CheckResultsHandleMut<'_> {
    /// Append a recorded result (never UPDATE — append-only).
    pub fn insert(&mut self, row: CheckResult) -> rusqlite::Result<()>;
}
```

**Negative space (enforced by the type, see 04 §4):** there is **no**
`insert_with_conclusion(name, head_oid, conclusion)` public entry point that
accepts a caller-supplied `Conclusion` without a real run. The only code that
constructs a `CheckResult` with a non-`success` exit-derived conclusion is the
runner, from a real `std::process::ExitStatus`.

## §2 — `.gitbutler/checks/*.toml` (check definitions)

Ref-pinned config-as-code, read from the **target ref tree** via the exact
`read_config_blob` path the merge gate uses for `gates.toml`
(`crates/but-api/src/legacy/merge_gate.rs:211-242` — `find_reference` →
`peel_to_commit` → `tree()` → `lookup_entry_by_path` → `find_blob`). The working
tree is **never** consulted (parity with `governance_present`,
`crates/but-authz/src/config.rs:53`).

```toml
# .gitbutler/checks/rust.toml  (one or more [[check]] per file; all *.toml under
# .gitbutler/checks/ are unioned, like .gitbutler/gates.toml's [[gate]] list)

[[check]]
name = "cargo-test"                 # UNIQUE key across all .gitbutler/checks/*.toml
command = "cargo"                   # argv[0]
args = ["test", "-p", "but-checks"] # argv[1..]
trigger = "on-merge-attempt"        # "on-commit" | "on-merge-attempt" (UC-RUN-04)
success_exit_codes = [0]            # exit-code → success mapping (UC-DEFN-03); default [0]
timeout_seconds = 600              # hard cap (UC-RUN-05); fail timed_out past this
working_subdir = "."               # path under the checkout root to run in
```

### Wire type (mirror `GatesWire`/`GateWire` in merge_gate.rs:409-437)

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]          // same strictness as GatesWire
struct ChecksWire {
    #[serde(default)]
    check: Vec<CheckWire>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckWire {
    name: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    trigger: String,                   // validated into a Trigger enum post-parse
    #[serde(default = "default_success_codes")]
    success_exit_codes: Vec<i32>,
    #[serde(default = "default_timeout")]
    timeout_seconds: u64,
    #[serde(default = "dot")]
    working_subdir: String,
}
```

Malformed `.gitbutler/checks/*.toml` at the target ref → `config_invalid`
(mirrors merge_gate.rs:196-200 `config_invalid` on a `toml::from_str` error),
which the gate surfaces fail-closed (08 R-FAILCLOSED) as a denial with
`class: OperatorRequired` (a check definition is operator-owned).

## §3 — `[[required_check]]` in `.gitbutler/gates.toml`

The required-set policy lives in the **existing governance file**, ref-pinned,
mirroring the `[[gate]] type = "review"` requirement
(`crates/but-api/tests/confinement.rs:158-161`). This is the deliberate
parallel: a required review and a required check are both merge requirements on
the same target.

```toml
# .gitbutler/gates.toml (existing file — Check Runner ADDS this block)

[[branch]]
name = "main"
protected = true

[[gate]]                  # existing review requirement (governance)
branch = "main"
type = "review"
min_approvals = 1

[[required_check]]        # NEW (Check Runner) — required-check requirement
branch = "main"
name = "cargo-test"       # must match a [[check]] name in .gitbutler/checks/*.toml
```

### Wire type (extend `GatesWire`, normalize like `normalize_gates`)

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GatesWire {
    #[serde(default)] branch: Vec<BranchWire>,
    #[serde(default)] gate: Vec<GateWire>,
    #[serde(default)] required_check: Vec<RequiredCheckWire>,   // NEW
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredCheckWire {
    branch: String,
    name: String,
}
```

A `[[required_check]]` naming a `name` that does **not** resolve to any
`[[check]]` in `.gitbutler/checks/*.toml` is `config_invalid`. Copy the
**enforcing** pattern, not just the collector: `undefined_required_groups`
(merge_gate.rs:181-188) only **collects** the unsatisfiable names; the
fail-closed enforcement is the **caller's** `if !undefined_groups.is_empty() {
return Err(MergeGateError { … }) }` block at merge_gate.rs:62-76. The
implementer must mirror **merge_gate.rs:62-76 (the collection + the caller's
`is_empty()` → `MergeGateError` denial)** so an unsatisfiable required-check fails
closed rather than vacuously passing.

## §4 — `Conclusion` (typed enum)

GitHub-compatible vocabulary, modeled on `CiConclusion`
(`crates/but-forge/src/ci.rs:169-181`) so the forge-CI producer-zero path (02 §6)
maps cleanly. **v1 producers emit only `Success` / `Failure` / `TimedOut`**; the
remaining variants exist for forward compatibility with the existing forge-CI
cache vocabulary.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Success,        // exit code ∈ success_exit_codes
    Failure,        // exit code ∉ success_exit_codes (real ExitStatus)
    TimedOut,       // killed at timeout_seconds
    // --- forward-compat only; v1 local runner never emits these ---
    Cancelled,
    Neutral,
    Skipped,
    ActionRequired,
    Unknown,
}

impl Conclusion {
    /// The ONLY constructor that maps a real process exit to a conclusion.
    pub fn from_exit(status: std::process::ExitStatus, success_codes: &[i32]) -> Self { /* … */ }
    /// Does this conclusion satisfy a required-check at the gate?
    pub fn is_passing(self) -> bool { matches!(self, Conclusion::Success) }
}
```

The gate treats **only `Success`** as passing. `Neutral`/`Skipped` are **not**
passing for a _required_ check (a required check that no-ops must not satisfy the
gate — fail-closed, 08 R-FAILCLOSED).

## Cross-references

- Reproducibility & no-crypto stance: [`01-architecture-posture.md`](./01-architecture-posture.md) §3, §5
- The negative-space rule that protects the conclusion column: [`04-api-design.md`](./04-api-design.md) §4
- SHA-binding match semantics at the gate: [`01-architecture-posture.md`](./01-architecture-posture.md) §4, [`08-technical-risks.md`](./08-technical-risks.md) R-SHARESET
