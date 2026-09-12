# Plan: Castle Realism, Museum Arches, and the Settings Menu — 2026-09-12 07:25

## Status
IN PROGRESS — Phases A, B, C, D, E and F shipped 2026-09-12. Remaining: operator verdict on the
stone progression, and the live in-game manifestation run (the CLI pipeline completes in ~5
minutes standalone; driven from inside the running game it produced no stage output in 25).

## Operator directive (verbatim intent, 2026-09-12 morning)

1. **The table goes away.** Only the animated spinning disk gets put on the floor. The
   manifester sits on the floor in the middle of the spinning disk. There is no table.
2. **Plan docs are not being read.** Requests are not being answered correctly. Read them.
3. **The settings menu is still non-existent.**
4. **No stone surfaces. The building looks fake. Make it look real.**
5. **Spend the day building in Blender and converting to game assets.**
6. **Open the archways up into doorways as planned.**
7. **Rooms hold art in frames with placards, from `C:\chronos2` museum mode.**
8. **Fix the floor on the arcade rings.**

The operator is checking from a phone through the day, so each phase ships evidence frames as
it lands rather than one report at the end.

---

## What the plan docs actually say, and where the previous units stopped

Read in full before writing this:

| Doc | Bearing on today |
|---|---|
| `plan_2026-09-11_2200_handoff_rotunda_heart_and_stone.md` | Phase 1 (stone, world-scaled UVs), Phase 3 (floors) — **both still PENDING**. Its Phase 2 shipped but landed the *table*, which is item 1 above. |
| `plan_2026-09-11_2130_museum_arches.md` | The whole museum design: 12 walkable bays of 504, the doorway exemption, the chamber module, the Chronos2 staging rule. **PENDING**. |
| `plan_2026-09-12_0008_restore_rotunda_heart.md` | COMPLETED — it is what put `table.glb` at the centre. Today supersedes its placement decision, not its wiring. |
| `plan_2026-09-11_2245_hide_rooms_and_figures_except_jester.md` | COMPLETED — the six satellite rooms and all figures but the Jester are gated off. The museum chambers are where that content returns. |
| `GAMEPAD_AND_SETTINGS_GUIDE.md` + `plan_2026-09-10_2330_...` | The settings menu is fully specified and **was never built**. `services/settings.rs` persists values; nothing in the engine displays them. Confirmed: the only reference to `GameSettings` outside its own file is `camera.rs:226`. |

**Correction to the record this plan carries forward.** The 2026-09-12 00:08 unit read the
handoff's "the old table's spinning portal effect brought back" as *restore the table*. The
operator's actual want is the **disc effect only**. The table was never the point; the vortex
was. That is item 1, and it is first.

---

## Phase A — The centre: portal disc on the floor, altar in the middle of it

**Current state.** `world.rs:275` spawns `table.glb` at 2.6x scale with root y derived from the
authored foot at local z=-0.766, putting the tabletop at ~2.4m and the Stargate disc at
y=3.1716. `MANIFESTATION_PEDESTAL_POS` sits on that disc plane at y=3.1716, so the altar stands
on a table. `camera.rs:185` collides the whole 2.65m table silhouette.

**Target.** No table in the world at all. The spinning disc lies flush on the Council floor at
`GROUND_Y`. The altar stands on the floor at the disc's centre, so the vortex turns in a ring
around its base.

- [ ] New Blender module `scripts/author_portal_disc.py` exporting `assets/scenes/portal_disc.glb`:
      the `Stargate_Portal` node alone, carrying the same `portal_vortex_v2.png` emissive
      material, authored at a real world radius rather than the table's 0.72m local one.
      **Authoring a standalone module, not hiding table children,** so nothing in the shipped
      scene is a mesh being kept invisible.
- [ ] The node must still be named `Stargate_Portal` — `chamber/portal.rs` binds by that name
      and already spins it and pulses its emissive. That system is not modified.
