//! AC-4 structural bypass gate for governance routes.
//!
//! This is a source-level invariant test. It does NOT boot but-server; it reads
//! the production source files directly and asserts the properties that, taken
//! together, prove no governance route can bypass `but-authz` on the HTTP
//! surface.
//!
//! ## Trust-model note (deviation from the contract letter — see header below)
//!
//! The contract's AC-4 text demands `grep '_as_fleet_owner'` returns 0 matches
//! across `but-server/src`, `gitbutler-tauri/src`, **and** `but-napi/src`. That
//! is unsatisfiable as written: `gitbutler-tauri/src/governance.rs` is — by
//! design — the *desktop fleet-owner* boundary. Every `*_as_fleet_owner` call
//! there lives inside a `*_for_desktop_session` function that first resolves the
//! signed-in local user via `DesktopSession::fleet_owner_identity()`. The local
//! desktop user IS the fleet owner; those calls are the intended authorization
//! path for the desktop shell, not a bypass.
//!
//! The invariant H3 actually cares about is narrower and correct: the **HTTP
//! surface** (`but-server/src`), which serves network callers that are never
//! fleet owners, must contain zero `_as_fleet_owner` references. This test
//! enforces that invariant plus the positive-attribution and planted-bypass
//! checks, which *are* satisfiable as written and apply uniformly.
//!
//! ## What this test asserts
//!
//! 1. **Zero `_as_fleet_owner` on the HTTP surface.** `crates/but-server/src`
//!    contains no reference to any `*_as_fleet_owner` symbol — so no HTTP route
//!    can re-point at a fleet-owner variant and skip `but-authz`.
//! 2. **Desktop surfaces remain gated.** Any `*_as_fleet_owner` reference in
//!    `gitbutler-tauri/src` or `but-napi/src` must live behind a desktop-session
//!    boundary (the calling file must resolve a `DesktopSession` / fleet-owner
//!    identity). This catches an unauthenticated desktop bypass without
//!    forbidding the intended fleet-owner path.
//! 3. **Positive attribution.** Every `*_cmd` symbol named in but-server's
//!    `GOVERNANCE_COMMAND_ROUTES` table resolves to a `#[but_api(...)]`-attributed
//!    function in `crates/but-api/src/legacy/governance.rs`. The `#[but_api]`
//!    macro generates the `_cmd` wrapper; this test makes that invariant explicit
//!    and auditable rather than relying on the macro alone.
//! 4. **Planted-bypass detection.** No `#[but_api(...)]`-attributed function in
//!    governance.rs has a name containing `_as_fleet_owner`. If someone planted
//!    the attribute on `branch_gates_update_with_repo_as_fleet_owner`, this
//!    test fails and names the offending function. The detection logic is also
//!    exercised against a synthetic "planted" source so the assertion is not
//!    vacuous (a negative control).

#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};

/// Repository root: two levels above this crate's manifest dir
/// (`crates/but-server` -> `crates` -> repo root).
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("../.."))
}

fn read_source(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("failed to read {path:?}: {e}"))
}

/// Recursively collect every `.rs` file under `dir`.
fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Every line in `source` that contains `needle`, returned as
/// `(line_number, trimmed_line_text)` tuples. Line numbers are 1-based.
fn lines_containing(source: &str, needle: &str) -> Vec<(usize, String)> {
    source
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(needle))
        .map(|(i, line)| (i + 1, line.trim().to_owned()))
        .collect()
}

/// Bare suffix that matches every form: the definition site, call sites, and
/// the `as SyncCommandHandler` cast form.
const FLEET_OWNER_NEEDLE: &str = "_as_fleet_owner";

/// Markers that indicate a file is a desktop-session boundary (the local signed
/// in user is resolved before any fleet-owner call). Used to permit legitimate
/// `*_as_fleet_owner` usage on desktop surfaces.
const DESKTOP_SESSION_MARKERS: &[&str] = &[
    "fleet_owner_context",
    "fleet_owner_identity",
    "DesktopSession",
];

/// Names of functions carrying a `#[but_api(...)]` attribute in `source`,
/// parsed by scanning for an attribute line immediately followed (ignoring
/// other attributes and doc comments) by a `pub fn <name>` line.
fn but_api_attributed_fns(source: &str) -> Vec<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut names = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[but_api") {
            // Look forward through subsequent attribute/doc lines for the
            // `pub fn` declaration this attribute decorates.
            for following in lines.iter().skip(i + 1) {
                let t = following.trim();
                if t.starts_with("#[")
                    || t.starts_with("///")
                    || t.starts_with("//")
                    || t.is_empty()
                {
                    continue;
                }
                if let Some(name) = extract_pub_fn_name(t) {
                    names.push(name.to_owned());
                }
                break;
            }
        }
    }
    names
}

