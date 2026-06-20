# MGMT-UI-008: GroupsList (ExpandableSection per group; create/grant/add-member)

> **Red-Hat Remediation (cycle 1):** Resolved S2 (CRITICAL) — AC-6 + TC-7 added for T-MGMT-046 last-member-of-referenced-group warning banner before group_remove_member. Resolved S4 (MEDIUM) — AC-7 + TC-8 added for immediate group_revoke on Toggle-OFF. Resolved SEC4 (LOW) — AC-8 + TC-9 added for isReadOnly=true disabling Toggles with 0 SDK calls. Resolved S6 (MEDIUM) — DESIGN-MGMT-001, DESIGN-MGMT-002, DESIGN-MGMT-003 added to depends_on.

## What this does

The Groups tab: each group is an `ExpandableSection` showing its granted permission set (`Toggle`s) and members (`TagInput`). Create / grant / add-member / remove-member / delete map to `but group …` SDK calls (immediate — a group is the authority holder, not a per-principal batch). Delete is preceded by a `Modal` confirmation (B17). Removing the last member of a group referenced by a branch gate shows a warning banner before the SDK call (T-MGMT-046). An `EmptyStatePlaceholder` invites creating the first group. `isReadOnly=true` disables all Toggles so zero SDK calls fire on click.

## Why

Sprint 06a · PRD UC-MGMT-03 · criteria T-MGMT-012/013/014/016/046 · capability CAP-AUTHZ-01. The surface where an admin defines the teams that GRPS union semantics and gate `require_approval_from_group` reference.

## How to verify

PRIMARY **AC-1** — `pnpm test:ct:desktop -- GroupsListRows`: with 2 seeded groups, expanding `"eng"` shows the `contents:write` Toggle ON and member chips `"claude-agent"` and `"codex-agent"`. Full gate set in the spec below.

## Scope

- `apps/desktop/src/components/governance/GroupsList.svelte` (NEW)
- `apps/desktop/tests/governance/GroupsList.spec.ts` (NEW — CT specs)

<details>
<summary>▸ Full agent specification (TASK-TEMPLATE v5.2 — required reading for implementer + reviewer)</summary>

