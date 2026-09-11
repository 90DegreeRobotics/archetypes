# Plan: Architect Workshop — 2026-09-11 00:35

## Status
PENDING

## Goal

Deliver Steps 7-8 of `plan_2026-09-11_0015_inner_castle_grounded_capabilities.md`: a player-owned
`BuildIntent` planning artifact, created/edited/sequenced/closed at a real workshop table inside
the Architect room. Local-first, append-only, every persisted action explicitly confirmed by the
player, and a visible failure whenever a write cannot happen.

This plan begins with a pre-flight audit rather than a feature list, because the audit found that
**the workshop table is not reachable on foot today** and that a prompt written the way the
existing ones are written is **not visible on screen**. Building the capability before fixing that
would produce a room that only passes review from flight mode, which is the exact failure the
governing plan's execution contract forbids.

---

## Pre-flight audit (performed 2026-09-11 against current `main`)

### Verified correct — no action needed

| Claim | Evidence |
|---|---|
| Doorway gap agrees between art and collision | `build_wall_ring_mesh` takes `door_width_rad` and uses `half_door = door_width_rad * 0.5` (`world.rs:1554`); rooms pass `0.48`, so the visual gap is ±0.24 rad. `camera.rs:419` exempts `diff.abs() < 0.24`. Exact match — the visible doorway is exactly the passable doorway. |
| The focus resolver is the right seam to extend | `interaction.rs::resolve_focus` already collects candidates and picks `min_by(distance, then priority)` with archetype = 0, altar = 1. A third device slots in without inventing a parallel mechanism. |
| Legacy world builder is inert | `setup_legacy_rotunda_world` (`world.rs:429`) is `#[allow(dead_code)]` and never registered; only `setup_inner_world` runs at `OnEnter(Loading)`. |

### Verified broken — blocking this feature

**B1 — Room interiors are not walkable past their own center.**
`camera.rs:407-426` pushes any *walking* player within 17.1 m of a room center, outside the ±0.24 rad
doorway wedge, **outward** to radius 17.1. For the Architect room (center `(0,-62)`, radial `(0,-1)`,
door bearing `+π/2`):

| Object | Position | Bearing from room centre | `diff` vs door | Result |
|---|---|---|---|---|
| Architect embodiment | `(0, -67)` | `-π/2` | `π` | ejected to `(0, -79.1)` |
| `Architect_FunctionalCounter` (the table) | `(0, -70.6)` | `-π/2` | `π` | ejected to `(0, -79.1)` |

That is an ~8.5 m teleport into the far wall the instant the player walks past the room's centre.
The block exists only in the `LocomotionMode::Walking` arm — `Flying` is unaffected, which is why
every capture to date looks fine: the capture harness flies.
Empirical confirmation: `artifacts/visual-proof/encounter-memory-2026-09-11/02_room_architect.png`
is a `[STATUS: FREE FLIGHT]` frame.

Conversations only work today because `ENCOUNTER_RANGE` (6.75) exceeds the statue's 5.0 m offset, so
the prompt is triggerable from the walkable centre. That is luck, not design, and a 3 m device range
does not get the same free pass.

*Fix:* treat the ring as a shell, not an inward ejector. The wall occupies roughly radius 16.6-17.6
(ring 17.1, thickness 1.05). Collide only inside that band — inside-half pushes to 16.6, outside-half
pushes to 17.6, doorway wedge exempt. A player on a bridge at r≈25 must remain untouched; a naive
"push inward" inversion would snap them onto the wall, so the band is load-bearing, not cosmetic.

**B2 — Two embodiment collision capsules are mirrored.**
Comparing the hand-typed `character_obstacles` list (`camera.rs:388-395`) against the positions
`world.rs:300` actually spawns (`center + radial * 5.0`):

| Room | Figure spawned at | Obstacle listed at | Verdict |
|---|---|---|---|
| Sentinel | `(58.02, -33.50)` | `(58.02, -33.5)` | ok |
| Explorer | `(58.02, 33.50)` | `(58.02, 33.5)` | ok |
| Mentor | `(-58.02, 33.50)` | `(-58.02, 33.5)` | ok |
| Oracle | `(-58.02, -33.50)` | `(-58.02, -33.5)` | ok |
| **Architect** | `(0, -67.00)` | `(0, -57.0)` | **mirrored, 10 m off** |
| **Empath** | `(0, 67.00)` | `(0, 57.0)` | **mirrored, 10 m off** |

