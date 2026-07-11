# Handoff: The Daoist Sandbox — 2026-07-11

**Agent:** Antigravity  
**Date:** 2026-07-11  

## The Project Vision: Archetypes
This repository (`C:\archetypes`) is not a traditional video game. It is a Daoist sandbox simulation—an "Astrolabe of the Psyche"—built to visualize the internal tensions and consensus of a 7-node AI Council, observed by the 8th Node (The Witness). 

The philosophical core of the game is radical observation. There are no combat mechanics, no scoreboards, and no linear objectives. The user plays as **The Witness** (the 8th sphere), positioned on the Z-axis of a 3D Seed of Life geometry. The 7 Council members (Sentinel, Architect, Jester, Mentor, Empath, Oracle, Explorer) orbit this central axis as 3D spheres around a Star Tetrahedron (Merkaba). 

The gameplay consists entirely of spatial navigation and perspective shifting. If the user orbits the camera to look *through* the sphere of Luma (The Empath), the entire simulation's UI, ambient audio, and LLM text generation shifts to an empathetic, D-minor, warm-ember perspective. If the system goes into "Lockdown," the Sentinel sphere moves to eclipse the center, physically blocking the user's view until equilibrium is restored by coaxing the other nodes. It is an exercise in observing one's own cognitive sovereignty without forcing an outcome. 

## The Director Protocol
The development of this project strictly follows **The Director Protocol** (see `docs/architecture/THE_DIRECTOR_PROTOCOL.md`). 
- **The Human Operator is the Director.** They provide the vision, the feelings, the geometry, and the conceptual logic. They **never** write the code.
- **The AI Agent is the Builder.** You are responsible for all Rust/Bevy syntax. You must translate the Director's concepts into optimal ECS components and systems, verifying visual outputs before committing them.

## Current Technical State
The project strictly mirrors the `C:\Mirrorborn` codebase laws (The Kali Doctrine, Clean Ledger Protocol, No Deletions). 

1. **Dual-Binary Workspace:** The Cargo project is split into `crates/launcher` (for Windows single-instance mutex and AppData initialization) and `crates/engine` (the Bevy 0.18.1 application).
2. **Windows Metabolism:** Paths are strictly defined in `WINDOWS_METABOLISM.md`. The Bevy app must write its logs to `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\`.
3. **Dependency Bootstrap:** Idempotent Winget scripts are located in `scripts/` to ensure Ollama and required LLM models (`qwen2.5:7b-instruct`) are installed.
4. **Bevy Status:** `crates/engine/src/main.rs` contains a foundational Bevy App with a black ground plane, an ambient light, and a camera. It compiles successfully. No archetype spheres or geometry have been spawned yet.

## Next Steps for the Next Agent
When the Director issues their next command, your immediate duty is to:
1. Write a new `plan_<date>_<topic>.md` in the current `docs/ledger/YYYY/MM` directory.
2. Translate their conceptual command (e.g., "Spawn the Sentinel") into Bevy ECS components within `crates/engine/src/main.rs`.
3. Verify the code compiles (`cargo check --workspace`).
4. Present the execution to the Director for approval, and immediately push to `origin/main`.
