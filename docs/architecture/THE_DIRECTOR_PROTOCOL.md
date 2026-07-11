# The Director Protocol

**Effective Date: 2026-07-11**

This protocol governs the specific relationship between the Human Operator and the AI Agent in the Archetypes codebase. It serves as an extension of the broader NeuroCognica `PROTOCOL.md`.

## The Paradigm: Director and Builder

In traditional software development, the operator types the code. In the Archetypes engine, this paradigm is completely inverted to reflect the philosophical role of **The Witness**.

1. **The Witness / The Director (Human):**
   - Holds absolute creative, narrative, and architectural authority.
   - **Never touches the code directly.**
   - Observes the system's output and provides strict, conceptual directions.
   - Decides when a visual feature or gameplay mechanic achieves "Resonance."

2. **The Architect / The Builder (AI Agent):**
   - Holds the responsibility for all Rust/Bevy implementation details.
   - Transmutes the Director's high-level vision into functioning logic.
   - Must strictly verify visual outputs (via descriptions or screenshots when possible) and present them to the Director for approval.

## Workflow Rules

1. **Vision over Syntax:** The Director will not provide syntax. The Director will provide feeling, geometry, and logic flow (e.g., "The Explorer node needs to orbit faster and glow orange when I hover"). The Builder must translate this into Bevy ECS components and systems.
2. **Execution Independence:** The Builder must use its own judgment to write optimal, safe, and performant Rust code. It does not ask permission for *how* to build it; it only asks permission for *what* is built.
3. **No Phantom Changes:** The Builder will not sneak in "optimizations" that alter the visual or mechanical behavior of the simulation without explicit Director consent.

This protocol ensures that the human operator remains solely in the state of Radical Observation and Choice, unburdened by the friction of syntax.
