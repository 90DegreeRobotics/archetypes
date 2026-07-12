# Archetypes Status

**Last Updated: 2026-07-12**

This document tracks time-sensitive status, current blockers, and recent test runs.

## Current State
- **Phase 0:** Repository and dual-binary Rust workspace initialized with NeuroCognica governance protocols.
- **Chamber prototype:** The engine loads `assets/scenes/uiscene1.glb`, exported from the Director's authoritative Blender scene. It contains the star-tetrahedron merkaba, reflection plane, authored materials and lighting, the Witness, and seven spheres in their authored positions.
- **Ritual loop:** One continuous council-world ritual runs through the live app — Onboarding (persistent Witness profile) → table offering → deliberation → Architect focus → Architect interior → Witness verdict → Chronos artifact return. Typing and `Enter` drive it.
- **World encapsulation (new 2026-07-12):** The `ArchetypeTheme` registry is no longer dead code. On entering the Architect interior the world inverts to the *Luminous Blueprint* environment — the ceremonial dark void lerps to the archetype's luminous void and the global ambient light floods to structural clarity — driven generically by `CurrentFocus`. The focused sphere now holds the sovereign center through the whole interior sequence.
- **Artifact return (new 2026-07-12):** On a verified Chronos render the returned PNG is staged into the asset tree and displayed in-game as an image with its artifact id and provenance. If Chronos is unreachable or not ready, the chamber fails closed with a precise reason and shows no image (never a placeholder).
- **Runtime behavior:** Bevy owns the active Witness camera, rotates the two merkaba bodies, binds gameplay identities by exported node name, preserves authored sphere resting positions, and moves Architect to the center focus lane.
- **Viren:** Ember Covenant palette, no-ambient-glow law, 140/220/480 ms motion timings, and single struck E-flat signature are encoded in the theme registry. Viren is a catalytic subnode, not one of the seven council spheres.

## Blockers
- The launcher remains a console TODO stub and does not supervise the engine, enforce a single instance, or perform readiness checks.
- Only the Architect world exists; the other six archetype interiors are not built. The centered focus sphere sits inside the merkaba (no forward "focus corridor" yet), so it reads as partly occluded.
- The canonical portal table is not yet present in the Blender scene; portal input is a screen-space text overlay rather than an in-world surface.
- No sphere portrait/icon panels, audio, Chronos lineage/mutation or world memory, installer, or end-to-end installed-layout proof exists yet. Persistence covers the Witness profile and the last artifact receipt only.

## Verification
- `cargo test --workspace` passed on 2026-07-12: 9 passed, 0 failed (engine 9, launcher 0).
- `cargo build --workspace` passed on 2026-07-12.
- **Live-window, end-to-end (2026-07-12):** The deterministic capture mode drove the full ritual through the running GPU app and saved eight stage screenshots (onboarding → table → deliberation → Architect focus → Architect interior → Witness verdict → artifact pending → artifact result). The Architect interior showed the luminous inversion; the artifact-result frame showed a real Chronos/Blender render (~120 s) displayed in-chamber with `Status: complete` and a verified `C:\chronos\renders\...png` path. Chronos Director reported `readiness: ready` for this run.
- Vulkan validation-layer warnings remain non-fatal on this machine.

