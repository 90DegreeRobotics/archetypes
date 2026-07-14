# CODEX LANE A — The Oracle's Riddle (Reverse Prompt)

**Owner: Claude. Status: WAITING ON LANE 0.**
**Cold-start brief — everything you need is in this file.**

## What you are building
The depth-first "front door" mode: machine as puzzle. A hidden 3-word prompt generates an image. The player reconstructs the prompt with embedding-tolerant per-word scoring.

## File ownership (touch ONLY these)
- `crates/engine/src/modes/oracle_riddle/**` (new)

## Build spec
1. Loop: hidden 3-word prompt -> cached/generated image -> player reconstructs -> embedding-tolerant per-word scoring -> seal round to ledger -> next.
2. Difficulty tiers = semantic distance (Beginner: Dog/Red/Running -> Impossible: Entropy/Memory/Bloom).
3. Consumes `services::{llm,chronos,ledger}` + mode framework from Lane 0.
4. Competitive shells (Daily/Speed/Hardcore/Infinite/Versus) layer on the base loop.
