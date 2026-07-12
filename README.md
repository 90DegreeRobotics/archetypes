# Archetypes

Archetypes is a council-driven world engine based on the Mirrorborn architecture. It externalizes the tensions and consensus of a seven-node AI Council while the player occupies the sovereign eighth position: the Witness.

## Current Prototype

The Bevy engine loads the Director's Blender-authored chamber scene — the star-tetrahedron merkaba, reflection plane, lights, materials, Witness, and seven council spheres in their authored three-dimensional positions — and drives one continuous council-world ritual on top of it.

Typing and `Enter` advance the loop:

1. **Onboarding** — the Witness names the sovereign center, sealing a persistent profile.
2. **The table** — the Witness types a thought offering to the council.
3. **Deliberation** — the merkaba turns and the Architect glides to the sovereign center.
4. **Architect interior** — the world itself inverts into the Architect's *Luminous Blueprint* environment: the ceremonial dark void becomes luminous silver-white and the ambient light floods to structural clarity. This is a real environmental translation of the archetype's design tokens, not a caption.
5. **Witness verdict** — the council resolves a buildable brief under the Witness's retained authority.
6. **Artifact return** — the authorized brief is sent to the local [Chronos](../chronos) Director, which renders a still through Blender; the verified PNG returns and is **displayed in-game** inside the chamber, with its artifact id and provenance. When Chronos is not ready, the chamber fails closed with a precise reason and shows no image.

Verified end-to-end in the live window against a running Chronos Director (see `STATUS.md`). What remains: the other six archetype worlds, a forward "focus corridor" so the centered sphere is not occluded by the merkaba, portrait/icon sphere faces, Chronos lineage/mutation and world memory, audio, the authored portal table, installer delivery, and launcher supervision. See [Council-Driven World Engine](docs/architecture/COUNCIL_WORLD_ENGINE.md) for the canonical vertical slice.

A deterministic self-capture mode (`ARCHETYPES_CAPTURE=1`, optional `ARCHETYPES_CAPTURE_DIR`) scripts the whole ritual and writes a screenshot of each stage, so the rendered-frame green check is reproducible on any machine.

## Documentation
- [STATUS.md](STATUS.md): Time-sensitive status ledger and blockers.
- [AGENTS.md](AGENTS.md): Core constitution for agent collaboration.
- [CLAUDE.md](CLAUDE.md): Agent orientation and rules.
- [PROTOCOL.md](PROTOCOL.md): Full development protocol.

