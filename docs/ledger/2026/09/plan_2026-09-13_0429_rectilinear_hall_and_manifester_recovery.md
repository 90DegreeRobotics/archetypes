# Plan: Rectilinear hall and Manifester recovery — 2026-09-13 04:29

## Status
COMPLETED

## Goal
Replace the rejected circular Council Chamber with a clean, fully rectilinear workspace and repair the proven ComfyUI startup race that caused the live `sword` manifestation to waste seven minutes before failing. The installed Taskbar build must expose the new room and a Manifester that recovers from a lost ComfyUI job instead of silently waiting through the full timeout.

## Steps

### Step 1 — Preserve the failed-run evidence
- [x] Action: Inspect the exact live `sword` process, output directory, engine log, ComfyUI health, queue, and history.
- Files touched: This plan only.
- Expected outcome: A source-level repair tied to the real failure rather than a guess.

### Step 2 — Replace the circular live building
- [x] Action: Build a rectangular floor, walls, ceiling, structural bays, and lighting with no circular room shell, radial arcade, or domed fresco on the live path.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`, `crates/engine/src/modes/inner_chambers/castle.rs`.
- Expected outcome: A stable, legible, non-round Manifester hall with no coplanar decorative overlays.

### Step 3 — Recover lost ComfyUI jobs
- [x] Action: Harden ComfyUI readiness/submission/history behavior so an owned live job survives the bounded history poll while a missing job fails explicitly; remove the startup `/free` race and stale 420 second machine override.
- Files touched: Chronos source under `C:\chronos2`, Archetypes manifestation UI/source as required, and focused tests.
- Expected outcome: The witnessed startup race cannot strand the player at 52 percent for the full legacy timeout.

### Step 4 — Verify runtime behavior
- [x] Action: Run focused tests, then `cargo test --workspace`; run a real installed manifestation witness and visually inspect the installed rectangular hall.
- Files touched: Test evidence and plan status only.
- Expected outcome: Both the room and generation recovery are proven through the buyer-facing path.

### Step 5 — Install and publish
- [x] Action: Run `pwsh -File scripts\install_shortcut.ps1`, confirm installed hashes/timestamps, complete the plan, commit the coherent unit on `main`, and push `origin main`.
- Files touched: This plan and generated installation surfaces.
- Expected outcome: The pinned Taskbar launcher runs exactly the verified build and the repository is clean and remotely auditable.

## Result

- The live Manifester hall is a 72 m by 52 m rectilinear room with one floor slab, four straight walls, a flat ceiling, straight structure, and stable rough materials. The circular shell, radial arcade, tiled overlays, and vault fresco are not on the live path.
- A final installed scene inventory proved `LoreCouncilChamber`, `LC_GroundPlane_80m`, and `LC_HemisphericalDome` absent while `ManifesterHall_SingleSurfaceFloor` and `ManifesterHall_FlatCeiling` were present. The retired round scene remains preserved in the repository behind the explicit `ARCHETYPES_LORE_CHAMBER` archive opt-in.
- The game no longer sends the pre-generation ComfyUI `/free` request, forces the accepted 768 px Flux lane, bounds the reference poll override at 180 seconds, and uses the recovered attempt-2 seed. Chronos now preserves an owned in-progress Comfy job rather than confusing a slow render with missing history.
- The live installed altar route submitted `sword`, completed Flux -> TripoSR MC 256 -> Blender -> topology-preserving GLB import, and reported `Manifested 'sword' atop the altar!`. The reference panel showed the new sword image rather than the stale first render. The object remained recognizable in the inspected front, oblique, and high render views.
- Final full Rust gate after all source changes: 288 engine + 19 launcher + 5 Windows identity = 312 passed, 0 failed.
- `pwsh -File scripts\install_shortcut.ps1` completed. Final installed engine SHA256: `BD40536400894A3C795FE208C52990764EBBA83046039F1BAD4CFF0A1DCABBDA`; launcher SHA256: `D45E3CF72C96144DB47E5E66E86147EA5B9C1A6A38B496CDAA3EC0F651A49A65`. The Taskbar shortcut resolves to `%LOCALAPPDATA%\Programs\Archetypes\launcher.exe`.
- Quality boundary: this restores a usable, coherent sword baseline and honest failure for bad references. The measured subject-match score was `0.2911`; excellent arbitrary-object quality is not yet achieved and remains the continuing product goal.
