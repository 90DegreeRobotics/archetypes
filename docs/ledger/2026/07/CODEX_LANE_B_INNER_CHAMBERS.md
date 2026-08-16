# CODEX LANE B — The Inner Chambers (Architect)

**Owner: Codex. Status: COMPLETED (playable Architect interior shipped in product rebuild 0.2.0 — `plan_2026-08-16_1045_product-rebuild.md`). Historical brief below is the original lane contract.**
**Cold-start brief — everything you need is in this file.**

## What you are building
One fully-realized interior (Architect's grid-alignment world). Introduces the first player-navigable camera. Machine as place. Reading the space = reading the archetype's mind; extracted truth seeds a future Mode-A round.

## File ownership (touch ONLY these)
- `crates/engine/src/modes/inner_chambers/**` (new)

## Build spec
1. One fully realized interior (Architect's grid-alignment world).
2. Introduces the first player-navigable camera.
3. Reading the space = reading the archetype's mind; extracted truth seeds a future Mode-A round.
4. Manage VRAM budget explicitly if Flux is resident (unload the previous scene).
5. Consumes mode framework + ledger from Lane 0.