The two wrong entries are exactly the rooms at ±π/2 where `cos(angle) ≈ 0` — a sign flip in
hand-computed values. Effect: an invisible 1.5 m pillar sitting between the Architect doorway and the
room centre (directly on the walk to the workshop), and no collision at all on the statue itself.

*Fix:* derive the obstacle list from the same expression `world.rs` spawns with, instead of literals,
and add a test asserting every embodiment obstacle equals its spawn position.

**B3 — The hint line has competing writers, and the proximity prompt loses.**
`extraction.rs:42-56` overwrites `InnerChambersHint` **every frame** (truth-node text, else
`locomotion_hud_text()`), bailing only when an encounter is already open. `encounters.rs`
(`focus_or_open_encounter`) writes `"[E / Pad-X] Speak with …"` into the same single `Text`, with no
ordering constraint between the two systems.

Empirically extraction wins: the Architect capture frame is standing beside the Architect embodiment,
well inside `ENCOUNTER_RANGE`, and still displays the locomotion legend. **The speak prompt is
currently dead on screen.** A workshop prompt written the same way would be equally invisible.

*Fix:* single ownership. Proximity/modal systems publish into a `HintRequest` resource with an
explicit priority (modal > device > embodiment > truth node > locomotion legend); one `render_hint`
system, ordered last, is the only writer of the `Text`.

**B4 — `E` and `Escape` are multi-bound and not state-scoped.**
- `E` is consumed by `check_extraction` (truth node), `focus_or_open_encounter` (conversation) and
  `handle_manifestation_input` (altar) in the same frame. The latter two disambiguate through
  `InteractionFocus`; extraction does not participate in focus resolution at all.
- `Escape` is worse: `extraction.rs:36-39` exits the entire mode, and it early-returns only when an
  *encounter* is open. So pressing `Escape` while the **manifestation** modal is open closes the modal
  *and* exits Inner Chambers in the same frame. A workshop modal added naively inherits exactly this:
  cancelling a plan edit would eject the player out of the castle.

This is the retired Smash Room plan's own §11.1 check — "`T`, `E`, `F`, `R`, left mouse and `Esc` are
state-scoped and cannot all react to one input in the same frame" — which was never implemented. It
is still the right check.

*Fix:* one `InnerInput` arbiter resolved before consumers; consumers read the arbitrated action, and
any open modal suppresses world-level bindings.

