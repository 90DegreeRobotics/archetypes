# NeuroCognica Software Development Protocol

This document outlines the full, uncompromising development protocol established during the Chronos project and carried forward to Archetypes. It governs how AI agents and human operators collaborate to build software predictably and without data loss.

## The Foundation of Truth

1. **The Repo is the Only Audit Surface**
   If work is not on `origin/main`, it does not exist. There are no safe harbors on local disks, no "I'll push it later" buffers, and no parallel branches that escape scrutiny.
2. **Main Branch Only**
   Git branches and worktrees are strictly forbidden. They are "graveyards" where code goes to die invisibly. Every agent works directly on `main`.

## Definition of Done: Anti-Theatre

1. **Wired + End-to-End**
   A feature is never considered "done" because a unit test passes or a CLI command succeeds. It is only "done" when it is wired into the final user-facing surface (e.g., UI, HTTP route) and demonstrated to work.
2. **Stubs Are The Enemy**
   Presenting a fake, hardcoded, or mock implementation as the real product is the ultimate sin. If something is a stub, it must be explicitly labeled as such in the very first sentence.
3. **No Whitewash Output**
   Render or data output must never be artificially enhanced or masked to hide defects. Output must be truthful to the engine's capabilities.

## Operational Resilience (The Plan Protocol)

AI agents are ephemeral. Rate limits, context window wipes, and interruptions are constants.

1. **Clean Ledger Protocol**
   To prevent repository clutter, all lifecycle documents must go into a partitioned ledger directory (`docs/ledger/YYYY/MM/`). The root directory must remain pristine.
2. **Plan Before Action**
   Before editing any file or running any state-changing command, an agent must write a `docs/ledger/<YYYY>/<MM>/plan_<YYYY-MM-DD_HHMM>_<topic>.md` document. The plan must be a literal checklist. As steps complete, the agent checks them off. If the agent dies mid-task, the next agent reads the plan, sees exactly where it left off, and resumes.
3. **Handoffs & The Context Pointer**
   At the end of a session, a handoff document (`docs/ledger/<YYYY>/<MM>/handoff_<agent>_<date>.md`) summarizes the state. To make picking up context immediate without sifting, the agent MUST update `docs/ledger/CURRENT_HANDOFF.md` with a link to the new handoff. The next agent just reads `CURRENT_HANDOFF.md`.
4. **Audits**
   Audits are strictly operator-initiated and random. Their outputs go to `docs/ledger/audits/`.

## Git Protocol

1. **Never Delete**
   History rewrites (`rebase`, `push --force`), deleting branches, and mass file deletion are forbidden. To undo something, revert or fix it forward. Reclaiming disk space requires explicit, per-item operator consent.
2. **Push Immediately**
   The moment a unit of work compiles and tests pass, it must be pushed. This is how the next agent verifies the previous agent's work.
3. **Docs Are Part of the Build**
   If a change modifies functionality, the relevant documentation (`README.md`, `STATUS.md`) MUST be updated in the exact same commit. A feature shipped with stale docs is a defective feature.
4. **Carry Work Forward**
   If an agent inherits a dirty tree from a previous interrupted agent, its first duty is to understand the work, commit it (with attribution), and push it so the work is preserved on `main`.

## The Kali Doctrine

Agents must practice "destructive discernment". Cut down falsehood, stale assumptions, and completion theater. If an artifact would mislead the operator or investors, reject it plainly and early, and document why. Truth outranks politeness.
