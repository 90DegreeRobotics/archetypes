# Plan: Character Authoring Template and Scene Setup in Blender — 2026-09-13 03:51

## Status
COMPLETED — 2026-09-13 03:57

## Goal
Establish a modular, reusable character authoring template and scene setup pipeline in Blender 4.5 for at least 8 game characters, starting with the Nebula Jester reference image (`C:\Users\m\Downloads\ChatGPT Image Sep 9, 2026, 11_49_42 AM.png`). The setup focuses on reference-aligned modeling and a game-ready biped animation rig template, setting up standard collections, calibrated scale, mirrored blockout geometry, and studio viewport ergonomics.

## Steps

### Step 1 — Ingest and Prepare Character Reference Asset
- [x] Action:
  - Copy and register the Nebula Jester reference image into `assets/authoring/characters/nebula_jester/reference_front.png`.
  - Also generate a chroma-keyed transparent alpha version `reference_front_alpha.png` so Blender viewport X-Ray and wireframe modeling is clear and distraction-free without blinding green glare.
- Files touched:
  - `assets/authoring/characters/nebula_jester/reference_front.png`
  - `assets/authoring/characters/nebula_jester/reference_front_alpha.png`
- Expected outcome: Clean reference assets stored in canonical repository structure.

### Step 2 — Build the Automated Blender Scene Setup Script
- [x] Action:
  - Create `scripts/setup_character_scene.py` and `scripts/build_character_template.py` executed via Blender 4.5 (`C:\Program Files\Blender Foundation\Blender 4.5\blender.exe`).
  - Configure:
    - Metric unit scale ($1.0$).
    - Collection `01_Reference`: Orthographic reference image plane and empty centered on $X=0$, feet flush at $Z=0$, height matched to $1.95\text{m}$.
    - Collection `02_Blockout_Mirrored`: Clean symmetrical humanoid blockout geometry (Head, Neck, Shoulders, Horns, Bells, Torso, Pelvis, Limbs) with active X-Mirror modifiers (clipping enabled).
    - Collection `03_Rig_Armature`: Clean biped armature (Root, Hips, Spine, Chest, Neck, Head, Clavicle, Arm, Forearm, Hand, Thigh, Shin, Foot, Toe, Horns) with X-Axis mirror enabled.
    - Collection `04_Guides_Studio`: Three-point studio lighting, ground circle at $Z=0$, Front Ortho camera.
  - Saves out both a clean generic template `assets/authoring/characters/template/character_template.blend` and the configured character project `assets/authoring/characters/nebula_jester/nebula_jester_modeling.blend`.
- Files touched:
  - `scripts/setup_character_scene.py`
  - `scripts/build_character_template.py`
  - `assets/authoring/characters/template/character_template.blend`
  - `assets/authoring/characters/nebula_jester/nebula_jester_modeling.blend`
- Expected outcome: Ready-to-open `.blend` scenes with full organization.

### Step 3 — Verify Scene Generation and Inspect Visual Render
- [x] Action:
  - Run the scene generator via Blender 4.5.
  - Render front-view orthographic and 3/4 perspective test viewport images into `artifacts/visual-proof/character_setup/` to visually verify camera, reference alignment, blockout symmetry, and armature positioning.
- Files touched:
  - `artifacts/visual-proof/character_setup/viewport_preview.png`
  - `artifacts/visual-proof/character_setup/perspective_preview.png`
- Expected outcome: Visual proof confirming accurate scale, alignment, and viewport readiness.

### Step 4 — Document the 8-Character Modeling & Rigging Workflow
- [x] Action:
  - Provide a practical tutorial and reference guide on how to use the scene, shortcuts, mirror modeling steps, and how to replicate it for the other 7 characters.
- Files touched:
  - `docs/architecture/CHARACTER_MODELING_AND_RIGGING_PIPELINE.md`
- Expected outcome: Reusable, documented workflow.

## Execution Record
- Ingested raw reference and generated spill-suppressed chroma-keyed alpha image (`reference_front_alpha.png`).
- Executed `scripts/build_character_template.py` through headless Blender 4.5.4 LTS.
- Generated `nebula_jester_modeling.blend` and master `character_template.blend`.
- Rendered front-view orthographic preview (`viewport_preview.png`) and 3/4 perspective proof (`perspective_preview.png`).
- Verified quad tube generation, limb alignment, horn curvature, bell tips, and armature mirror symmetry.
- Documented full pipeline and hotkeys in `docs/architecture/CHARACTER_MODELING_AND_RIGGING_PIPELINE.md`.
