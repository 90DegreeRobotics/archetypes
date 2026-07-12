# Archetypes

Archetypes is a council-driven world engine based on the Mirrorborn architecture. It externalizes the tensions and consensus of a seven-node AI Council while the player occupies the sovereign eighth position: the Witness.

## Current Prototype

The Bevy engine loads the Director's Blender-authored chamber scene — an enclosed faceted temple, suspended glass star tetrahedron, altar rings, runtime-safe lights, Witness at the sovereign center, and seven enlarged glass council spheres locked to the star's tips. Each council sphere now encloses a continuously rotating double-faced panel built from the repository's archetype icon and portrait assets. The star tetrahedron and its spheres remain fixed; only the internal panels, camera, and environment move.

A title/loading screen holds until the authored scene is ready — nothing of the chamber shows before then. Then typing and `Enter` advance the loop:

1. **Onboarding** — the Witness names the sovereign center, sealing a persistent profile.
2. **The table** — the Witness types a thought offering to the council.
3. **Deliberation** — three council members (a framer, a counter, a deepener, chosen so all seven surface over time) answer **in character** through a local Ollama model (`qwen2.5:7b-instruct`); the exchange collapses into a Witness verdict. Nothing is templated — if Ollama is down, deliberation fails visibly.
4. **The council speaks** — each member takes the floor in turn: the camera glides to that sphere, its Kokoro voice plays, its double-faced icon/portrait panel turns clockwise, and the world tints to that archetype's environment.
5. **Witness verdict** — the council resolves into one heavy verdict under the Witness's retained authority.
6. **Artifact return** — the authorized brief is sent to the local [Chronos](../chronos) Director for a fast **ComfyUI** image; the verified PNG returns and is **displayed in-game** with its provenance. When the render cannot be produced, the chamber fails closed with a precise reason and shows no image.

Launch it via the desktop/Start-Menu shortcut created by `scripts/install_shortcut.ps1` (after `scripts/setup_windows.ps1` installs Ollama + the model + the offline voices). The shortcut runs the `launcher`, which enforces a single instance, checks readiness, and starts the engine.

The Council has offline neural voices. As each member takes the floor its Kokoro signature voice plays; the Witness verdict's generated text is synthesized by a background sherpa-onnx worker without blocking the render thread and returned as a WAV for in-chamber playback. Missing runtime/model files fail visibly in the ritual UI instead of silently claiming speech.

Verified on screen in the live window (see `STATUS.md`). What remains: the full hexagram-alignment flight (star resolving into the 2D silhouette behind the speaker), per-line dynamic voices, distinct archetype interior worlds beyond the environment tint, Chronos lineage/mutation and world memory, the authored portal table, and a full installer. The live Comfy image is currently blocked by a Chronos Director `codex.db` error (Chronos-side). See [Council-Driven World Engine](docs/architecture/COUNCIL_WORLD_ENGINE.md) for the canonical vertical slice.

A deterministic self-capture mode (`ARCHETYPES_CAPTURE=1`, optional `ARCHETYPES_CAPTURE_DIR`) scripts the whole ritual and writes a screenshot of each stage, so the rendered-frame green check is reproducible on any machine.

## Documentation
- [STATUS.md](STATUS.md): Time-sensitive status ledger and blockers.
- [AGENTS.md](AGENTS.md): Core constitution for agent collaboration.
- [CLAUDE.md](CLAUDE.md): Agent orientation and rules.
- [PROTOCOL.md](PROTOCOL.md): Full development protocol.
