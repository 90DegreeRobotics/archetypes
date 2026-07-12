# Archetypes Status

**Last Updated: 2026-07-12**

This document tracks time-sensitive status, current blockers, and recent test runs.

## Current State
- **Phase 0:** Repository and dual-binary Rust workspace initialized with NeuroCognica governance protocols.
- **Chamber prototype:** The engine loads `assets/scenes/uiscene1.glb`, exported from the Director's authoritative Blender scene. It contains the fixed star tetrahedron (two interlocked tetrahedra — `Star_Tetra_A`/`Star_Tetra_B`, no "Merkaba" label, no rotation), now **crystal-clear glass with black wireframe edges** so the glass spheres read; the reflection plane; authored materials and lighting; the Witness at the sovereign center; and seven spheres locked to the star's tips.
- **Ritual loop:** One continuous council-world ritual runs through the live app — Onboarding (persistent Witness profile) → table offering → deliberation → Architect focus → Architect interior → Witness verdict → Chronos artifact return. Typing and `Enter` drive it.
- **World encapsulation (new 2026-07-12):** The `ArchetypeTheme` registry is no longer dead code. On entering the Architect interior the world inverts to the *Luminous Blueprint* environment — the ceremonial dark void lerps to the archetype's luminous void and the global ambient light floods to structural clarity — driven generically by `CurrentFocus`.
- **Artifact return (new 2026-07-12):** On a verified Chronos render the returned PNG is staged into the asset tree and displayed in-game as an image with its artifact id and provenance. If Chronos is unreachable or not ready, the chamber fails closed with a precise reason and shows no image (never a placeholder).
- **Runtime behavior:** Bevy owns the active Witness camera and binds gameplay identities by exported node name. The star tetrahedron and every sphere are held fixed at their authored positions — the star does not rotate and no sphere ever moves. Focus is expressed by camera movement and environment change, not by relocating geometry.
- **Viren:** Ember Covenant palette, no-ambient-glow law, 140/220/480 ms motion timings, and single struck E-flat signature are encoded in the theme registry. Viren is a catalytic subnode, not one of the seven council spheres.

## Blockers
- The launcher remains a console TODO stub and does not supervise the engine, enforce a single instance, or perform readiness checks.
- The set is still an open-edge void, not the intended enclosed ancient temple (star hung in a chosen sky-north above a circular table whose center is the stargate/prompt wavefield, camera looking up).
- The camera does not yet fly to the fixed alignment that turns the star into the hexagram silhouette and brings the selected archetype's sphere to face the Witness. Current camera motion is a placeholder zoom.
- Only the Architect world exists; the other six archetype interiors are not built.
- The canonical portal table is not yet present in the Blender scene; portal input is a screen-space text overlay rather than an in-world surface.
- No sphere portrait/icon panels, audio, Chronos lineage/mutation or world memory, installer, or end-to-end installed-layout proof exists yet. Persistence covers the Witness profile and the last artifact receipt only.

## Verification
- `cargo test --workspace` passed on 2026-07-12: 9 passed, 0 failed (engine 9, launcher 0).
- `cargo build -p engine` rebuilt the runnable binary on 2026-07-12 before every capture.
  (Lesson recorded: `cargo test` builds test harnesses, **not** the runnable `engine.exe`;
  a prior "static" claim was made against a stale binary and was wrong.)
- **Static star — mechanical proof (2026-07-12):** the re-exported GLB has `animations: 0`
  and no engine code rotates anything. Two Onboarding frames captured 3.5 s apart (camera
  provably fixed in that state) differ on **6 of 3,686,400 pixels**, max delta **2/255** —
  i.e. nothing moves. Rotation would have changed thousands of pixels.
- **Glass retheme (2026-07-12):** the star tetrahedron renders crystal-clear with black
  wireframe edges; the glass spheres are the visual focus (verified in the live window).
- **Earlier end-to-end (prior build, 2026-07-12):** capture mode drove the full ritual and
  the artifact-result frame showed a real ~120 s Chronos/Blender render displayed in-chamber
  (`Status: complete`, verified `C:\chronos\renders\...png`). The interior-inversion and
  artifact-display systems are unchanged by the star/glass work.
- Vulkan validation-layer warnings remain non-fatal on this machine.

