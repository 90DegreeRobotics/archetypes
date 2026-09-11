# Manus prompt — CRM OpenRAIL provenance follow-up

Drafted 2026-09-11 as a narrow follow-up to
`C:\chronos2\docs\plans\plan_2026-09-07_1530_quality-engine-licence.md`. Not a
new sweep — Manus already screened CRM out once in the Q7 alternatives pass;
this goes deep on the specific mechanism found this session.

## Prompt sent to Manus

Continuing the licence work in `chronos2/docs/plans/plan_2026-09-07_1530_quality-engine-licence.md` — same methodology as your TripoSR and Hunyuan3D research there: primary sources only, commit-pinned URLs, quote the exact clause, answer plainly. This is a narrow follow-up, not a new sweep — you already screened CRM out in the Q7 alternatives pass; this goes deep on the specific reason.

**New finding to verify, not take on faith:** CRM's (`thu-ml/CRM`) own training configs show both of its diffusion checkpoints resume from an external checkpoint:
- `configs/nf7_v3_SNR_rd_size_stroke_train.yaml` → `resume: release_models/ImageDream/sd-v2.1-base-4view-ipmv.pt` (trains `pixel-diffusion.pth`)
- `configs/stage2-v2-snr_train.yaml` → same resume path (trains `ccm-diffusion.pth`)

That checkpoint is ByteDance's ImageDream, itself a 4-view fine-tune of Stability's `stable-diffusion-2-1-base`, which carries CreativeML Open RAIL++-M. CRM's own GitHub `LICENSE` and HF weights are tagged plain MIT with no disclaimer splitting code from these two checkpoints.

**Q1 — Verify the chain at primary sources.** Confirm at commit-pinned URLs: (a) the two `resume:` paths above actually exist in CRM's repo history, (b) `sd-v2.1-base-4view-ipmv.pt` is really ImageDream's SD2.1-base fine-tune, (c) SD2.1-base's actual license file. If this doesn't hold up, say so and stop — no need to answer Q2-Q4.

**Q2 — Does the operator-install posture that cleared TripoSR also clear this?** TripoSR's answer was "MIT imposes no obligation at all on a local env-var invocation — nothing to be a licensee of until something is copied." Open RAIL++-M requires giving "any Third Party recipients of the Model or Derivatives of the Model a copy of this License." If NeuroCognica never distributes `pixel-diffusion.pth`/`ccm-diffusion.pth` themselves — operator downloads it directly from the publisher, product only shells out to a local subprocess — is there still a "Third Party recipient" for RAIL purposes, or does the same "nothing copied, nothing to be a licensee of" logic apply here too?

**Q3 — Does the restriction reach the output?** Hunyuan3D's territory clause reached the *output*, not just the model — that's what killed it. TripoSR had no output restriction at all. Does Open RAIL++-M's Attachment A (prohibited uses) restrict what can be done with a *generated mesh*, or only the model itself?

**Q4 — Is CRM.pth (the reconstruction network) actually clean and separable?** Is CRM's own reconstruction stage trained from scratch (not resumed from anything upstream), and is it architecturally usable on multiview images from a *different, unencumbered* source — i.e., is the OpenRAIL taint confined to the two diffusion checkpoints, or does it reach the whole pipeline?

**Format:** Short, like your TripoSR amendment. Plain verdict per question, one-line citation per claim (URL + what it says), no summary padding. If Q1 fails, one paragraph and stop.

## Context this session already established (for whoever reads Manus's answer)

- InstantMesh: Apache-2.0 code, but requires Zero123++ (SUDO-AI) whose weights
  are CC-BY-NC-4.0 — flat non-commercial, no workaround. Dead end.
- Stable Zero123 (Stability): non-commercial-only free tier; commercial
  variant requires a paid Stability AI membership. Dead end for a one-time
  local install.
- CRM: the only candidate not flatly dead, contingent entirely on how Q2-Q4
  above come back.
