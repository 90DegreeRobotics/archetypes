# Plan: Manifestation performance and quality — 2026-09-09 20:30

## Status
COMPLETED

## Goal

Reduce real prompt-to-pedestal latency to a buyer-credible budget on the RTX
3060 without accepting disconnected or scene-shaped TripoSR output. Measure the
actual shared-GPU contention, drive the game at a low-power cadence while a
manifestation is running, consume Chronos2's tuned reconstruction contract, and
prove the result through a real installed Taskbar launch.

## Steps

### Step 1 — Establish real bottlenecks
- [x] Action: Inspect the panther and strawberry bundles, receipts, generated
  references, prepared inputs, process tree, VRAM, and per-process GPU-engine
  counters.
- Files touched: none.
- Expected outcome: Separate proven bottlenecks from speculative explanations.

### Step 2 — Yield GPU capacity during manifestation
- [x] Action: Cap/reduce Bevy update and render cadence while Chronos2 owns the
  generation stage, restoring normal cadence after success, failure, or cancel.
- Files touched: engine runtime and tests.
- Expected outcome: The 4K chamber does not consume most of the GPU while the
  player is waiting for GPU inference.

### Step 3 — Consume measured Chronos2 quality/performance settings
- [x] Action: Keep the no-recipe subprocess contract while accepting explicit
  reconstruction tuning and truthful stage/timing receipts from Chronos2.
- Files touched: manifestation bridge and tests as required.
- Expected outcome: Latency settings come from the generating product and remain
  recorded in the receipt.

### Step 4 — Verify the buyer path
- [x] Action: Run the full Rust gate, refresh the installed Taskbar product,
  measure idle and manifestation GPU load, run real diverse prompts, inspect
  artifacts, verify cancellation, and record exact timings.
- Files touched: evidence/plan records and only intentional runtime artifacts.
- Expected outcome: A fast result is not accepted unless it is recognizable,
  connected, and visibly placed on the pedestal.

### Step 5 — Publish
- [x] Action: Commit only owned files, preserve the live generated manifested
  GLB as evidence, push `main`, and report proven versus remaining work.
- Files touched: named files from this plan.
- Expected outcome: Clean owned change set with no claim beyond evidence.

## Results

- Full workspace gate: 112 passed, 0 failed.
- Canonical Taskbar install completed and verified the installed engine hash as
  `9D7AA3A94ADB83513660D29277B5DC4B471ECBFA5789BDFE1C8F7D4133AF421D`.
- The exact pinned shortcut launched the installed binary with native title
  `Archetypes 1.0.5 (build 6) — Council Chamber`.
- Live Chronos trials produced a coherent strawberry and recognizable brass
  astrolabe. TripoSR fell from 191 seconds to 38.808 seconds under old-game GPU
  contention and 33.349 seconds without it.
- The game GLB importer reduced the strawberry from 187,461 to 74,997
  triangles and from 20,243,404 to 1,654,328 bytes while preserving one mesh
  and the `Color` vertex attribute.
- Trusted Windows Computer Use was unavailable (`Trusted RPC service is not
  configured: sky`), so the newly installed post-change player click-through
  could not be automated. The prior player screenshots prove the same prompt
  dispatch path; the new direct pipeline, importer, install hashes, and pinned
  launch were verified separately. The operator should re-witness the combined
  path physically.
