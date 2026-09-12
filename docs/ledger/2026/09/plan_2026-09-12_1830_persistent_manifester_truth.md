# Plan: Persistent Manifester truth and player library — 2026-09-12 18:30

## Status

IN-PROGRESS

## Goal

Turn the Manifester's currently partial persistence path into an honest player-owned object system while preserving the boundary that generated geometry quality belongs to Chronos2. Every successfully manifested artifact must retain a durable identity and provenance, placements must survive restart, and the player must be able to browse and summon a prior creation. The bridge must consume a verified Chronos2 game-artifact contract rather than infer success from a private bundle layout.

## Evidence at start

- `cargo test --workspace` passed on 2026-09-12: 248 engine, 19 launcher, and 5 Windows-identity tests.
- `objects.rs` implements take, carry, place, `R` duplicate, and an append-only placement ledger. It does not expose a library/summon interaction.
- `manifestation.rs` invokes Chronos2 and requires an OBJ plus the presence of `triposr_artifact.json`, but does not validate schema, hashes, source-prompt binding, Sentinel disposition, or a game-ready output digest.
- The quality-lab's multi-angle renders show that current single-view TripoSR output does not meet an all-around object-quality bar. This plan does not represent a presentation fix as a geometry fix.

## Steps

### Step 1 — Specify and validate the game artifact receipt

- [ ] Action: Consume only a versioned Chronos2 game-artifact receipt that binds the prompt digest, Sentinel outcome, source mesh digest, game GLB digest, and no-recipe declaration; fail visibly on every mismatch.
- Files touched: manifestation bridge, focused tests, truth documentation.
- Expected outcome: Archetypes never stages a mesh merely because a process exited successfully or an internal path happens to exist.

### Step 2 — Make the player library reachable

- [ ] Action: Add an in-game catalog interaction that reads the durable artifact library, presents player-created objects with their original prompt/provenance, and summons a chosen artifact into the player's hand without duplicating or overwriting the artifact file.
- Files touched: Inner Chambers interaction/UI/object systems, artifact service tests, player documentation.
- Expected outcome: Any retained player creation can be called back at any time; carrying, placement, and duplication remain one coherent lifecycle.

### Step 3 — Prove persistence and lifecycle end to end

- [ ] Action: Exercise manifest-to-library, summon, place, duplicate, withdraw, reload, and re-summon against a real artifact fixture; refresh the desktop launcher and capture a visible buyer-path witness after runtime changes.
- Files touched: tests and designated visual-proof artifacts only.
- Expected outcome: The system is proven as a player-facing loop, not a collection of independent unit tests.

### Step 4 — Publish only proven state

- [ ] Action: Update status/truth documents, run the full Rust gate, run `pwsh -File scripts\install_shortcut.ps1` for player-facing changes, inspect the installed launcher witness, stage explicit paths, commit, push `main`, and verify clean remote parity.
- Files touched: plan and required truth surfaces.
- Expected outcome: The main branch contains a truthful, auditable handoff.

## Dependency boundary

Chronos2 owns the high-quality full-volume generation engine and its game-artifact contract. This plan cannot make TripoSR's single-view inferred rear surface accurate; it refuses to hide that limitation.
