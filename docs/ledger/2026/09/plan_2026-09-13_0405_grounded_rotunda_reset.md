# Plan: Grounded rotunda architectural reset — 2026-09-13 04:05

## Status
COMPLETED

## Goal
Replace the live Inner Chambers shell that visibly z-fights, floats above an unreachable void,
and overwhelms the Manifester with a single grounded rotunda. Preserve the working altar,
manifestation, grabbing and persistence systems while giving the player one continuous,
stable floor, a reachable arcade wall, and a properly seated vault.

## Evidence

- Live installed walkthrough reproduced the operator's defect. Close floor views show broad
  rectangular interference bands, ghosted joints, and moving moire across the paving.
- `spawn_castle_platform` puts the solid cylinder's top at `GROUND_Y == 0.4` and then puts the
  flagstone top at the exact same height. Two opaque surfaces compete for every floor pixel.
- The live shell spawns seven gallery levels, 504 arcade scene instances, 672 balusters, 42
  high-range point lights, a 96m wall and a 30m vault.
- Collision deliberately returns `ABYSS_Y` between the 24m Council floor and the promenade at
  radius 100m, so the arcade cannot be reached on foot after the outer rooms were hidden.

## Steps

### Step 1 — Replace the live floor with one surface
- [x] Action: Stop layering physical flagstones over a coplanar cylinder and spawn one matte,
  continuous floor disc from the altar to the wall.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`.
- Expected outcome: No z-fighting, bleed-through, giant triangular seams, or floor shimmer.

### Step 2 — Ground and simplify the architectural shell
- [x] Action: Replace the seven-storey live ascent with one arcade ring, a lower enclosing wall,
  a seated vault, and a small bounded light set. Preserve the old ascent code as non-live history.
- Files touched: `crates/engine/src/modes/inner_chambers/world.rs`.
- Expected outcome: One coherent room instead of floating stacked scenery over a black void.

### Step 3 — Make the whole room reachable on foot
- [x] Action: Update the dimensional collision contract so the continuous floor reaches the
  arcade wall, while retaining the wall clamp and existing object placement behavior.
- Files touched: `crates/engine/src/modes/inner_chambers/castle.rs`.
- Expected outcome: The player can sprint from the altar to the arcade without flying.

### Step 4 — Prove and install
- [x] Action: Add focused source/geometry tests, run `cargo test --workspace`, rebuild and stage
  with `scripts/install_shortcut.ps1`, then inspect the installed Taskbar build.
- Files touched: this plan and `STATUS.md`.
- Expected outcome: Tests, source, installed build, visible witness and `origin/main` agree.

## Execution record

- `cargo check -p engine`: passed.
- Focused castle tests: 43 passed.
- Focused world tests before the final contract addition: 9 passed.
- Final `cargo test --workspace`: 285 engine + 19 launcher + 5 Windows identity = 309 passed.
- `pwsh -File scripts\install_shortcut.ps1`: completed. Installed engine SHA256
  `FAA16E8AD1A6B2152408F53D90DBF66413F9628E68B12505731B324A1D61C95C`; pinned Taskbar target
  verified as `%LOCALAPPDATA%\Programs\Archetypes\launcher.exe`.
- Relaunched the installed product, entered Inner Chambers, waited for GLB/texture loading, and
  inspected the altar, close floor, arcade and vault. The former overlapping tile/triangle
  patterns and text-like bleed are absent. The replacement is intentionally sparse and stable;
  final environment detailing remains future art-direction work.
