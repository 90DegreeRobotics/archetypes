# Plan: Hard Reset — Coherence Before Surface — 2026-09-12 20:10

## Status
IN-PROGRESS

## Goal

Freeze new features and museum dressing. Fix the game's physical coherence (sprint,
floor, shadows, frame overlay, crosshair), remove bad old art, add quality gates for
manifested objects, and begin the multi-view generation research as a separate
workstream. This is a priority reorder, not a repository reset.

Operator directive: *"Freeze new features. Make the game physically coherent. Remove
bad old art. Treat every manifested object as failed unless it passes review. Build the
full-volume generation path separately."*

## Steps

### Step 1 — Phase 1: Remove bad old art
- [ ] Action: Replace stale default manifestation assets, audit and quarantine bad
  manifested/museum assets, resolve dirty working tree.
- Files touched: `assets/scenes/`, `assets/manifested/`, `assets/museum/`, manifestation
  spawn code.
- Expected outcome: Altar is clean by default; no blob ships.

### Step 2 — Phase 2: Physical coherence
- [ ] Action: Sprint (Shift), council floor stone, contact shadows, FPS overlay, crosshair.
- Files touched: `camera.rs`, `world.rs`, `interaction.rs`, new `hud.rs` or similar.
- Expected outcome: Castle traversal feels like a real space.

### Step 3 — Phase 3: UI polish
- [ ] Action: Context-sensitive HUD, settings menu visual pass, loading screen.
- Files touched: `world.rs`, `settings_menu.rs`, `mod.rs`.
- Expected outcome: HUD reads as a shipped product.

### Step 4 — Phase 4: Quality gates
- [ ] Action: Automated geometry checks, multi-angle render gate, threshold calibration,
  single emitter copy.
- Files touched: manifestation bridge, quality scripts, build.rs.
- Expected outcome: Bad geometry is rejected with a measurement.

### Step 5 — Phase 5: Multi-view research (separate workstream)
- [ ] Action: Survey, prototype, document candidates.
- Files touched: docs only in this step.
- Expected outcome: A measured comparison of single-view vs multi-view on the test spread.

## Execution record

### 2026-09-12 — recovery checkpoint

- Repaired the interrupted sprint/HUD patch until `cargo check -p engine` and the complete
  workspace gate both passed. The checkpoint adds hold-Shift sprinting, a hidden F3 frame-rate
  witness, and a crosshair that disappears while a modal owns input.
- Replaced the shared `scenes/manifested_reference.png` runtime target with a per-artifact image
  path. Bevy caches by asset path; reusing that path was the direct cause of every later
  manifestation showing the first reference image.
- A library-write failure now reaches the player as a failure rather than reporting an object as
  successfully manifested when it will disappear from the reusable collection on restart.
- Library reads now fold repeated artifact IDs to the newest row. Historical seeding/retry rows
  therefore no longer render as several copies of one creation.
- Gate: `cargo test --workspace` passed: 282 engine, 19 launcher, 5 Windows-identity tests.
- Remaining work in this plan is deliberately still open: visual-quality quarantine, traversal
  witness, UI pass, geometry gates, and full-volume research are not represented as done here.

### 2026-09-12 — exhibition and texture containment

- Quarantined the unreviewed staged museum collection from the live arcade without deleting its
  files or manifest. The rooms remain architecture until a curated replacement set is approved.
- Enabled 8x anisotropic sampling on tiled masonry material inputs to reduce oblique-angle
  crawling. This is a targeted stability improvement, not a claim that it solves every source
  of shimmer; the installed visual witness still has to judge that.

### 2026-09-12 — installed visual witness findings

- Ran `ARCHETYPES_WALK_CAPTURE` through the installed executable. It proved the climb and museum
  doorway are physically traversed by the real walking system, but also exposed old placements
  still polluting the Council floor and unsupported HUD glyphs rendering as boxes.
- Replaced the HUD glyphs with ASCII and quarantined historical placed meshes from automatic
  world restore. Their placement/library ledgers are retained unchanged; they are withheld from
  the live scene until the required multi-angle quality review exists.
