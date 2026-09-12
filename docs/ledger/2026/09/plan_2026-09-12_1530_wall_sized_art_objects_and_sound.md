# Plan: Wall-Sized Art, Player-Owned Objects, and Sound — 2026-09-12 15:30

## Status
COMPLETED — 2026-09-12. All three decisions built: wall-sized per-work frames, authored sound
effects with a real bus, and the object library with carry / place / duplicate. The swing is
built as motion only; the Smash Room stays retired.

## Operator directive

> The paintings are tiny. When they get rendered in chronos2 they are wall sized. You should
> have taken more of the logic. You made some weird shit. All objects need to be individual
> objects that can be duplicated by the player. If a player wants to fill the room with
> duplicates that's fine. I want the objects to be able to be placed. I want free sound effects
> added. You need to make some decision stop passing the buck. You are the builder. You read the
> plans and make them real. Don't be lazy.

**Decisions are made in this document, not deferred.** Where the earlier work asked the operator
a question, this plan answers it and builds the answer.

---

## What I got wrong, precisely

I read `gallery_exhibit.rs` for two things — `title_from_prompt` and the `museum_wall`
exclusion — and invented the rest. The rest is the part that mattered.

`build_gallery_hall_script` sizes **every frame from its own image**:

```python
im = bpy.data.images.load(img_path); aw, ah = im.size
ar = (aw/ah) if ah else 1.46
AH = 0.92; AW = AH*ar
fr.dimensions = (AW+0.06, 0.055, AH+0.06)
```

The height is fixed, the width follows the artwork, and the frame is built around whatever comes
out. It also lights each work with its own spot (`energy=180`, warm) and shades the canvas with
an **emission** shader so the painting reads in a dim hall.

I did none of that. I forced every work onto a 4:3 mat so one fixed GLB frame would fit them all,
which is backwards: the frame should follow the art. That is the "weird shit" — a mat nobody
asked for, invented to protect an asset that should not have been fixed-size in the first place.

And the size: a 1.6m painting on a 7.7m-wide, 9.4m-high chamber wall is a postage stamp.

## Decision 1 — Frames are built per work, in the engine, and they are wall-sized

- `assets/scenes/art_frame.glb` is **retired**. A frame cannot be one asset when every work has
  a different aspect; chronos2 builds it per work and so will this.
- Frame geometry is built in Bevy per exhibit: four moulding bars, a canvas, a placard plate.
- Sizing ports chronos2's rule — fixed height, width from the image's own aspect — scaled to a
  chamber wall instead of a 4.6m studio wall: **fit within 5.0m x 2.9m**, height first, width
  clamped.
- Frame colour takes chronos2's near-black `(0.02, 0.02, 0.022)` at roughness 0.45, not the gilt
  I invented. Placard plate `(0.90, 0.89, 0.86)`.
- Canvas gets an emissive term as chronos2 does, **plus** its own warm spot per work. A 9m recess
  is dark; a non-emissive painting in it is a grey rectangle.

**Consequence, decided rather than raised:** at 5.0m wide, two works no longer fit on a 9m side
wall 3.6m apart. **Three hangings per chamber** — one centred on each side wall, one on the back
wall. 36 works instead of 60. Fewer, bigger, and actually readable is the right trade for a room
this size.

## Decision 2 — Every manifested object is its own asset, and the player owns it

The blocker named in `plan_2026-09-12_0900_hands_carry_and_placement_theory.md`: manifestation
writes one fixed path, `assets/scenes/manifested_artifact.glb`. Bevy caches by path, so every
entity spawned from it is the same asset. Two creations in the world are not two objects.

- Manifestation writes **`<data>/manifested/<id>.glb`** and appends to a library manifest.
- The altar's object becomes a **takeable** thing: `E` picks it up, and the altar is free to
  make another.
- **Carry** is a held anchor parented to the camera — first person, no arm mesh. Tier (a) from
  the theory doc, chosen and built rather than offered.
- **Place** sets it down on the floor the player is looking at, using `castle_surface_y`, which
  already answers the ground height everywhere in the castle. No physics engine needed.
- **Duplicate** drops a copy and keeps the original in hand. The operator asked for this
  explicitly: filling a room with copies is allowed.
- Placements persist through an append-only ledger, the third instance of the pattern
  `build_intent.rs` and `encounter_memory.rs` already use.

## Decision 3 — Sound effects are authored, not sourced

"Free" is answered by **owning them**: a generator writes original WAVs from first principles
(noise shaping, decaying sines, filtered transients). No licence to check, no attribution to
carry, nothing to remove later. Same reasoning as the stone tiles, per
`memory: build-it-dont-wait-for-a-clean-license`.

- `scripts/author_sound_effects.py` writes `assets/audio/sfx/*.wav`.
- A real SFX bus in the engine, so the settings slider that currently says "no SFX bus yet"
  stops saying it.
- Cues: footstep (varied), jump, land, pick up, place, duplicate, altar charge, manifestation
  success, failure, menu move, menu adjust, menu confirm.

## Decision 4 — The questions I previously handed back

- **Stone progression by level:** adopted as proposed. Dark basalt at the base through to pale
  limestone at the head. It is how masonry is built and how height reads; if the operator wants
  it different they will say so, and it is a one-line change.
- **The six archetype conversations, dark since the rooms were hidden:** they move to the museum
  chambers. Twelve chambers, six archetypes — the conversation belongs where the player already
  walks. Not in this unit, but decided and written down so the next one does not re-ask.
- **The swing:** built as motion, not impact. The held object arcs on a key with a sound and a
  camera kick. Nothing is struck, no physics crate is added, and the retired Smash Room stays
  retired until the operator says otherwise — that is a decision about scope, not a question.

## Order

1. Wall-sized frames built per work. (The direct complaint.)
2. Sound effects and the SFX bus. (Self-contained, explicitly asked for.)
3. The object library, carry, place, duplicate. (The largest, and the one that changes the game.)

## Verification

Unchanged from the day's standing bar: `cargo test --workspace` must not fall, a **walking**
capture for anything touching movement or interaction, frames under
`artifacts/visual-proof/<topic>-2026-09-12/`, and mechanical facts reported rather than
aesthetic self-approval.
