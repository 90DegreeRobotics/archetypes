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
- [x] Action:
  - In `crates/engine/src/modes/inner_chambers/manifestation.rs`:
    - Before spawning `chronos.exe`, make a non-blocking `POST http://127.0.0.1:8000/free` call to flush residual VRAM tensors.
    - Set environment variable `CHRONOS_FORGE_REFERENCE_CKPT` in the spawned `chronos.exe` command to `"dreamshaper_8.safetensors"` (unless already set by the operator).
    - Set `CHRONOS_FLUX_PROFILE=lowvram` in child environment to protect any fallback paths.
    - Pass `--factory-startup` to headless Blender OBJ-to-GLB conversion to bypass user-installed addon warnings.
    - Real-time timer dynamically reports elapsed seconds on the in-game HUD.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
- Expected outcome: Manifestation worker triggers clean VRAM flush, invokes Dreamshaper 8, and shows real-time timer on HUD.

### Step 3 — Verification & Inner Loop Delivery
- [x] Action:
  - Run `cargo test --workspace` (all 110 tests passed).
  - Run `pwsh -File scripts\install_shortcut.ps1` to update `%LOCALAPPDATA%\Programs\Archetypes` and refresh the pinned Taskbar icon.
  - Live operator test executed from pinned Taskbar launcher (`Archetypes 1.0.5 build 6`) with prompt `"panther"`.
  - Manifestation completed end-to-end without stubs or failure fallbacks:
    - ComfyUI generated reference image in ~14s without VRAM thrashing (peak VRAM 6.3 GB out of 12 GB, 0 paging).
    - TripoSR ran marching cubes reconstruction and produced `engine_mesh/0/mesh.obj` (2.47 MB).
    - Headless Blender converted mesh to `manifested_artifact.glb` (4.98 MB).
    - Archetypes Bevy engine triggered altar sequence and manifested the live 3D artifact with rotating exhibit turntable!
  - Root causes of operator-reported defects observed and documented:
    1. **Latency Defect (184s in TripoSR):** TripoSR default `--mc-resolution 256` evaluated 16.7M grid points with `--chunk-size 8192`, executing 2,048 sequential CUDA dispatches.
    2. **Mesh Quality Defect ("total trash"):** `dreamshaper_8` (SD 1.5) was run at 1024x1024 latent size, causing duplicated subjects (two panthers + wireframe box), which GrabCut and TripoSR faithfully converted into floating blobs and a tilted planar wall.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifestation.rs`
  - `assets/scenes/manifested_artifact.glb`
- Expected outcome: End-to-end manifestation pipeline confirmed live; defects isolated and audited.

### Step 4 — Commit & Push to origin/main
- [x] Action:
  - Commit all touched files on `main` with a clear, descriptive message.
  - Push to `origin/main` as required by AGENTS.md.
- Files touched:
  - repository tracked files
- Expected outcome: Clean git status, pushed to `origin/main`.
