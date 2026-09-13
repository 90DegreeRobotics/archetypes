# Installed-game American-football witness — disambiguated prompt

## Verdict

**Gameplay pickup: PASS. Object quality: FAIL.**

Because the exact prompt `football` produced a hybrid soccer-ball reference, the installed game was run again with `American football`. The new artifact completed the real pipeline, was taken from the altar through the normal `E` interaction, and produced both a populated carried-object resource and one camera-child `CarriedVisual`.

The reference correctly depicts an American football, but single-view reconstruction failed badly: the held object is an open, over-thick shell rather than a closed football. Chronos measured `46.9%` invented outline, set `matches=false`, and explained that too few views left material no view refused. Despite that explicit failure, the game still accepted and staged the artifact. That acceptance gap is a blocking Manifester quality defect.

## Identity and measurements

- Artifact id: `838b4a2b-3acc-4c89-8fde-d23586280881`
- Installed asset: `assets/manifested/838b4a2b-3acc-4c89-8fde-d23586280881.glb`
- Installed asset SHA-256: `48FEF7F57FD5E17FA53D242CF1C6F3033B58CDA03BFA570F09FEFA782E49A381`
- Held screenshot SHA-256: `BA7AD21FF16E2E4B6301298D5AED5067B0ACCDF58F47800FE31CCF333CE63573`
- Subject-match score: `0.502417266368866`
- Reference coverage: `0.9036789536476135`
- Invented outline fraction: `0.4691551923751831`
- Subject match: `false`
- Imported triangle count: `74,999` (from `111,312`)
- End-to-end pickup observed: `372.1s`

`01_object_held_in_hand.png` proves that the installed game carries the generated artifact. It does not prove acceptable generation quality. The reference and five reconstruction angles are retained beside it to make the failure unambiguous.