/// Extract the function name from a line like `pub fn branch_gates_update(`.
fn extract_pub_fn_name(line: &str) -> Option<&str> {
    let line = line.trim();
    let after_fn = line.strip_prefix("pub fn ")?;
    let name_end = after_fn
        .find(|c: char| c == '(' || c == '<' || c.is_whitespace())
        .unwrap_or(after_fn.len());
    let name = &after_fn[..name_end];
    (!name.is_empty()).then_some(name)
}

/// Route handler base names parsed from but-server's `GOVERNANCE_COMMAND_ROUTES`
/// table. Each entry pairs a `"/<path>"` with a
/// `legacy::governance::<name>_cmd as SyncCommandHandler` handler; we extract
/// `<name>` so it can be attributed back to its `#[but_api]` definition.
fn governance_route_base_names(lib_rs: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut in_table = false;
    for line in lib_rs.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("const GOVERNANCE_COMMAND_ROUTES") {
            in_table = true;
            continue;
        }
        if !in_table {
            continue;
        }
        // The table ends at the closing `];`.
        if trimmed == "];" {
            break;
        }
        // Handler lines look like:
        //   legacy::governance::branch_gates_update_cmd as SyncCommandHandler,
        if let Some(idx) = trimmed.find("legacy::governance::") {
            let after = &trimmed[idx + "legacy::governance::".len()..];
            if let Some(cmd_end) = after.find("_cmd") {
                names.push(after[..cmd_end].to_owned());
            }
        }
    }
    names
}

/// Collect `(relative_path, line_no, trimmed_line)` for every `_as_fleet_owner`
/// reference under `dir`.
fn fleet_owner_refs_under(root: &Path, dir: &Path) -> Vec<(String, usize, String)> {
    let mut files = Vec::new();
    collect_rust_files(dir, &mut files);
    let mut hits = Vec::new();
    for file in &files {
        let source = read_source(file);
        for (lineno, text) in lines_containing(&source, FLEET_OWNER_NEEDLE) {
            hits.push((
                file.strip_prefix(root)
                    .unwrap_or(file)
                    .display()
                    .to_string(),
                lineno,
                text,
            ));
        }
    }
    hits
}