```
================================================================================
TASK: MGMT-UI-008 — GroupsList (ExpandableSection per group; create/grant/add-member)
================================================================================

TASK_TYPE:  FEATURE
STATUS:     Backlog
PRIORITY:   P1
EFFORT:     M  (75 min)
AGENT:      implementer=sveltekit-implementer | reviewer=sveltekit-reviewer
PROPOSED-BY: sveltekit-planner
SPRINT:     ./SPRINT.md
PRD_REFS:   UC-MGMT-03, T-MGMT-012, T-MGMT-013, T-MGMT-014, T-MGMT-016, T-MGMT-046
CAPABILITIES: CAP-AUTHZ-01

RUNTIME_COMMANDS:
  test:  pnpm test:ct:desktop -- GroupsListRows
  check: pnpm -F @gitbutler/desktop check   |   lint: pnpm lint

--------------------------------------------------------------------------------
OUTCOME
--------------------------------------------------------------------------------
GroupsList with 2 seeded groups (eng: 2 members + contents:write; platform: 1 member) renders 2
ExpandableSections. Expanding eng shows the contents:write Toggle ON + two member chips. Creating a new
group calls group_create. "Delete group" shows a Modal confirmation before group_delete. Empty fixture
shows EmptyStatePlaceholder with "+ Create group". Removing the last member of a gate-referenced group
shows a warning banner BEFORE calling group_remove_member. Toggling a grant Toggle OFF calls group_revoke
immediately. isReadOnly=true disables all Toggles (0 SDK calls on click).

--------------------------------------------------------------------------------
🚫 CRITICAL CONSTRAINTS (Never tier — read before acting)
--------------------------------------------------------------------------------
- [MUST] group_create on "+ New group" confirm; group_grant/group_revoke on a grant Toggle change;
  group_add_member/group_remove_member on member chip add/remove — IMMEDIATE SDK calls (the group is the
  authority holder, not a per-principal batch).
- [MUST] group_delete preceded by a Modal confirmation showing "Remove group X? N principals will lose
  inherited permissions." (B17).
- [MUST] Removing the LAST member of a group referenced by a branch gate's require_approval_from_group
  MUST show a warning banner BEFORE calling group_remove_member (T-MGMT-046); the user must confirm
  or cancel; the SDK call fires only on confirmation.
- [MUST] Toggling a granted permission Toggle OFF in an expanded group fires group_revoke('group','perm')
  immediately (no batch).
- [MUST] With isReadOnly=true all grant Toggles are disabled; 0 group_* SDK calls fire on any Toggle click.
- [MUST] EmptyStatePlaceholder shown when group_list returns no groups; all writes go through the but-sdk.
- [NEVER] NEVER batch group grant/member changes behind a Save button (unlike PrincipalEditor); NEVER skip
  the delete confirmation dialog; NEVER write .gitbutler/permissions.toml directly from the renderer;
  NEVER fire group_remove_member for a gate-referenced last-member without a warning+confirmation step.
- [STRICTLY] No relative imports — @gitbutler/ references. No console.log. Component-scoped $state for expand/dialog.

--------------------------------------------------------------------------------
DONE WHEN
--------------------------------------------------------------------------------
- [ ] AC-1 [PRIMARY]: groups listed as ExpandableSection with grants + members
- [ ] AC-2: create/grant/add-member issue correct but group_* SDK calls
- [ ] AC-3: empty state shows EmptyStatePlaceholder with create-first-group action
- [ ] AC-4: delete group shows confirmation dialog before group_delete
- [ ] AC-5: removing a member chip calls group_remove_member
- [ ] AC-6: removing LAST member of gate-referenced group shows warning banner before SDK call (T-MGMT-046)
- [ ] AC-7: toggling a grant Toggle OFF fires group_revoke immediately
- [ ] AC-8: isReadOnly=true disables Toggles; 0 group_* SDK calls fire on click
- [ ] All verification gates pass; only write_allowed files modified

--------------------------------------------------------------------------------
ACCEPTANCE CRITERIA (each behavioral AC carries a scenario — see REQUIREMENT-CONTRACT)
--------------------------------------------------------------------------------
AC-1 [PRIMARY]: groups listed as ExpandableSection with grants and members
  GIVEN: GroupsList mounted with 2 seeded groups
  WHEN:  the component renders and user expands "eng"
  THEN:  an ExpandableSection for "eng"; expanding shows contents:write Toggle ON + chips "claude-agent","codex-agent"
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListRows

AC-2: create/grant/add-member issue correct but-sdk group_* calls
  GIVEN: GroupsList mounted with seeded groups
  WHEN:  user (a) creates "security", (b) grants merge in eng
  THEN:  (a) group_create("security") called; (b) group_grant("eng","merge") called
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListSDKCalls

AC-3: empty state shows EmptyStatePlaceholder with create-first-group action
  GIVEN: group_list returns []
  WHEN:  the component renders
  THEN:  EmptyStatePlaceholder with "No groups yet" + "+ Create group" action
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListEmpty

AC-4: delete group shows confirmation dialog before issuing group_delete
  GIVEN: seeded groups
  WHEN:  user clicks "Delete group" on eng
  THEN:  a confirmation dialog mentioning "eng" appears; confirming calls group_delete("eng"); canceling does not
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListDeleteConfirm

AC-5: removing a member chip calls group_remove_member
  GIVEN: seeded groups; eng expanded
  WHEN:  user clicks ✕ on the "codex-agent" chip (eng still has "claude-agent" — not the last member)
  THEN:  group_remove_member("eng","codex-agent") is called and the chip is removed from the UI
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListRemoveMember

AC-6: removing LAST member of a gate-referenced group shows warning banner BEFORE group_remove_member (T-MGMT-046)
  GIVEN: GroupsList mounted with seeded_last_member_gate_ref (group "eng" has exactly 1 member "claude-agent";
         a branch gate references eng via require_approval_from_group)
  WHEN:  user clicks ✕ on the "claude-agent" chip (the last member)
  THEN:  a warning banner appears BEFORE the group_remove_member SDK call stating that this group is
         referenced by a gate; group_remove_member is NOT called until the user confirms; on cancel,
         the chip remains and no SDK call is made
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListLastMemberWarning

AC-7: toggling a grant Toggle OFF fires group_revoke immediately
  GIVEN: seeded groups; eng expanded; contents:write Toggle is ON
  WHEN:  user clicks the contents:write Toggle to OFF
  THEN:  group_revoke("eng","contents:write") is called immediately (no batch/Save step)
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListRevokeToggle

AC-8: isReadOnly=true disables Toggles; 0 group_* SDK calls fire on click
  GIVEN: GroupsList mounted with seeded groups and isReadOnly=true
  WHEN:  user attempts to click the contents:write Toggle in eng
  THEN:  all grant Toggles carry the disabled attribute; 0 group_grant / group_revoke calls are issued
  TEST_TIER: integration   VERIFICATION_SERVICE: desktop-ct-harness
  VERIFY: pnpm test:ct:desktop -- GroupsListReadOnly

--------------------------------------------------------------------------------
TEST CRITERIA (boolean; maps to ACs)
--------------------------------------------------------------------------------
- TC-1 (-> AC-1): GroupsList with seeded groups renders an ExpandableSection per group showing grants and member chips
    VERIFY: pnpm test:ct:desktop -- GroupsListRows
- TC-2 (-> AC-2): clicking "+ New group" and confirming calls group_create with the entered name
    VERIFY: pnpm test:ct:desktop -- GroupsListSDKCalls
- TC-3 (-> AC-3): group_list returning [] renders EmptyStatePlaceholder with create action
    VERIFY: pnpm test:ct:desktop -- GroupsListEmpty
- TC-4 (-> AC-4): clicking "Delete group" shows confirmation dialog before calling group_delete
    VERIFY: pnpm test:ct:desktop -- GroupsListDeleteConfirm
- TC-5 (-> AC-5): clicking ✕ on a member chip calls group_remove_member with the group + member name
    VERIFY: pnpm test:ct:desktop -- GroupsListRemoveMember
- TC-6 (-> AC-1): pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-008 lands
    VERIFY: pnpm -F @gitbutler/desktop check
- TC-7 (-> AC-6): warning banner appears before group_remove_member for the last member of a gate-referenced group; canceling does not call the SDK
    VERIFY: pnpm test:ct:desktop -- GroupsListLastMemberWarning
- TC-8 (-> AC-7): toggling a grant OFF calls group_revoke immediately (no batch save)
    VERIFY: pnpm test:ct:desktop -- GroupsListRevokeToggle
- TC-9 (-> AC-8): with isReadOnly=true all Toggles are disabled and 0 group_* SDK calls fire on click
    VERIFY: pnpm test:ct:desktop -- GroupsListReadOnly

--------------------------------------------------------------------------------
CAPABILITY BOUNDARY
--------------------------------------------------------------------------------
touches: CAP-AUTHZ-01
provides: GroupsList.svelte — Groups tab with ExpandableSection per group, grant toggles, member TagInput, create/delete
consumes: MGMT-IPC-004 (group_create/grant/revoke/add-member/remove-member/delete SDK); MGMT-UI-003 (readonly prop);
          apps/desktop ExpandableSection/ReduxResult; packages/ui Toggle/TagInput/Badge/EmptyStatePlaceholder/Button/KebabButton/Modal
boundary_contracts:
  - group writes are immediate SDK calls (the group is the authority holder); delete requires a Modal confirmation;
    last-member-of-gate-referenced-group requires a warning banner + confirmation before SDK call (T-MGMT-046);
    group_revoke fires immediately on Toggle-OFF; isReadOnly=true disables all Toggles (0 SDK calls)

--------------------------------------------------------------------------------
SCOPE (file-level write permissions)
--------------------------------------------------------------------------------
writeAllowed:
  - apps/desktop/src/components/governance/GroupsList.svelte (NEW)
  - apps/desktop/tests/governance/GroupsList.spec.ts (NEW)
writeProhibited:
  - apps/desktop/src/components/shared/ExpandableSection.svelte (reuse as-is)
  - packages/ui components (reuse as-is)
  - apps/desktop/src/components/governance/PrincipalsList.svelte (owned by MGMT-UI-006)
  - any direct .gitbutler/permissions.toml write from the renderer; any file not in write_allowed

--------------------------------------------------------------------------------
READING LIST
--------------------------------------------------------------------------------
1. apps/desktop/src/components/shared/ExpandableSection.svelte — accordion pattern (label/summary/content snippets)
2. packages/ui/src/lib/components/Toggle.svelte — grant permission toggles (disabled=isReadOnly)
3. packages/ui/src/lib/components/TagInput.svelte — member chip add/remove
4. packages/ui/src/lib/components/EmptyStatePlaceholder.svelte — no-groups empty state
5. packages/ui/src/lib/components/Modal.svelte — confirmation dialog for delete (B17) + last-member warning (T-MGMT-046)
6. packages/ui/src/lib/components/KebabButton.svelte — per-group overflow menu
7. packages/ui/src/lib/components/InfoMessage.svelte — warning variant for last-member-of-gate-referenced-group
8. .spec/prds/governance/10-technical-requirements/10-ui-infrastructure.md — Groups-tab wireframe (ExpandableSection layout)
9. .spec/prds/governance/08-uc-mgmt.md UC-MGMT-03 — group create/grant/add-member/delete spec + B11/B17 confirmation; T-MGMT-046 last-member gate-ref warning

--------------------------------------------------------------------------------
VERIFICATION GATES
--------------------------------------------------------------------------------
- pnpm test:ct:desktop -- GroupsListRows                 -> Exit 0
- pnpm test:ct:desktop -- GroupsListSDKCalls             -> Exit 0
- pnpm test:ct:desktop -- GroupsListEmpty                -> Exit 0
- pnpm test:ct:desktop -- GroupsListDeleteConfirm        -> Exit 0
- pnpm test:ct:desktop -- GroupsListRemoveMember         -> Exit 0
- pnpm test:ct:desktop -- GroupsListLastMemberWarning    -> Exit 0
- pnpm test:ct:desktop -- GroupsListRevokeToggle         -> Exit 0
- pnpm test:ct:desktop -- GroupsListReadOnly             -> Exit 0
- pnpm -F @gitbutler/desktop check                       -> Exit 0
- pnpm lint                                              -> Exit 0

--------------------------------------------------------------------------------
DESIGN / CODE PATTERN
--------------------------------------------------------------------------------
references: DESIGN-MGMT-001 (Groups-tab region mapping); DESIGN-MGMT-002 (per-group pending Badge in summary); DESIGN-MGMT-003 (isReadOnly treatment)
notes (from enrichment): GroupsList renders one ExpandableSection per group. The summary shows member count +
  a pending Badge (warning/soft) if the group has staged changes. The content shows GRANTED (a Toggle per
  permission, disabled=isReadOnly), MEMBERS (TagInput, readonly=isReadOnly), and a "Delete group" Button
  (danger, omitted in read-only) that opens a Modal ("Remove group {name}? N principals will lose inherited
  permissions."). "+ New group" disabled=isReadOnly. Empty -> EmptyStatePlaceholder.
  LAST-MEMBER GATE REF (T-MGMT-046): before calling group_remove_member, check if the member is the last in a
  group that is referenced by any branch gate (require_approval_from_group). If so, show a warning InfoMessage
  or Modal explaining that removing this member will leave the gate's required group empty. Require the user to
  explicitly confirm. Only call group_remove_member after confirmation.
  IMMEDIATE REVOKE: toggling a grant Toggle OFF calls group_revoke immediately (no batch). The same Toggle ON
  calls group_grant immediately.
  READ-ONLY: with isReadOnly=true, all Toggle elements carry disabled=true; the SDK must receive 0 calls on
  any Toggle click.
pattern: ExpandableSection per group; group writes immediate (group_create/grant/revoke/add-member/remove-member/delete); Modal confirm on delete; warning + confirm for last-member-of-gate-ref
pattern_source: apps/desktop/src/components/shared/ExpandableSection.svelte (label/summary/content snippet slots)
anti_pattern: batching group changes behind a Save (unlike PrincipalEditor); skipping the delete confirmation; direct toml writes; a custom expandable; skipping the last-member gate-ref warning; enabled Toggles in read-only mode

--------------------------------------------------------------------------------
AGENT ASSIGNMENT
--------------------------------------------------------------------------------
implementer: sveltekit-implementer
reviewer: sveltekit-reviewer
coding_standards: apps/desktop/AGENTS.md, frontend.md

--------------------------------------------------------------------------------
DEPENDENCIES
--------------------------------------------------------------------------------
Depends on: MGMT-UI-003 (page host + readonly prop), MGMT-IPC-004 (group_* SDK);
            DESIGN-MGMT-001 (Groups-tab IA), DESIGN-MGMT-002 (pending Badge), DESIGN-MGMT-003 (isReadOnly treatment)
Blocks:     none
```