**B5 — Truth-node geography is desynced from the castle (scope, don't fix here).**
`extraction.rs` reads `catalog.rs`'s old heptagon (`HUB_RADIUS` 22, seven rooms at `i·τ/7`); the live
world is the Seed-of-Life castle (six rooms at radius 62). Computed reachability of the 14 nodes:

- 8 of 14 are reachable on foot only because they happen to land within 3.25 m of a bridge.
- 6 of 14 are flight-only (over the abyss between the Council platform at r=16 and the rooms at r=62).
- Architect's own nodes (`chamber_index` 0) sit ~0.8 m from the **Empath** bridge — a player can read
  "LUMINOUS BLUEPRINT / NODE 1 ALIGNED" while standing nowhere near the Architect room.

This does not block the workshop's own code, but extraction is the system clobbering the hint and
stealing `E`, so it must be scoped deliberately: gate it off in the Seed-of-Life world (one
reversible line) and re-anchor the nodes to the real rooms in a separate unit. Do not silently delete
it — the Oracle Riddle seeding path consumes `persist_extracted_truth`.

---

## Phase 0 — Make the Architect room a real place *(blocking)*

- [ ] Fix B1: shell-band room-wall collision, doorway wedge exempt, bridges unaffected.
- [ ] Fix B2: derive embodiment obstacles from the spawn formula; delete the hand-typed literals.
- [ ] Fix B3: `HintRequest` resource + single ordered `render_hint` writer, with explicit priority.
- [ ] Fix B4: `InnerInput` arbiter; modal-open suppresses world bindings; `Escape` never both closes a
      modal and exits the mode.
- [ ] Scope B5: gate `check_extraction` off in the Seed-of-Life world, with a `TODO` naming the
      follow-up unit. Keep `persist_extracted_truth` intact for Oracle Riddle.
- **Tests:** walking from the Architect doorway to `(0,-70.6)` ends at `(0,-70.6)` and not `(0,-79.1)`;
  a player at r=25 on the bridge is untouched; each embodiment obstacle equals its spawn position;
  one frame with truth node + embodiment + device in range produces exactly one action; `Escape` in a
  modal leaves `InnerChambersState::Navigating`.
- **Gate:** a *walking* (never flying) capture frame standing at the Architect table, with the table's
  prompt — not the locomotion legend — on screen.

## Phase 1 — `BuildIntent` artifact and service

New `services/build_intent.rs`, following `encounter_memory.rs`'s proven shape: append-only JSONL,
fold-on-read, no in-memory cache, scratch-path `#[cfg(test)]` helpers so tests never touch the shared
`ledger.jsonl` (the parallel-write race that corrupted the chain on 2026-09-11 is documented in the
0015 plan — do not reintroduce it).

```rust
struct BuildIntent {
    id: String, version: u32, title: String, intent_statement: String,
    constraints: Vec<String>, risks: Vec<String>,
    actions: Vec<PlanAction>,          // ordered; sequence is player-controlled
    supporting_ids: Vec<String>,       // EncounterRecord ids, Oracle source ids later
    status: PlanStatus,                // Open | Stalled | Closed
    created_at_utc_ms: u64, updated_at_utc_ms: u64,
    completion_evidence: Option<String>,
    content_hash: String,
}
struct PlanAction { id: String, description: String, done: bool, created_at_utc_ms: u64 }
```

- [ ] Events: `Created`, `ActionAdded`, `ActionCompleted`, `ActionReordered`, `ConstraintAdded`,
      `RiskAdded`, `StatusChanged`, `Closed`. Current state = fold of the log; a later event always
      supersedes an earlier one, nothing is rewritten in place.
- [ ] Path: `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\plans\journal.jsonl` (sibling of
      `data\encounters\`, via `paths::app_data_root()`).
- [ ] Ledger receipt per event through the existing `append_to_ledger(GameMode::InnerChambers, …)`.
      **Sentinel decision:** reuse the existing `memory.write` action rather than adding a new one —
      `PROTECTED_ACTIONS` is a fixed `[&str; 40]` locked by `inventory_covers_forty_canonical_actions`
      and mirrored in `docs/security/SENTINEL_PROTECTED_ACTIONS.md`. Expanding the security inventory
      for a journal write is not justified; record the reasoning in the module doc.
- [ ] `supporting_ids` may reference `EncounterRecord` ids, and must resolve through
      `encounter_memory::recallable_records` only — a forgotten encounter must not resurface as plan
      provenance. Test this explicitly.
- **Tests:** create → add action → complete → close → reopen-from-disk proves ordering and state;
  a forgotten supporting encounter is not resolvable; content-hash tamper sensitivity.

## Phase 2 — Device focus contract

- [ ] Add `InteractionTarget::ArchitectWorkshop`; mark the table entity in `world.rs`.
- [ ] `WORKSHOP_INTERACTION_RANGE = 3.0` (tighter than `ENCOUNTER_RANGE` 6.75, near the altar's 3.2).
- **The overlap is real and needs a deliberate rule.** Statue at radial 5.0, table at radial 8.6 —
  3.6 m apart. A player standing 1.5 m from the table (radial ≈ 7.1) is ~2.1 m from the statue, so
  *both* candidates are in range every time the player uses the workshop. Distance alone decides it
  today, which makes `E` position-sensitive and unpredictable at the boundary.
- [ ] Rule: a physical device inside its own (tighter) range outranks an embodiment conversation.
      Implement as priority, not distance, and test at the exact Architect coordinates.
- **Tests:** at `(0,-69.1)` focus is the workshop, not the Architect; stepping back to the room centre
  returns focus to the Architect; the prompt text changes accordingly.

## Phase 3 — Workshop interaction surface

- [ ] `WorkshopState` resource with explicit sub-states: `Closed`, `Browsing`, `Editing(field)`,
      `Confirming(action)`. No free-form composite state.
- [ ] **Locomotion gating:** add `WorkshopState` to the early-return in `camera.rs::player_locomotion`
      beside `ManifestationPhase::Prompting` and `EncounterState::is_open()` — otherwise the player
      walks and flies while typing a plan.
- [ ] **Cursor ownership:** three systems already mutate `CursorOptions` independently. Add a single
      helper that modal open/close calls, rather than a fourth ad-hoc pair of writes.
- [ ] **Text entry:** `encounters.rs` and `manifestation.rs` each carry a ~40-branch
      `KeyCode → char` table. The workshop would be the third copy. Factor one
      `services::text_entry::apply_typed_keys(&keyboard, &mut String, max_len)` and migrate all three,
      with a test for the cap and for backspace.
- [ ] **Gamepad honesty:** West opens the workshop and East cancels (consistent with the altar and
      encounters), but a gamepad **cannot type** a plan — same limitation as the manifestation
      conduit. State it in the prompt rather than implying full pad support; an on-screen keyboard is
      explicitly out of scope here.
- [ ] Every persisted mutation goes through `Confirming` — the player sees exactly what will be
      written before it is written, and a write failure leaves the draft intact on screen with an
      explanatory status (never a silent discard).

## Phase 4 — The table as a physical system

Today `Architect_FunctionalCounter` is a single untextured `Cuboid(7.2, 1.5, 2.4)` — visible in the
capture frame as a plain brown box behind the figure. Promoting it to a marker component is enough
for the mechanic, **not** enough for Step 8's "readable entry affordance".

- [ ] Give the table legs/frame, an angled drafting surface, and a distinct material.
- [ ] Add a notice board or shelf behind it carrying open-plan state, **only after** the workflow is
      proven useful (Step 8's own ordering).
- [ ] Approach lighting so the table reads as a destination from the doorway — the room currently
      renders near-black around it.
- **Gate:** operator verdict on a walking capture. Per repo law, no self-approval of visuals.

## Phase 5 — Architect proposals *(deferred to v2)*

Wire `services::archetype_conversation` so the Architect may **propose** one next action or flag one
risk, which the player accepts (written) or dismisses (nothing persisted). The local model may never
commit. "Under-practiced area" surfacing uses only measurable on-disk signals — open plan count,
stalled-action age, completed-vs-open ratio — never invented psychometrics. Not started until v1's
workflow has proven useful in the room.

---

## Non-goals

- No gamepad on-screen keyboard.
- No cloud sync, no remote task creation, no calendar/issue-tracker integration.
- No re-anchoring of the truth-node system (separate unit; B5 is only gated here).
- No changes to `encounter_memory` semantics; plans reference encounters, they do not duplicate them.

## Risks and open questions

1. **Phase 0 is roughly half this plan's work and touches movement code that currently "looks fine".**
   Every fix needs a regression capture, because flight-mode captures cannot detect a walking bug.
2. **The B1 fix changes room feel for all six rooms, not just Architect.** It should ship and be
   operator-reviewed on its own before the workshop lands on top of it.
3. **Ordering risk:** `HintRequest` and `InnerInput` add two new arbitration points. If they are
   introduced without explicit system ordering they reproduce exactly the B3 defect they exist to fix.
4. Open question for the operator: should `Escape` at the workshop return to the room (my assumption),
   or exit the castle as it does elsewhere today?

## File-by-file change list

| File | Change |
|---|---|
| `modes/inner_chambers/camera.rs` | shell-band wall collision; derived obstacle list; workshop added to locomotion gating |
| `modes/inner_chambers/extraction.rs` | hint write moved behind `HintRequest`; gated off in Seed-of-Life world |
| `modes/inner_chambers/interaction.rs` | `ArchitectWorkshop` target, device-outranks-embodiment priority |
| `modes/inner_chambers/encounters.rs` | hint via `HintRequest`; text entry via shared helper |
| `modes/inner_chambers/manifestation.rs` | text entry via shared helper; cursor helper |
| `modes/inner_chambers/workshop.rs` | **new** — `WorkshopState`, UI, confirm flow |
| `modes/inner_chambers/world.rs` | table marker component, table geometry, approach lighting |
| `services/build_intent.rs` | **new** — artifact, event log, fold, ledger receipts |
| `services/text_entry.rs` | **new** — shared typed-key helper |
| `modes/inner_chambers/mod.rs`, `services/mod.rs` | registration |

## Delivery order

Phase 0 → operator review of walking movement → Phase 1 → Phase 2 → Phase 3 → Phase 4 → operator
verdict → (Phase 5 only on request). Each phase is its own commit/push after `cargo test --workspace`,
`scripts\install_shortcut.ps1` restage, and a capability-appropriate real witness.