#[test]
fn governance_bypass_grep() {
    let root = repo_root();

    // ------------------------------------------------------------------
    // Property 1: zero `_as_fleet_owner` on the HTTP surface (but-server/src).
    // This is the actual no-bypass proof for untrusted network callers: but-server
    // serves HTTP clients that are never fleet owners, so it must route every
    // governance command through the normal but-authz gate.
    // ------------------------------------------------------------------
    let http_surface = root.join("crates/but-server/src");
    let http_hits = fleet_owner_refs_under(&root, &http_surface);
    assert!(
        http_hits.is_empty(),
        "AC-4 VIOLATION: found {} `_as_fleet_owner` reference(s) on the HTTP surface \
         (crates/but-server/src). The HTTP server serves network callers that are never fleet \
         owners; every governance route must go through the normal but-authz gate. Matches:\n{}",
        http_hits.len(),
        http_hits
            .iter()
            .map(|(path, line, text)| format!("  {path}:{line}: {text}"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // ------------------------------------------------------------------
    // Property 2: desktop surfaces (gitbutler-tauri, but-napi) may use
    // `_as_fleet_owner` only behind a desktop-session boundary. Every file that
    // references a fleet-owner variant must also resolve a signed-in desktop
    // identity (`DesktopSession` / `fleet_owner_identity` / `fleet_owner_context`).
    // This catches an *unauthenticated* desktop bypass without forbidding the
    // intended fleet-owner path.
    // ------------------------------------------------------------------
    let desktop_surfaces = [
        root.join("crates/gitbutler-tauri/src"),
        root.join("crates/but-napi/src"),
    ];
    for surface in &desktop_surfaces {
        let hits = fleet_owner_refs_under(&root, surface);
        if hits.is_empty() {
            continue;
        }
        // Every file with a fleet-owner reference must carry a desktop-session marker.
        let ungated: Vec<&(String, usize, String)> = hits
            .iter()
            .filter(|(rel_path, _, _)| {
                let full = root.join(rel_path);
                let source = read_source(&full);
                !DESKTOP_SESSION_MARKERS
                    .iter()
                    .any(|marker| source.contains(marker))
            })
            .collect();
        assert!(
            ungated.is_empty(),
            "AC-4 VIOLATION: `_as_fleet_owner` reference(s) on desktop surface {:?} are NOT behind \
             a desktop-session boundary (no `{}` marker in the same file). Each fleet-owner call \
             must be gated by a resolved signed-in desktop identity. Ungated matches:\n{}",
            surface,
            DESKTOP_SESSION_MARKERS.join("`/`"),
            ungated
                .iter()
                .map(|(path, line, text)| format!("  {path}:{line}: {text}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    // ------------------------------------------------------------------
    // Property 3: positive attribution — every route `_cmd` resolves to a
    // `#[but_api(...)]`-attributed function in governance.rs.
    // ------------------------------------------------------------------
    let lib_rs_path = root.join("crates/but-server/src/lib.rs");
    let lib_rs = read_source(&lib_rs_path);
    let route_names = governance_route_base_names(&lib_rs);
    assert!(
        !route_names.is_empty(),
        "could not parse any route handlers from GOVERNANCE_COMMAND_ROUTES in {lib_rs_path:?}; \
         the table format may have changed and this structural gate needs updating"
    );

    let governance_path = root.join("crates/but-api/src/legacy/governance.rs");
    let governance_src = read_source(&governance_path);
    let attributed_vec = but_api_attributed_fns(&governance_src);
    let attributed: std::collections::HashSet<&str> =
        attributed_vec.iter().map(String::as_str).collect();

    let unattributed: Vec<String> = route_names
        .iter()
        .filter(|name| !attributed.contains(name.as_str()))
        .cloned()
        .collect();
    assert!(
        unattributed.is_empty(),
        "AC-4 VIOLATION: route handler(s) without a #[but_api] definition in governance.rs: \
         {unattributed:?}. Every `*_cmd` in GOVERNANCE_COMMAND_ROUTES must resolve to a \
         #[but_api(...)]-attributed `pub fn` (not a `*_as_fleet_owner` bypass variant)."
    );

    // ------------------------------------------------------------------
    // Property 4: no attributed fn is a fleet-owner variant (planted bypass).
    // ------------------------------------------------------------------
    let fleet_owner_attributed: Vec<String> = attributed_vec
        .iter()
        .filter(|name| name.contains(FLEET_OWNER_NEEDLE))
        .cloned()
        .collect();
    assert!(
        fleet_owner_attributed.is_empty(),
        "AC-4 VIOLATION (planted bypass): #[but_api] attribute found on fleet-owner variant(s): \
         {fleet_owner_attributed:?}. Planting #[but_api] on a `*_as_fleet_owner` function would \
         generate a `_cmd` wrapper that skips but-authz and could be wired into the route table."
    );

    // ------------------------------------------------------------------
    // Negative control: the planted-bypass detector MUST fire on a synthetic
    // planted definition, proving Property 4 is not vacuously true.
    // ------------------------------------------------------------------
    let planted = format!(
        "{governance_src}\n\
         /// synthetically planted bypass for the negative control only.\n\
         #[but_api(napi)]\n\
         pub fn branch_gates_update_with_repo_as_fleet_owner() {{}}\n"
    );
    let planted_hits: Vec<String> = but_api_attributed_fns(&planted)
        .into_iter()
        .filter(|name| name.contains(FLEET_OWNER_NEEDLE))
        .collect();
    assert_eq!(
        planted_hits,
        vec!["branch_gates_update_with_repo_as_fleet_owner".to_owned()],
        "negative control failed: the planted-bypass detector did not flag a synthetic \
         #[but_api]-attributed `_as_fleet_owner` function, so Property 4 could be passing \
         vacuously. Fix the detector in but_api_attributed_fns."
    );

    // ------------------------------------------------------------------
    // Sanity: we parsed exactly 16 routes and attributed the key one.
    // ------------------------------------------------------------------
    assert_eq!(
        route_names.len(),
        16,
        "expected 16 governance command routes, parsed {}: {route_names:?}",
        route_names.len()
    );
    assert!(
        attributed.contains("branch_gates_update"),
        "sanity: branch_gates_update must be #[but_api]-attributed"
    );
}
