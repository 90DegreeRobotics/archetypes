# Archetypes

Archetypes is a council-driven world engine based on the Mirrorborn architecture. It externalizes the tensions and consensus of a seven-node AI Council while the player occupies the sovereign eighth position: the Witness.

## Current Prototype

The Bevy engine loads the Director's Blender-authored chamber scene — an enclosed faceted temple, suspended glass star tetrahedron, altar rings, runtime-safe lights, Witness at the sovereign center, and seven enlarged glass council spheres locked to the star's tips. Each sphere encloses a fixed double-faced archetype panel that stays upright and camera-readable.

A plain black ceremonial veil fades in `ARCHETYPES`, then `A GAME BY MICHAEL HOLT`, holds until the menu and chamber assets are ready, and slowly fades to the settled portal-table main menu:

1. **Choose mode** — `STANDARD MODE` explicitly begins the current ritual. `ORACLE RIDDLE` is the first non-standard playable mode: a reverse-prompt puzzle built from concrete visual triples, visible 3-word input validation, replay, order-insensitive scoring with alias credit, and player-facing Insight rewards. `INNER CHAMBERS` and `LIVING ENGINE` remain registered as locked future contracts; they are not playable stubs.
2. **Onboarding** — the Witness names the sovereign center, sealing a persistent profile.
3. **The portal table** — the Witness types an offering over a supported metallic-gold astrolabe table. Its emissive stargate rotates and pulses beneath the intent surface; submitting hides the table and reveals the council star.
4. **Deliberation** — three council members (a framer, a counter, a deepener, chosen so all seven surface over time) answer **in character** through a local Ollama model (`qwen2.5:7b-instruct`); the exchange collapses into a Witness verdict. Nothing is templated — if Ollama is down, deliberation fails visibly.
5. **The council speaks** — each member takes the floor in turn: the camera glides to that sphere, its Kokoro voice plays, its upright icon/portrait panel remains readable, and the world tints to that archetype's environment.
6. **Witness verdict** — the council resolves into one heavy verdict under the Witness's retained authority.
7. **Artifact return** — the authorized brief is sent to the local [Chronos](../chronos) Director's direct canvas-image endpoint. Chronos generates the standalone painting with its bounded SDXL workflow without rendering a museum/easel wrapper or invoking Blender. The verified PNG returns to the portal table with its sealed completion event.

Lane 0 has also split the shared spine into `services::{llm, chronos, paths, ledger}` and `modes::{game_mode, difficulty}`. The Standard Mode flow seals profile, offering, artifact request, and artifact result events into a local hash-chained JSONL ledger under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data`.

Launch it via the desktop/Start-Menu shortcut created by `scripts/install_shortcut.ps1` (after `scripts/setup_windows.ps1` installs Ollama + the model + the offline voices). The shortcut runs the `launcher`, which enforces a single instance, fail-closes unless Chronos Director reports Sentinel authority in `enforce` mode, writes a client-signed and body-bound `archetypes_launch_requested` event through Chronos `POST /api/v1/codex/append`, verifies Ollama, offline TTS, and Comfy (`:8000`), then starts the engine. There is no environment-variable bypass for the Chronos/Sentinel launch gate.

The Council has offline neural voices. As each member takes the floor its Kokoro signature voice plays; the Witness verdict's generated text is synthesized by a background sherpa-onnx worker without blocking the render thread and returned as a WAV for in-chamber playback. Missing runtime/model files fail visibly in the ritual UI instead of silently claiming speech.

Verified functional paths and operator-reviewed visuals are separated in `STATUS.md`. What remains: operator approval and iteration on the portal-table composition and transcript/sphere bubbles, the full hexagram-alignment flight, distinct archetype interior worlds beyond the environment tint, Chronos lineage/mutation and world memory, and a full installer. See [Council-Driven World Engine](docs/architecture/COUNCIL_WORLD_ENGINE.md) for the canonical vertical slice.

A deterministic self-capture mode (`ARCHETYPES_CAPTURE=1`, optional `ARCHETYPES_CAPTURE_DIR`) scripts the whole ritual and writes a screenshot of each stage, so the rendered-frame green check is reproducible on any machine.

## Documentation
- [STATUS.md](STATUS.md): Time-sensitive status ledger and blockers.
- [AGENTS.md](AGENTS.md): Core constitution for agent collaboration.
- [CLAUDE.md](CLAUDE.md): Agent orientation and rules.
- [PROTOCOL.md](PROTOCOL.md): Full development protocol.
