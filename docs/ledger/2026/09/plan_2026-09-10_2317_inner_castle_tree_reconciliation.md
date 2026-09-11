# Plan: Inner Castle Tree Reconciliation — 2026-09-10 23:17

## Status
IN-PROGRESS

## Goal

Reconcile the complete operator-authorized working tree into one honest Inner Castle
checkpoint on `main`. This includes the concurrent manifestation, Seed-of-Life castle,
embodiment, capture, conversation, and newly supplied music work. The checkpoint must use
the supplied music from the actual runtime, resolve the known competing-`E` interaction seam,
and only be committed after the full Rust and installed-launcher gates pass.

## Steps

### Step 1 — Inventory and classify every pending path
- [x] Action: Record tracked/untracked paths and current branch head before edits.
- Files touched: This plan.
- Expected outcome: No concurrent or generated artifact is silently omitted.

### Step 2 — Reconcile gameplay contracts
- [x] Action: Audit the castle, manifestation, extraction, and encounter input paths; make one
  target decision for `E` at archetype/altar/node overlap.
- Files touched: `crates/engine/src/modes/inner_chambers/**` as needed.
- Expected outcome: The in-world chat and existing manifestation system coexist deterministically.

### Step 3 — Wire supplied music honestly
- [x] Action: Add the two operator-supplied WAV derivatives to a lifecycle-owned Castle music
  player with a clear fallback/availability state.
- Files touched: `assets/audio/music/**`, Inner Chambers runtime modules, audio plan metadata.
- Expected outcome: Music actually plays from installed assets while Inner Chambers is active;
  44.1 kHz MP3-derived provenance remains explicit.

### Step 4 — Verify and deliver the complete tree
- [x] Action: Run the full Rust suite and refresh the Desktop/Taskbar
  product, perform a visible Inner Chambers witness, then commit and push `origin/main`.
- Files touched: Plan/status proof paths as earned.
- Expected outcome: A clean, pushed `main` checkout with evidence rather than a source-only claim.

## Verification record

- `cargo test --workspace`: 97 engine, 19 launcher, and 5 Windows identity tests passed; 0 failed.
- `pwsh -File scripts\install_shortcut.ps1`: passed. Installed `engine.exe` SHA-256
  `6A06A9FC7CC97B0ED3359BC45EB12B8F6BCC59B3753EA86B1E3288174216DAB4`;
  installed launcher and Taskbar target were verified.
- Music is intentionally `PlaybackSettings::ONCE` at 0.22 linear volume. The MP3-derived tracks
  have no proven loop boundary, so a hard infinite loop would be a knowingly bad implementation.
