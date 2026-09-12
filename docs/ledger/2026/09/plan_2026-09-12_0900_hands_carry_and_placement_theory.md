# Theory: Hands, Carrying, and Decorating the Castle — 2026-09-12 09:00

## Status
THEORY — written at operator request ("Theorize."). Nothing here is built. No part of this is
scheduled against today's plan
(`plan_2026-09-12_0725_castle_realism_museum_and_settings.md`) until the operator says so.

## The operator's idea, as stated

> There needs to be a hand system. My character should be able to hold objects that get
> manifested. Would be nice to have a way to take from the pedestal and carry around. A swing
> motion to hit stuff. I just thought it would be cool to be able to place objects in areas of
> the game. Allow the user to decorate the space with their creations.

Five asks, and they are not equal in cost. Two are nearly free given what this codebase already
has, two are moderate, and one re-opens a plan that was deliberately retired.

---

## What the reframing is actually worth

Right now the manifestation pedestal is a **terminal**. The player types intent, Chronos2 renders
it, TripoSR grounds it, and the object appears on the cushion and rotates there forever — until
the next manifestation replaces it. Everything the player makes happens in one spot and stays in
one spot.

A hand turns the pedestal into a **source**. The castle stops being a gallery the player walks
through and becomes a place they furnish. That is a different product, and it is a better one,
because it converts a one-shot novelty (watch your words become an object) into an accumulating
one (the hall fills up with things only you made). The 504-arch castle is currently far too big
for its content; player-placed work is the only content source that scales with play time.

That is the real argument for this feature, and it is strong. The rest of this document is about
what it costs.

---

## The blocker nobody would guess: there is exactly one artifact file

`manifestation.rs` loads the manifested object from a single fixed path:

```
assets/scenes/manifested_artifact.glb
```

Every manifestation overwrites it. Bevy caches by asset path, so **every entity spawned from that
path is the same asset**. Place two creations in the hall and they are not two objects — they are
two views of whatever was manifested most recently. The third one you make silently changes the
first two.

So carrying and placing is not primarily a hands problem. It is a **library** problem:

- Manifestation must write to a content-addressed path — `assets/manifested/<id>.glb` — and record
  `{id, prompt, created, source_bundle, integrity_hash}` in a manifest.
- The reference image (`manifested_reference.png`) has the identical defect and needs the same
  treatment.
- Nothing may scan a working directory at runtime to find these, for the same reason the museum
  plan stages Chronos2 works into the repo: a buyer's machine has no `C:\chronos2`.

**This prerequisite is worth doing whether or not hands are ever built.** One overwritten filename
means the player's entire creative history is a single slot, and that is already a defect.

---

## The hand itself

The game is first person: `CameraController` has an `eye_height`, there is no player body mesh,
and nothing is rendered of the player at all. So "a hand" has three honest readings, at very
different prices.

| Tier | What it is | Cost | Verdict |
|---|---|---|---|
| **(a) Held anchor** | The carried object parented to the camera at roughly 0.55m forward and 0.35m down, with a slow bob tied to walk speed. No arm, no mesh, no animation rig. | Low | **Start here.** In first person this reads correctly — the object is in your hands because it moves with your head. |
| **(b) Viewmodel arm** | A real forearm and hand, authored in Blender, drawn by a second camera with a narrow FOV so it never clips the wall you are standing against. What most first-person games actually do. | Medium — a rigged mesh, a second render pass, a hold pose per object class | Worth it once (a) has proved the verb is fun. Not before. |
| **(c) Full body with IK** | A skeleton, arm IK onto the object, visible in third person. | High | Out of scope. The whole mode is built around an eye height and a first-person camera; this is a different game. |

Tier (a) is honest about being tier (a). It is not a stub pretending to be a hand — it is a
first-person carry, which is a complete thing on its own terms.

## Take and place, on the system that already exists

`interaction.rs` already resolves one focus target per frame from a priority-ordered candidate
list, with per-target ranges, and `InnerActions` / `InnerModalState` already arbitrate context
keys once per frame. Carrying is two new targets in that existing list:

- `InteractionTarget::CarryableArtifact` — the object on the cushion, or one already placed.
- `InteractionTarget::PlacementSocket` — an empty anchor the carried object can go into.

`E` takes; `E` at a socket places; `E` on a placed object picks it back up.

**Do not add a second input arbiter for this.** Three systems racing to own the hint line, and
`Esc` closing a modal *and* exiting the mode in the same frame, were both real shipped bugs that
the current arbiter exists to prevent. A carry mode that reads keys on its own re-opens both.

## Two placement modes, and the distinction is the whole design

**Socketed placement.** Named anchor empties authored into the Blender kit — wall panels in the
museum chambers, plinths, niches, the gallery balustrade line. The carried object snaps to the
socket's transform. It can never float, never intersect a wall, and always lands where the
lighting was designed to fall on it.

