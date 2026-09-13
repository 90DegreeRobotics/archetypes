# Plan: Rectilinear hall and Manifester recovery — 2026-09-13 04:29

## Status
IN-PROGRESS

## Goal
Replace the rejected circular Council Chamber with a clean, fully rectilinear workspace and repair the proven ComfyUI startup race that caused the live `sword` manifestation to waste seven minutes before failing. The installed Taskbar build must expose the new room and a Manifester that recovers from a lost ComfyUI job instead of silently waiting through the full timeout.

## Steps

### Step 1 — Preserve the failed-run evidence
- [x] Action: Inspect the exact live `sword` process, output directory, engine log, ComfyUI health, queue, and history.
- Files touched: This plan only.
- Expected outcome: A source-level repair tied to the real failure rather than a guess.

### Step 2 — Replace the circular live building
- [ ] Action: Build a rectangular floor, walls, ceiling, structural bays, and lighting with no circular room shell, radial arcade, or domed fresco on the live path.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`, `crates/engine/src/modes/inner_chambers/castle.rs`.
- Expected outcome: A stable, legible, non-round Manifester hall with no coplanar decorative overlays.

### Step 3 — Recover lost ComfyUI jobs
- [ ] Action: Harden ComfyUI readiness/submission/history behavior so a server restart or lost prompt is detected quickly and retried once; surface an actionable terminal failure instead of a seven-minute apparent hang.
- Files touched: Chronos source under `C:\chronos2`, Archetypes manifestation UI/source as required, and focused tests.
- Expected outcome: The witnessed startup race cannot strand the player at 52 percent for the full legacy timeout.

### Step 4 — Verify runtime behavior
- [ ] Action: Run focused tests, then `cargo test --workspace`; run a real installed manifestation witness and visually inspect the installed rectangular hall.
- Files touched: Test evidence and plan status only.
- Expected outcome: Both the room and generation recovery are proven through the buyer-facing path.

### Step 5 — Install and publish
- [ ] Action: Run `pwsh -File scripts\install_shortcut.ps1`, confirm installed hashes/timestamps, complete the plan, commit the coherent unit on `main`, and push `origin main`.
- Files touched: This plan and generated installation surfaces.
- Expected outcome: The pinned Taskbar launcher runs exactly the verified build and the repository is clean and remotely auditable.
