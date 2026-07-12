# Archetypes Status

**Last Updated: 2026-07-12**

This document tracks time-sensitive status, current blockers, and recent test runs.

## Current State
- **Phase 0:** Repository and dual-binary Rust workspace initialized with NeuroCognica governance protocols.
- **Chamber prototype:** The engine loads `assets/scenes/uiscene1.glb`, exported from the isolated Blender working copy `uiscene1.codex-temple.blend` without modifying the operator's original. It contains an enclosed basalt temple and vault, altar rings, a fixed cyan/magenta glass star tetrahedron with warm metal edges, glTF-compatible authored lighting, the Witness, and seven enlarged council vessels locked to the star's tips.
- **Archetype vessels (new 2026-07-12):** All seven council spheres now use deliberately art-directed translucent glass rather than blindly copying canonical theme colors. Each encloses a continuously rotating, double-faced icon/portrait panel using the previously untouched `assets/icons/*` and `assets/aura/*` artwork. The exported GLB contains seven animation actions and all fourteen art panels.
- **Camera composition (new 2026-07-12):** The Blender Witness camera is authored inside the temple with a 34 mm wide establishing frame that contains the suspended council composition and altar. The camera is now included in the GLB scene contract; Bevy's existing runtime camera remains the active live view and the state-driven alignment flight is still outstanding.
- **Ritual loop:** One continuous council-world ritual runs through the live app — Onboarding (persistent Witness profile) → table offering → deliberation → Architect focus → Architect interior → Witness verdict → Chronos artifact return. Typing and `Enter` drive it.
- **World encapsulation (new 2026-07-12):** The `ArchetypeTheme` registry is no longer dead code. On entering the Architect interior the world inverts to the *Luminous Blueprint* environment — the ceremonial dark void lerps to the archetype's luminous void and the global ambient light floods to structural clarity — driven generically by `CurrentFocus`.
- **Artifact return (new 2026-07-12):** On a verified Chronos render the returned PNG is staged into the asset tree and displayed in-game as an image with its artifact id and provenance. If Chronos is unreachable or not ready, the chamber fails closed with a precise reason and shows no image (never a placeholder).
- **Runtime behavior:** Bevy owns the active Witness camera and binds gameplay identities by exported node name. The star tetrahedron and every sphere are held fixed at their authored positions — the star does not rotate and no sphere ever moves. Focus is expressed by camera movement and environment change, not by relocating geometry.
- **Viren:** Ember Covenant palette, no-ambient-glow law, 140/220/480 ms motion timings, and single struck E-flat signature are encoded in the theme registry. Viren is a catalytic subnode, not one of the seven council spheres.

## Blockers
- The launcher remains a console TODO stub and does not supervise the engine, enforce a single instance, or perform readiness checks.
- The camera does not yet fly to the fixed alignment that turns the star into the hexagram silhouette and brings the selected archetype's sphere to face the Witness. Current camera motion is a placeholder zoom.
- Only the Architect world exists; the other six archetype interiors are not built.
- The canonical portal table is not yet present in the Blender scene; portal input is a screen-space text overlay rather than an in-world surface.
- No audio, Chronos lineage/mutation or world memory, installer, or end-to-end installed-layout proof exists yet. Persistence covers the Witness profile and the last artifact receipt only.

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
- **Temple export proof (2026-07-12):** Blender re-imported the final GLB with 82 objects, 65 meshes, 7 supported lights, 7 panel animation actions, 16 embedded images, 21 named icon/portrait panel nodes, and 41 temple nodes. The Blender visual proof is `artifacts/visual-proof/temple-overhaul-0040.png`.