- [ ] `world.rs`: drop the `table.glb` spawn and its derived constants; spawn the disc at
      `GROUND_Y` + a hair. Keep the blue portal light, re-seated.
- [ ] `manifestation.rs`: `MANIFESTATION_PEDESTAL_POS` → `(0.0, GROUND_Y, 0.0)`.
- [ ] `camera.rs`: collision radius drops from the 2.65m table silhouette to the altar's own
      footprint, derived from the shared constant, so the player can walk right up to the altar
      and stand on the disc.
- [ ] `interaction.rs`: re-check `ALTAR_INTERACTION_RANGE` = 3.2 now the altar is 2.77m lower —
      the vertical budget that nearly exhausted the range is gone, so it should widen, not
      narrow.
- [ ] **Tests:** the altar sits on the floor plane; the disc radius contains the altar footprint;
      collision no longer reserves table-sized ground.
- **Gate:** a walking capture standing on the disc with the altar reachable.

## Phase B — Floors: the Council platform and the arcade rings

**Two defects, both already diagnosed in the handoff and neither fixed.**

1. `spawn_castle_platform` calls `build_radial_flagstone_mesh(..., 16, 0.055)`. The gap is an
   **angle**, so at the Council's 24m radius it is a **1.32m hole** between stones. That is the
   radial spoke pattern in the operator's screenshot. Fix with the existing `joint_angle(m, r)`
   helper the gallery decks already use.
2. The same mesh raises its stones above its own origin; the gallery decks were fixed for this
   on 2026-09-11 but `spawn_castle_platform` was never checked. Verify the Council stone tops
   land on `GROUND_Y` and drop the mesh if they do not.

**Third defect, found today and not in any prior doc.** The gallery deck ring is
`build_radial_flagstone_mesh(GALLERY_INNER_RADIUS, GALLERY_OUTER_RADIUS, ...)` with **no radial
subdivision**. Each "flagstone" is one wedge 12m deep and ~5m wide. Real paving is not 12m
across. The ring needs courses in the radial direction as well as the circumferential one, so
the arcade decks read as paving rather than as a segmented disc.

- [ ] Give `build_radial_flagstone_mesh` radial courses with joints specified in metres.
- [ ] Apply to the Council platform, the promenade and all seven gallery decks.
- [ ] **Tests:** no joint anywhere exceeds its specified width in metres at its own radius;
      stone top faces equal the collision surface height for every deck.

## Phase C — Stone that reads as stone

The building is flat colour on UV'd geometry. This is the "looks fake" complaint and the single
biggest visual return.

- [ ] **World-scaled UVs first.** The kit uses `smart_project`, which packs into unit space per
      object and has no physical scale, so the same stone would be one size on a pier and
      another on a tread. Replace with a cube projection sized in metres in
      `author_arcade_bay.py::bevel_and_unwrap()` and `author_stair_flight.py::unwrap()`. Every
      texture decision after this depends on it.
- [ ] **Author the tiles rather than sourcing them.** The operator's nine-stone reference sheet
      is one 1254x1254 image, so each tile is ~378px — about 38 px/m on a 9.9m bay, which is
      mush. Per `memory: build-it-dont-wait-for-a-clean-license`, author originals: generate
      seamless 1K albedo + normal + roughness procedurally, matching the reference *looks*.
- [ ] Normal maps are non-negotiable: a flat photo of stone on a flat pier still reads flat
      because nothing responds to the light. `world.rs::chamber_floor_textures()` is the
      existing precedent for albedo+normal generation in this repo.
- [ ] **One bay GLB, material swapped per level at spawn time** — not seven exported bays.
- [ ] Stone varies by arcade level, heaviest at the base. The progression in the handoff is a
      proposal; the operator judges it.

## Phase D — Archways opened into doorways

Per `plan_2026-09-11_2130_museum_arches.md`, 12 of gallery 1's 72 bays become walkable, every
6th bay, 60 degrees apart, leaving five bays of solid masonry between openings.

