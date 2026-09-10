# Plan: Repair Manifestation Model Bottleneck & Fast Relic Generation — 2026-09-09 19:45

## Status

COMPLETED

## Goal

Resolve the manifestation hang where prompt-to-GLB generation was stuck on a 29 GB model (Flux Schnell + T5 XXL) exceeding the 12 GB VRAM capacity of the RTX 3060 while the 3D game engine is running. Switch reference generation to the compact, high-quality 2.1 GB Dreamshaper 8 checkpoint (or SDXL), configure ComfyUI VRAM reservation, and add live second-by-second elapsed timer feedback to the Archetypes HUD.

## Steps

### Step 1 — Model Linking and Workflow Resolution
- [x] Action:
  - Hardlink `dreamshaper_8.safetensors` (2.13 GB), `sd_xl_base_1.0.safetensors` (6.93 GB), and `Juggernaut-XL_v9_RunDiffusionPhoto_v2.safetensors` (7.10 GB) into `C:\Users\m\AppData\Local\CS\ComfyUI\models\checkpoints`.
  - Create `C:\Users\m\AppData\Local\CS\ComfyUI\extra_model_paths.yaml` pointing to `C:\Users\m\Documents\ComfyUI`.
  - In `C:\chronos2\crates\chronos_weaver\workflows\reference_object_white_bg.json`, tune EmptyLatentImage to 512x512 with 20 steps to match TripoSR's native 512x512 input resolution and deliver ~11-second reference generation.
- Files touched:
  - `C:\Users\m\AppData\Local\CS\ComfyUI\extra_model_paths.yaml`
  - `C:\chronos2\crates\chronos_weaver\workflows\reference_object_white_bg.json`
- Expected outcome: ComfyUI serves `dreamshaper_8.safetensors` at 512x512 in ~11 seconds with only ~1.5 GB VRAM.

### Step 2 — Engine Manifestation Worker Upgrades
- [ ] Action:
  - In `crates/engine/src/modes/inner_chambers/manifestation.rs`:
    - Before spawning `chronos.exe`, make a non-blocking `POST http://127.0.0.1:8000/free` call to flush residual VRAM tensors.
    - Set environment variable `CHRONOS_FORGE_REFERENCE_CKPT` in the spawned `chronos.exe` command to `"dreamshaper_8.safetensors"` (unless already set by the operator).
    - Set `CHRONOS_FLUX_PROFILE=lowvram` in child environment to protect any fallback paths.
    - Add a live elapsed seconds counter to the HUD status text while in `forge_plan` and `geometry_forge` so the operator sees active progress (e.g. `Planning geometry (12s elapsed) — Generating 2D reference...`).
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
- Expected outcome: Manifestation worker triggers clean VRAM flush, invokes Dreamshaper 8, and shows real-time timer on HUD.

### Step 3 — Verification & Inner Loop Delivery
- [ ] Action:
  - Run `cargo test --workspace` to ensure all tests pass.
  - Run `pwsh -File scripts\install_shortcut.ps1` to update `%LOCALAPPDATA%\Programs\Archetypes` and refresh the pinned Taskbar icon.
  - Launch Archetypes from the pinned launcher, walk up to the manifestation pedestal, press `E`, and type a test prompt.
  - Verify complete manifestation lifecycle:
    - Real-time timer counts up on HUD.
    - Reference generation completes in ~11s.
    - TripoSR builds 3D mesh in ~10s.
    - Blender imports OBJ and exports GLB in ~2s.
    - Pedestal underglow charges -> lightning flashes -> smoke dissipates -> manifested relic appears rotating on cushion.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
- Expected outcome: End-to-end manifestation succeeds cleanly in ~25 seconds with 0 GPU thrashing.

### Step 4 — Commit & Push to origin/main
- [ ] Action:
  - Commit all touched files on `main` with a clear, descriptive message.
  - Push to `origin/main` as required by AGENTS.md.
- Files touched:
  - repository tracked files
- Expected outcome: Clean git status, pushed to `origin/main`.