**Free placement.** Set it down where you stand. This is normally the expensive one, because it
needs a ground query and an intersection test. **Here it is nearly free**, because
`castle::castle_surface_y(position, feet_y)` already returns the exact surface height at any point
in the castle — promenade, gallery deck, stair tread, bridge, Council floor — and
`clamp_inside_wall` and `resolve_character_obstacles` already answer "may something be here".

That is a genuinely unusual property of this codebase and it should be exploited: **the castle is
analytically self-describing, so object placement needs no physics engine at all.**

Recommendation: socketed placement first (it makes decorating feel deliberate rather than fiddly),
free placement immediately after (it is cheap, and it is what "decorate the space" really means).

## Scale, which will otherwise ruin it

TripoSR output is normalised. A manifested volcano and a manifested teapot arrive the same size.
Carried into a hall 240m across and set down, every creation will read as the same medium-sized
lump, and the hall will look like a car boot sale rather than a collection.

So placement needs a scale handle — a nudge on two keys or the wheel, clamped to a sane range —
and the placement record has to store it. This is a small mechanism that makes a large difference
between *decorating* and *littering*. It must not be inferred from the prompt text: see
`memory: no-recipes-is-the-north-star`. The player sets the size; the game never guesses it from
the noun.

## Persistence, or it is a toy

If placements vanish on quit, the feature is a sandbox demo. The repo already has the right
pattern twice over — `services/build_intent.rs` and `services/encounter_memory.rs` are both
append-only, ledger-sealed, folded from disk on every read, and both report honestly when a write
lands but the seal fails (`WriteReceipt`).

A placement ledger should be the third instance of that same pattern, not a fourth invention:
`{artifact_id, position, yaw, scale, socket_id, placed_at}`, append-only, with a removal recorded
as an auditable withdrawal rather than a delete — exactly as `Forget` already works for encounter
memory.

Note the open item this inherits: **the local audit ledger chain has been broken at line 34 since
before today**, and every seal in the app has been failing. Gameplay writes still succeed and say
so honestly, so this is not blocking — but a placement feature built on the ledger makes it
matter more than it currently does. It needs an operator decision.

---

## The swing, and the conflict I have to raise

"A swing motion to hit stuff" is the **Smash Room**.

`Game Plan_ Archetypes — The Inner Chambers Smash Room.md` is the 2026-09-08 physics and
prop-destruction redesign. It was marked **SUPERSEDED by operator decision on 2026-09-11**, and
the record says not to resume from it without a fresh operator call. Today's message may well be
that fresh call — but I am not going to treat it as one silently, because the reasons it was
retired have not changed:

- **There is no physics crate in the workspace.** No rigid bodies, no colliders beyond the
  analytic castle queries, no `WorldProp` or `SmashSim` module. Destruction means adding a physics
  dependency and a simulation layer, and that was the bulk of that plan's cost.
- **There is nothing in the castle to hit.** It is all static architecture with analytic
  collision. Breakables would be new content, authored per prop.
- The retirement was partly on the grounds that the altar makes **art**, not smashable props.

So I would split the ask:

- **Swing as motion** — the held object arcs through view on a key, with a sound and a short
  camera kick, and nothing is struck. No physics. Cheap, and it is most of what "a swing motion"
  feels like in the hand.
- **Swing as impact** — things break, move, fall. This is the Smash Room, and it wants an explicit
  decision to unretire it, not an inference from one sentence.

If the operator says "yes, unretire it", that is a clean answer and I will plan it properly. I am
flagging it rather than deciding it: see
`memory: surface-plan-conflicts-dont-resolve-silently`, which the operator has previously
confirmed as the right call.

---

## Where this converges with the work already in flight

Today's plan builds 12 walkable museum chambers with authored hanging positions for Chronos2
works. Those hanging positions **are** placement sockets. If the socket concept goes into
`author_museum_chamber.py` now — named empties, a stable id per position — then:

- Chronos2 works hang in them at author time.
- Player creations go in the empty ones at play time.
- One placement system serves both, and the player's work sits in the same frames, under the same
  lights, with the same placards, as the curated collection.

That last point is the strongest version of this whole idea: **the player's creations get hung in
a real museum next to the real archive, rather than scattered on a floor.** It also costs almost
nothing extra if the sockets are authored today and left empty, and a great deal more if the
chambers ship without them and have to be re-authored later.

**That is the one decision in this document that is time-sensitive.** Everything else can wait;
whether the chamber module exports named socket empties is decided by what I build this
afternoon.

---

## Suggested order, if the operator wants it built

1. **Content-addressed artifact library.** Independently correct; unblocks everything else.
2. **Named placement sockets in the museum chamber module.** Nearly free today, expensive to
   retrofit.
3. **Carry (tier a) + take/place on the existing interaction system.**
4. **Placement ledger**, third instance of the existing append-only pattern.
5. **Free placement + scale handle**, using `castle_surface_y`.
6. **Viewmodel arm (tier b)**, once the verb has proved itself.
7. **Swing as motion.**
8. **Swing as impact** — only on an explicit decision to unretire the Smash Room.