- [ ] `castle.rs`: `MUSEUM_BAYS`, `museum_bay_bearing(index)`, a chamber-shaped exemption in
      `clamp_inside_wall`, and `museum_chamber_surface_y` continuous with the gallery deck.
- [ ] `clamp_inside_wall` must stay conditional — the wall is still a shell everywhere else.
- [ ] `scripts/author_museum_chamber.py`: interior 7.745m wide (matching the arch opening),
      9.0m deep into the 12m wall, barrel vault springing at the same 5.5m line as the arch
      outside, with flat UV'd hanging panels.
- [ ] **Tests:** a player at a museum bearing passes the wall face and stops at the back wall; a
      player one bay over is still stopped at the face; no step at the threshold.
- **Gate:** a walking capture entering an arch and standing inside it.

## Phase E — Art in frames with placards

- [ ] Stage selected works **into the repo at author time**. The game must not read
      `C:\chronos2` at runtime — the launcher already treats Chronos as optional and a buyer has
      no such directory. Script writes `assets/museum/` + `manifest.json`.
- [ ] Carry across Chronos2's `title_from_prompt` and, critically, its rule **excluding runs
      whose own vista was `museum_wall`**, so a photograph of a gallery wall is never hung on a
      gallery wall.
- [ ] Placards state only what the bundle records — title from `human_prompt.txt`, date from the
      bundle directory name, hash from `integrity_report.json`. A missing field omits its line.
      **No invented artist, year, or wall text.**
- [ ] Frame + canvas + placard as a Blender module, instanced per hanging position.
- [ ] Chamber lighting: the hall's tiered lamps do not reach into a 9m recess, and the artwork
      must be lit rather than emissive — an emissive painting is a lightbox, not a painting.

## Phase F — The settings menu

Specified in full in `GAMEPAD_AND_SETTINGS_GUIDE.md`; nothing was built. `GameSettings` already
persists atomically to `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json` and already
clamps on load and save.

- [ ] An in-game settings screen: Esc/Start opens it, sliders for mouse sensitivity, gamepad
      look sensitivity, deadzone, and the four volumes, keyboard + gamepad navigable.
- [ ] It must route through the existing `InnerModalState` arbiter, or Esc will close the menu
      *and* exit the mode in the same frame — that bug was fixed once already.
- [ ] Wire the volume values to something audible, or report plainly that the sliders move
      numbers that no audio bus reads yet. **Do not present a stub as the real surface.**

---

## Sequencing, and why

A → B → C → D → E → F.

The centre is the operator's first sentence and is self-contained. Floors come next because
they are mechanical and they change what the stone textures will be judged against. Stone third:
it is the largest visual return and it needs the UV work done before any tile is authored.
Doorways before art, because there is nowhere to hang art until an arch is passable. Settings
last of the six only because nothing else depends on it — not because it is optional; it is an
explicit operator ask that two prior units left undone.

## Verification every phase must pass

- `cargo test --workspace` — currently **201 passing**. The number must not fall.
- `pwsh -File scripts/install_shortcut.ps1` to restage the installed build.
- A **walking** capture (`ARCHETYPES_WALK_CAPTURE=1`) for anything touching movement, collision
  or interaction, with its `walk_report.txt` committed. The flying harness teleports and is for
  vantage shots only — `memory: verify-by-walking-not-flying`.
- Frames under `artifacts/visual-proof/<topic>-2026-09-12/`, with an explicit statement of what
  they do and do not prove.
- **Geometry proof is not aesthetic approval.** The operator is the sole judge of the look —
  `memory: never-self-approve-visuals`. Report mechanical facts, show the frame, do not call it
  good.

## Known risk carried into today

The six satellite archetype rooms are hidden, and with them every entity carrying
`ArchetypeEmbodiment` — which is the only thing `interaction.rs` and `encounters.rs` target. The
six archetype conversations are therefore currently unreachable. The handoff raised three
options and no operator decision has been recorded. **This plan does not decide it.** The museum
chambers may turn out to be where those conversations move, which is a reason to raise it again
once an arch is walkable, not before.