<!-- REQUIREMENT-CONTRACT v1 -->
<!--
{
  "version": "1",
  "task_id": "MGMT-UI-008",
  "proposed_by": "sveltekit-planner",
  "verification_policy": {
    "requires_tests": true,
    "requires_red_evidence": true,
    "requires_seeded_evidence": true
  },
  "fixtures": {
    "seeded_groups": {
      "description": "group_list fixture returns 2 groups via the but-sdk route the desktop CT harness mounts against.",
      "seed_method": "ui_flow",
      "records": [
        "eng: grants [contents:write], members [claude-agent, codex-agent]",
        "platform: grants [], members [cursor-bot]"
      ]
    },
    "seeded_groups_empty": {
      "description": "group_list fixture returns 0 groups.",
      "seed_method": "ui_flow",
      "records": [
        "0 groups configured"
      ]
    },
    "seeded_last_member_gate_ref": {
      "description": "group_list returns eng with exactly 1 member (claude-agent). A branch gate references eng via require_approval_from_group. Used to verify the T-MGMT-046 warning before group_remove_member.",
      "seed_method": "ui_flow",
      "records": [
        "eng: grants [contents:write], members [claude-agent]",
        "branch gate: require_approval_from_group = eng"
      ]
    },
    "seeded_groups_readonly": {
      "description": "group_list returns seeded_groups but the component is mounted with isReadOnly=true. Used to verify Toggle disabled state and 0 SDK calls.",
      "seed_method": "ui_flow",
      "records": [
        "eng: grants [contents:write], members [claude-agent, codex-agent]",
        "isReadOnly = true"
      ]
    }
  },
  "requirements": [
    {
      "id": "AC-1",
      "type": "acceptance_criterion",
      "primary": true,
      "description": "GIVEN 2 seeded groups WHEN user expands eng THEN an ExpandableSection for eng shows contents:write Toggle ON + member chips claude-agent, codex-agent",
      "verify": "pnpm test:ct:desktop -- GroupsListRows",
      "scenario": {
        "id": "AC-1",
        "primary": true,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "groups rendered as flat rows without ExpandableSection",
            "member chips are a static shell",
            "the list is disconnected from group_list"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "expand the `\"eng\"` ExpandableSection"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `ExpandableSection` for `\"eng\"`",
                "the `contents:write` Toggle ON inside the eng section",
                "member chips `\"claude-agent\"` and `\"codex-agent\"`"
              ],
              "must_not_observe": [
                "groups rendered as flat rows without the accordion",
                "member chips absent (none)"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-2",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded groups WHEN user creates 'security' and grants merge in eng THEN group_create('security') and group_grant('eng','merge') are called",
      "verify": "pnpm test:ct:desktop -- GroupsListSDKCalls",
      "scenario": {
        "id": "AC-2",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "SDK calls are not issued or have wrong args",
            "a static form with no SDK wiring",
            "create deferred to a batch save (no-op)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "click `\"+ New group\"`, enter `\"security\"`, confirm create"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_create(\"security\")` is called"
              ],
              "must_not_observe": [
                "create deferred to a batch save",
                "no SDK call (none) on confirm"
              ]
            }
          },
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "expand eng",
                "click the merge Toggle ON"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_grant(\"eng\", \"merge\")` is called"
              ],
              "must_not_observe": [
                "no SDK call (none) on toggle change",
                "a deferred/batched grant"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-3",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN group_list returns [] WHEN GroupsList renders THEN EmptyStatePlaceholder with 'No groups yet' + '+ Create group'",
      "verify": "pnpm test:ct:desktop -- GroupsListEmpty",
      "scenario": {
        "id": "AC-3",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "empty array renders an empty container without EmptyStatePlaceholder",
            "the placeholder is static and shows even with groups present",
            "the list is disconnected from group_list"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups_empty",
            "action": {
              "actor": "user",
              "steps": [
                "mount GroupsList against the empty group_list fixture",
                "observe the rendered state"
              ]
            },
            "end_state": {
              "must_observe": [
                "an `EmptyStatePlaceholder` with `\"No groups yet\"` text",
                "a `\"+ Create group\"` action button"
              ],
              "must_not_observe": [
                "an empty container with no placeholder",
                "an ExpandableSection with `(0)` groups"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-4",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded groups WHEN user clicks 'Delete group' on eng THEN a confirmation dialog mentioning eng appears and confirming calls group_delete('eng')",
      "verify": "pnpm test:ct:desktop -- GroupsListDeleteConfirm",
      "scenario": {
        "id": "AC-4",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "group_delete is called without a confirmation dialog",
            "the dialog is a static no-op",
            "delete proceeds with no confirmation step"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "click `\"Delete group\"` on eng",
                "confirm in the dialog"
              ]
            },
            "end_state": {
              "must_observe": [
                "a confirmation dialog appears mentioning `\"eng\"`",
                "confirming calls `group_delete(\"eng\")`"
              ],
              "must_not_observe": [
                "group_delete called before the user confirms",
                "no confirmation dialog (none) shown"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-5",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded groups, eng expanded (2 members) WHEN user clicks \u2715 on the codex-agent chip THEN group_remove_member('eng','codex-agent') is called and the chip is removed",
      "verify": "pnpm test:ct:desktop -- GroupsListRemoveMember",
      "scenario": {
        "id": "AC-5",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "the chip is removed from the UI without calling group_remove_member",
            "a static chip list",
            "the remove is a no-op"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "expand eng",
                "click the \u2715 on the `\"codex-agent\"` member chip"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_remove_member(\"eng\", \"codex-agent\")` is called",
                "the `\"codex-agent\"` chip is removed from the UI"
              ],
              "must_not_observe": [
                "the chip removed without an SDK call",
                "no member chips (none) shown"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-6",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_last_member_gate_ref (eng has 1 member claude-agent; a branch gate references eng) WHEN user clicks \u2715 on claude-agent THEN a warning banner appears BEFORE group_remove_member is called; canceling fires 0 SDK calls (T-MGMT-046)",
      "verify": "pnpm test:ct:desktop -- GroupsListLastMemberWarning",
      "scenario": {
        "id": "AC-6",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "group_remove_member is called before the warning banner is shown (no gate check)",
            "the warning is absent (static)",
            "the remove proceeds silently with no warning (no-op gate check)"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_last_member_gate_ref",
            "action": {
              "actor": "user",
              "steps": [
                "expand eng",
                "click the \u2715 on the `\"claude-agent\"` member chip (the last member)"
              ]
            },
            "end_state": {
              "must_observe": [
                "a warning  or dialog whose text contains `\"eng\"` and `\"gate\"` (or `\"require_approval_from_group\"`)",
                "`0` `group_remove_member` SDK calls fired before the warning is shown"
              ],
              "must_not_observe": [
                "group_remove_member called without a warning",
                "no warning banner shown (none)"
              ]
            }
          },
          {
            "start_ref": "seeded_last_member_gate_ref",
            "action": {
              "actor": "user",
              "steps": [
                "click \u2715 on claude-agent",
                "click Cancel in the warning"
              ]
            },
            "end_state": {
              "must_observe": [
                "the `\"claude-agent\"` chip remains in the UI"
              ],
              "must_not_observe": [
                "`group_remove_member` called after Cancel",
                "the chip removed on Cancel",
                "the warning absent `(0)` warning elements"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-7",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded groups eng expanded with contents:write ON WHEN user clicks the contents:write Toggle to OFF THEN group_revoke('eng','contents:write') is called immediately (no batch)",
      "verify": "pnpm test:ct:desktop -- GroupsListRevokeToggle",
      "scenario": {
        "id": "AC-7",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "group_revoke is deferred to a batch Save (unlike PrincipalEditor)",
            "a static Toggle that does not call group_revoke",
            "no SDK call on Toggle-OFF (none)"
          ]
        },
        "evidence": {
          "artifact_type": "stdout",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups",
            "action": {
              "actor": "user",
              "steps": [
                "expand eng",
                "click the `contents:write` Toggle to OFF"
              ]
            },
            "end_state": {
              "must_observe": [
                "`group_revoke(\"eng\", \"contents:write\")` is called immediately after the click"
              ],
              "must_not_observe": [
                "the Toggle OFF with no SDK call (none)",
                "a batch/Save required before the SDK call fires"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "AC-8",
      "type": "acceptance_criterion",
      "primary": false,
      "description": "GIVEN seeded_groups_readonly (isReadOnly=true) WHEN user attempts to click a grant Toggle THEN all Toggles carry the disabled attribute and 0 group_grant / group_revoke calls are issued",
      "verify": "pnpm test:ct:desktop -- GroupsListReadOnly",
      "scenario": {
        "id": "AC-8",
        "primary": false,
        "tier": "visible",
        "test_tier": "integration",
        "verification_service": "desktop-ct-harness",
        "negative_control": {
          "would_fail_if": [
            "a Toggle is interactive when isReadOnly=true",
            "group_grant/group_revoke is called on click in read-only mode",
            "the disabled attribute is absent on Toggles when isReadOnly=true"
          ]
        },
        "evidence": {
          "artifact_type": "screenshot",
          "required_capture": true
        },
        "cases": [
          {
            "start_ref": "seeded_groups_readonly",
            "action": {
              "actor": "user",
              "steps": [
                "expand eng",
                "attempt to click the `contents:write` Toggle"
              ]
            },
            "end_state": {
              "must_observe": [
                "all grant Toggles carry `disabled=true`",
                "`0` group_grant / group_revoke SDK calls issued on click"
              ],
              "must_not_observe": [
                "an interactive (non-disabled) Toggle when isReadOnly=true",
                "a group_grant or group_revoke call on Toggle click in read-only mode",
                "`(0)` disabled Toggles \u2014 all must be disabled, not none"
              ]
            }
          }
        ]
      }
    },
    {
      "id": "TC-1",
      "type": "test_criterion",
      "description": "GroupsList with seeded groups renders an ExpandableSection per group showing grants and member chips",
      "verify": "pnpm test:ct:desktop -- GroupsListRows",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-2",
      "type": "test_criterion",
      "description": "clicking '+ New group' and confirming calls group_create with the entered name",
      "verify": "pnpm test:ct:desktop -- GroupsListSDKCalls",
      "maps_to_ac": "AC-2"
    },
    {
      "id": "TC-3",
      "type": "test_criterion",
      "description": "group_list returning [] renders EmptyStatePlaceholder with create action",
      "verify": "pnpm test:ct:desktop -- GroupsListEmpty",
      "maps_to_ac": "AC-3"
    },
    {
      "id": "TC-4",
      "type": "test_criterion",
      "description": "clicking 'Delete group' shows confirmation dialog before calling group_delete",
      "verify": "pnpm test:ct:desktop -- GroupsListDeleteConfirm",
      "maps_to_ac": "AC-4"
    },
    {
      "id": "TC-5",
      "type": "test_criterion",
      "description": "clicking \u2715 on a member chip calls group_remove_member with the group + member name",
      "verify": "pnpm test:ct:desktop -- GroupsListRemoveMember",
      "maps_to_ac": "AC-5"
    },
    {
      "id": "TC-6",
      "type": "test_criterion",
      "description": "pnpm -F @gitbutler/desktop check exits 0 after MGMT-UI-008 lands",
      "verify": "pnpm -F @gitbutler/desktop check",
      "maps_to_ac": "AC-1"
    },
    {
      "id": "TC-7",
      "type": "test_criterion",
      "description": "warning banner before group_remove_member for last member of gate-referenced group; Cancel fires 0 SDK calls",
      "verify": "pnpm test:ct:desktop -- GroupsListLastMemberWarning",
      "maps_to_ac": "AC-6"
    },
    {
      "id": "TC-8",
      "type": "test_criterion",
      "description": "Toggle OFF calls group_revoke immediately (no batch Save required)",
      "verify": "pnpm test:ct:desktop -- GroupsListRevokeToggle",
      "maps_to_ac": "AC-7"
    },
    {
      "id": "TC-9",
      "type": "test_criterion",
      "description": "isReadOnly=true -> all Toggles disabled; 0 group_grant / group_revoke SDK calls on click",
      "verify": "pnpm test:ct:desktop -- GroupsListReadOnly",
      "maps_to_ac": "AC-8"
    }
  ]
}
-->
</details>
