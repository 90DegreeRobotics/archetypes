# Plan: Council Chamber Loop — 2026-07-11 19:38

## Status
COMPLETED

## Goal
Implement Phase 1 of the Council Chamber Game Direction. Build the foundation of the Council Chamber loop in the Bevy engine, including the Witness seat camera perspective, the idle merkaba and sphere orchestration, and a placeholder portal input mechanic that triggers the upward camera move.

## Steps
### Step 1 — Scaffold Chamber Module
- [x] Action: Create the `chamber` module in `crates/engine/src` with the plugin `CouncilChamberPlugin`.
- Files touched: `crates/engine/src/chamber/mod.rs`, `crates/engine/src/main.rs`
- Expected outcome: The plugin is integrated and defines basic state machines (`ChamberState`).

### Step 2 — Camera and Witness Seat
- [x] Action: Implement the camera setup and upward interpolation logic.
- Files touched: `crates/engine/src/chamber/camera.rs`
- Expected outcome: Camera starts at the table (Witness position) and can move towards the merkaba.

### Step 3 — Merkaba Structure
- [x] Action: Spawns the central astrolabe/merkaba rings with procedural geometry and continuous rotation.
- Files touched: `crates/engine/src/chamber/merkaba.rs`
- Expected outcome: A visually interesting, rotating core element above the table.

### Step 4 — Archetype Spheres
- [x] Action: Spawns the 7 spheres in idle orbits around the merkaba, with logic for individual rotation and the "focus" choreography.
- Files touched: `crates/engine/src/chamber/spheres.rs`
- Expected outcome: 7 orbiting spheres that can break orbit and glide to the center.

### Step 5 — Portal and State Transitions
- [x] Action: Implement placeholder text entry or a basic keyboard trigger representing the prompt ritual, and tie it to the `ChamberState` transitions.
- Files touched: `crates/engine/src/chamber/portal.rs`
- Expected outcome: A complete loop from idle table -> input -> upward camera -> merkaba deliberation -> focus choreography.
