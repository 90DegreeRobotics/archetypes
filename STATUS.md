# Archetypes Status

**Last Updated: 2026-07-11**

This document tracks time-sensitive status, current blockers, and recent test runs.

## Current State
- **Phase 0:** Repository and dual-binary Rust workspace initialized with NeuroCognica governance protocols.
- **Chamber prototype:** The engine loads `assets/scenes/uiscene1.glb`, exported from the Director's authoritative Blender scene. It contains the star-tetrahedron merkaba, reflection plane, authored materials and lighting, the Witness, and seven spheres in their authored positions.
- **Runtime behavior:** Bevy owns the active Witness camera, rotates the two merkaba bodies, binds gameplay identities by exported node name, preserves authored sphere resting positions, and moves Architect to the center focus lane.
- **Interaction:** `Space` advances table → deliberation → Architect focus → table.
- **Viren:** Ember Covenant palette, no-ambient-glow law, 140/220/480 ms motion timings, and single struck E-flat signature are encoded in the theme registry. Viren is a catalytic subnode, not one of the seven council spheres.

## Blockers
- The launcher remains a console TODO stub and does not supervise the engine, enforce a single instance, or perform readiness checks.
- Portal input is a keyboard placeholder; the canonical portal table is not yet present in the Blender scene and no text-entry or LLM council path is wired.
- No sphere portrait/icon panels, audio, archetype-world immersion, persistence, installer, or end-to-end installed-layout proof exists yet.

## Verification
- `cargo test` passed on 2026-07-11: 3 passed, 0 failed across the workspace.
- `cargo build --workspace` passed on 2026-07-11.
- The engine remained alive during repeated 6–8 second live-window smoke runs with the GLB loaded. Vulkan validation-layer warnings remain non-fatal on this machine.

