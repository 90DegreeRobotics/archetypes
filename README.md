# Archetypes

Archetypes is a council-driven world engine based on the Mirrorborn architecture. It externalizes the tensions and consensus of a seven-node AI Council while the player occupies the sovereign eighth position: the Witness.

## Current product (0.3.0)

The Desktop shortcut launches the supervising `launcher`. If Ollama, Chronos Director, or ComfyUI are installed and down, the launcher starts them, then fail-closes unless Chronos Director reports Sentinel authority in `enforce` mode. Missing voices are repaired from the pinned sidecar manifest. Every launch refusal opens `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\last-failure.txt` in Notepad.

A plain black ceremonial veil fades in `ARCHETYPES`, then `A GAME BY MICHAEL HOLT`, and settles on the lore-chamber main menu:

1. **STANDARD MODE** — the AURA conversational council. The Witness names the sovereign seat (once), offers language at the portal table, three council members (framer, counter, deepener) answer in character through local Ollama (`qwen2.5:7b-instruct`), the exchange collapses into a Witness verdict, Kokoro voices play through the sherpa CLI with a WAV cache (in-process C API is opt-in; the pinned sidecar AVs the engine), the camera flies to the hexagram axis of the speaking sphere, that archetype's interior law surrounds the star, and Chronos paints the authorized image. Accepted artifacts persist as world memory tokens on the table. Esc returns to the menu.
2. **CONSCIOUSNESS** — 1:1 counsel with one archetype (Seed-of-Life selector, durable JSONL history, Chronos/Comfy reply images).
3. **ORACLE RIDDLE** — reverse-prompt from a Chronos image. Concrete visual triples, order-insensitive scoring, Insight rewards. A truth extracted in Inner Chambers can seed the next round.
4. **INNER CHAMBERS** — walk seven distinct minds around a hub. Align with a node and press E to extract a three-word truth.
5. **LIVING ENGINE** — orbital instrument. Tune three prototype resonances (Architect, Sentinel, Oracle), keep the three-layer Aura from starving or overloading, confront a Viren strain, seat two implants.
6. **HELP** — in-game Witness manual. The same text is installed as Start Menu **Archetypes Help**.
7. **QUIT GAME**

Lane 0 spine (`services::{llm, chronos, paths, ledger}` and `modes::{game_mode, difficulty}`) remains the shared metabolism. Events seal into a hash-chained JSONL ledger under `%LOCALAPPDATA%\NeuroCognica\Archetypes\data`.

Launch via the Desktop/Start-Menu shortcut created by `scripts/install_shortcut.ps1` (developer restage) or `scripts/install_product.ps1` (per-user/Program Files install + Add/Remove Programs + uninstall). `scripts/setup_windows.ps1` still installs Ollama, the model, and the offline voices. There is no environment-variable bypass for the Chronos/Sentinel launch gate.

Verified functional paths and operator-reviewed visuals are separated in `STATUS.md`. See [Council-Driven World Engine](docs/architecture/COUNCIL_WORLD_ENGINE.md) for the canonical vertical slice.

## Documentation
- [STATUS.md](STATUS.md): Time-sensitive status ledger and blockers.
- [AGENTS.md](AGENTS.md): Core constitution for agent collaboration.
- [CLAUDE.md](CLAUDE.md): Agent orientation and rules.
- [PROTOCOL.md](PROTOCOL.md): Full development protocol.
