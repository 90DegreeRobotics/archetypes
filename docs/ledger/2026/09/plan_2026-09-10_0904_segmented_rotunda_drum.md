# Plan: segmented-rotunda-drum — 2026-09-10 09:04

## Status
COMPLETED — 2026-09-10. The circular-drum implementation, Rust gate, installed build,
and rendered-frame proof completed. The separate multiview evaluation remains outside
this unit and open in the parent plan.

## Goal
Close the remaining visual architecture gap in the Inner Chambers: replace the four visible square-shell slabs with a procedural segmented circular drum, while retaining the proven movement envelope and all already-captured archetype niches. This is a narrow continuation of Step 2, not a multiview-reconstruction or manifestation change.

## Steps

### Step 1 — Measure and preserve the existing envelope
- [x] Action: Confirm `693d0c0` is on `origin/main`, inspect the established wall, ceiling, lights, capture harness, and the +/-36m movement clamp.
- Files touched: None.
- Expected outcome: The new shell fits inside the established collision/movement surface and does not alter the unrelated dirty manifestation GLB.

### Step 2 — Author the procedural drum shell
- [x] Action: Added a 64-bay parameterized annular wall shell, 16 radial buttresses,
  continuous Torus cornice, four cardinal processional portal frames, circular ceiling cap,
  and 12 radial ribs. The previous four visible square-wall slabs, square cornice spans,
  and warehouse-grid rafters are no longer spawned.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs` and focused tests.
- Expected outcome: Met mechanically and in the new low interior capture angle. This remains
  a procedurally authored architectural shell, not a premade scene mesh.

### Step 3 — Prove the actual installed experience
- [x] Action: `cargo test --workspace` passed 117/117 (93 engine, 19 launcher, 5
  Windows identity). `pwsh -File scripts\\install_shortcut.ps1` rebuilt/staged the release
  workspace, verified installed engine SHA-256 `47756EC56CBCB57FD7ECA1F9DA11E72308D272F1779A1E5C1E501227CD68C765`,
  verified launcher SHA-256 `C5FD3B5688073207A22DEBE511987116283F1D5202203179CDAA3EF06BF44163`,
  and refreshed the Taskbar target. The installed engine produced eight fresh capture frames
  under `artifacts/visual-proof/inner-chambers-rotunda-drum-2026-09-10/`, including
  `07_rotunda_drum_and_portal.png`.
- Files touched: capture proof, this plan, `STATUS.md` if and only if the proof passes.
- Expected outcome: Met for the staged installed binary. The shell-launched capture command
  returned without a visible process/log witness (normal Windows-GUI process behavior in this
  noninteractive session); do not represent that specific shell invocation as a launcher E2E
  proof. The installer staging and Taskbar target were independently verified.
