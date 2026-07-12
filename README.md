# Archetypes

Archetypes is a council-driven world engine based on the Mirrorborn architecture. It externalizes the tensions and consensus of a seven-node AI Council while the player occupies the sovereign eighth position: the Witness.

## Current Prototype

The Bevy engine loads the Director's Blender-authored chamber scene — an enclosed faceted temple, suspended glass star tetrahedron, altar rings, runtime-safe lights, Witness at the sovereign center, and seven enlarged glass council spheres locked to the star's tips. Each council sphere now encloses a continuously rotating double-faced panel built from the repository's archetype icon and portrait assets. The star tetrahedron and its spheres remain fixed; only the internal panels, camera, and environment move.

Typing and `Enter` advance the loop:

1. **Onboarding** — the Witness names the sovereign center, sealing a persistent profile.
2. **The table** — the Witness types a thought offering to the council.
3. **Deliberation** — the star tetrahedron holds fixed; the camera moves in on the Architect as the world begins to change.
4. **Architect interior** — the world itself inverts into the Architect's *Luminous Blueprint* environment: the ceremonial dark void becomes luminous silver-white and the ambient light floods to structural clarity. This is a real environmental translation of the archetype's design tokens, not a caption.
5. **Witness verdict** — the council resolves a buildable brief under the Witness's retained authority.
6. **Artifact return** — the authorized brief is sent to the local [Chronos](../chronos) Director, which renders a still through Blender; the verified PNG returns and is **displayed in-game** inside the chamber, with its artifact id and provenance. When Chronos is not ready, the chamber fails closed with a precise reason and shows no image.

The Council now has offline neural voices. When an archetype takes focus, Bevy immediately plays its Kokoro signature voice; when the Architect forms a verdict, a background sherpa-onnx worker synthesizes that generated text without blocking the render thread and returns a WAV for in-chamber playback. Keyboard input remains the Witness interface. Missing runtime/model files fail visibly in the ritual UI instead of silently claiming speech.

Verified end-to-end in the live window against a running Chronos Director (see `STATUS.md`). What remains: the state-driven camera-alignment flight that turns the fixed star to frame the selected sphere in the hexagram view, the other six archetype worlds, Chronos lineage/mutation and world memory, audio, the authored portal table, installer delivery, and launcher supervision. See [Council-Driven World Engine](docs/architecture/COUNCIL_WORLD_ENGINE.md) for the canonical vertical slice.

A deterministic self-capture mode (`ARCHETYPES_CAPTURE=1`, optional `ARCHETYPES_CAPTURE_DIR`) scripts the whole ritual and writes a screenshot of each stage, so the rendered-frame green check is reproducible on any machine.

## Documentation
- [STATUS.md](STATUS.md): Time-sensitive status ledger and blockers.
- [AGENTS.md](AGENTS.md): Core constitution for agent collaboration.
- [CLAUDE.md](CLAUDE.md): Agent orientation and rules.
- [PROTOCOL.md](PROTOCOL.md): Full development protocol.
