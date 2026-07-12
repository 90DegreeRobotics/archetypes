# Plan: Archetype Theme Registry — 2026-07-11 19:44

## Status
INTERRUPTED

The registry scaffold and chamber color mapping were implemented, but the original claim that all nine themes were meticulously and exactly transcribed was not verified. Viren was reconciled against the canonical manuscript on 2026-07-11; the remaining eight themes require their own source-by-source audit before this plan can honestly be called complete.

## Goal
Implement Phase 2 of the Council Chamber Game Direction. Extract all UI themes, CSS variable equivalents, motion laws, and harmonic signatures from the three AURA text documents and encode them strictly as Rust constants in a new `theme` module in the Bevy engine. Integrate this theme registry into the chamber simulation.

## Steps
### Step 1 — Scaffold Theme Module
- [x] Action: Create the `theme` module in `crates/engine/src` with `mod.rs` defining the `ArchetypeTheme` struct.
- Files touched: `crates/engine/src/theme/mod.rs`, `crates/engine/src/main.rs`
- Expected outcome: The `ArchetypeTheme` data structure is ready to hold colors, timings, and motion curves.

### Step 2 — Encode Meticulous Constants
- [ ] Action: Create `theme/constants.rs` and transcribe the 9 specific archetype themes (Architect, Sentinel, Jester, Mentor, Explorer, Oracle, Empath, Codex, Viren).
- Files touched: `crates/engine/src/theme/constants.rs`, `crates/engine/src/theme/mod.rs`
- Expected outcome: All CSS schemas from the design documents are rigorously encoded as Bevy-compatible Rust constants.

### Step 3 — Apply Themes to Spheres
- [x] Action: Update the `spheres.rs` module to map the 7 spheres to their specific archetype themes (colors and glowing behavior).
- Files touched: `crates/engine/src/chamber/spheres.rs`
- Expected outcome: The 7 orbiting spheres visually differentiate themselves based on their exact canonical colors (e.g., Luma Rose Quartz, Sentinel Null Aegis).

### Step 4 — Dynamic Focus State
- [x] Action: Introduce `CurrentFocus` resource to track the dominant archetype and drive potential transitions.
- Files touched: `crates/engine/src/chamber/mod.rs`
- Expected outcome: The chamber state has the plumbing necessary to switch global themes when an archetype takes focus.
