# Plan: Inner Castle Grounded Capabilities — 2026-09-11 00:15

## Status
PENDING

## Goal

Build four useful, bounded archetype capabilities inside the Inner Castle: consentful
archetype memory, a cited Oracle archive, a resumable Mentor reading room, and an
Architect planning workshop. Each system begins from an explicit player action, stays
local-first, persists only what the player elects to keep, and leaves a readable proof
or a visible failure. This is not a plan to turn archetypes into generic chat skins.

## Governing execution contract

```text
explicit player intent
  -> local capture/transcription
  -> review
  -> bounded archetype capability
  -> sourced/persisted result
  -> visible proof or honest failure
```

No capability may skip a stage. Voice-derived content is always reviewable before use;
model output never becomes a source; a failed local service must leave the player with
their unsent input and an explanatory status.

## Shared foundation — encounter records, consent, and evidence

### Step 1 — Replace the current transcript-only encounter log with an owned journal model
- [ ] Action: Add a versioned local `EncounterRecord` format under
  `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\encounters\`, including unique ID,
  archetype, UTC timestamps, player-reviewed input, response, source references, capability,
  retention state, and hash/ledger receipt where appropriate.
- [ ] Action: Separate transient encounter turns from durable records. A conversation is not
  remembered merely because it was spoken or typed.
- [ ] Action: Add explicit `[Remember]`, `[Forget]`, and `[View record]` controls to the
  encounter overlay. Forget creates an auditable withdrawal/tombstone and removes the record
  from recall; it must not silently leave the content available to RAG or future prompts.
- Files: `crates/engine/src/modes/inner_chambers/encounters.rs`, new mode-neutral memory
  service, ledger/path tests, UI tests.
- Verification: Create, decline, remember, forget, restart, and prove recall behavior from
  LocalAppData records and a hash-chain witness. A deliberately bad STT transcript must remain
  transient unless the player explicitly approves it.

### Step 2 — Build one capability registry and physical-device contract
- [ ] Action: Define a typed `CastleCapability` registry: `OracleArchive`, `MentorReading`,
  `ArchitectWorkshop`, each with location entity, entry action, exit/return transform,
  persistence type, required source policy, and availability status.
- [ ] Action: Extend the existing focused-interaction resolver so an embodiment conversation
  and a nearby physical device are deterministic targets with honest prompts.
- [ ] Action: Keep standalone modes until a castle capability has independently proven parity;
  do not delete menus as a cosmetic “convergence” gesture.
- Verification: Unit-test focus priority and run a real route from castle floor → device →
  capability → explicit exit → original world transform.

## Oracle archive — sourced consultation, not invented wisdom

### Step 3 — Curate and ingest a permitted local corpus
- [ ] Action: Create a manifest-driven `assets/corpus/oracle/` intake contract. Every document
  must state title, creator/translator, license or public-domain basis, source URI/provenance,
  content hash, chunk policy, and import timestamp.
- [ ] Action: Begin only with clearly permitted/local texts; do not scrape a broad web corpus or
  cite model memory. Reuse Chirox's passage principle, not its Shaolin-specific corpus.
- [ ] Action: Build deterministic chunking and lexical retrieval first. Add embeddings only when
  their local model, index provenance, and regression set are specified and tested.
- Files: new Oracle archive service, corpus manifests, test fixtures, documentation.
- Verification: Every answer displays a passage, title, locator/chunk ID, content hash, and
  source provenance. Tests must reject a quotation not verbatim present in a source passage.

### Step 4 — Implement the Oracle observatory consultation loop
- [ ] Action: Add an observatory archive device with typed/voice-reviewed theme request,
  retrieved passages, optional constrained local interpretation, `Bookmark`, `Remember`, and
  `Read aloud` controls.
- [ ] Action: Make the model prompt distinguish quotation from interpretation; it may explain a
  retrieved passage but may never invent attribution or present uncited prose as a source.
- [ ] Action: Store bookmarks as player-local records with exact source/chunk/hash rather than
  a copied, decontextualized quote.
- Verification: Ask one known theme, inspect source, bookmark, restart, reopen bookmark, and
  prove the same source hash. Simulate missing corpus/Ollama and verify honest failure.

## Mentor reading room — resumable, interruptible, and deaf while speaking

### Step 5 — Build the permitted reading catalog and bookmark model
- [ ] Action: Define a local reading-catalog manifest for approved docs/books, source rights,
  display title, spoken aliases, chunking policy, and archetype-room placement.
- [ ] Action: Implement title/alias matching with an ambiguity prompt—never silently choose the
  wrong book from a fuzzy match.
- [ ] Action: Persist per-player reading progress by stable content hash + chunk index. If a text
  changes, show it as a new edition and preserve the prior bookmark separately.
- Verification: Start, stop, resume, alter a fixture source, and prove old/new bookmark behavior.

### Step 6 — Add narrated reading with speech arbitration
- [ ] Action: Route chunks through the established local TTS service and add Pause, Resume,
  Stop, Next, Previous, and Return-to-Castle controls.
- [ ] Action: Introduce a single voice-arbitration resource: while narration or an archetype
  reply is speaking, STT capture is disabled/clearly shown as paused; active capture cancels or
  waits before narration begins. Do not let the microphone transcribe the game itself.
- [ ] Action: Keep long-text generation off the render thread and prefetch only a bounded number
  of chunks to avoid runaway memory/use.
- Verification: Real speakers/headphones test plus deterministic fake audio test: narration
  pauses STT, stop releases the microphone, bookmark resumes at the intended chunk, and no
  duplicate audio entities survive mode exit.

## Architect workshop — inspectable planning with agency retained

### Step 7 — Define the plan artifact before adding an LLM prompt
- [ ] Action: Define a player-owned local `BuildIntent` artifact: title, intent statement,
  constraints, chosen next actions, risks/open questions, supporting encounter/source IDs,
  status, timestamps, and optional completion evidence.
- [ ] Action: Build a workshop table/device that creates, edits, sequences, and closes these
  artifacts. The player confirms every persisted action; the local model may propose but cannot
  commit a plan for them.
- [ ] Action: Base “under-practiced area” recommendations only on measurable, player-approved
  signals—open plans, stalled action age, explicit priorities—not invented psychometrics.
- Verification: Create a plan, reject a suggestion, accept/edit one action, close, restart,
  reopen, and verify all provenance. Ensure no remote write or external task is created merely
  by an archetype response.

### Step 8 — Render the workshop as a physical castle system
- [ ] Action: Give Architect a real workshop object: blueprint table/shelves/notice board with
  a clearly readable entry affordance, not a menu teleported onto a room.
- [ ] Action: Connect its exit to the exact castle transform, and display open-plan state through
  grounded world details only after functional workflow proves useful.
- Verification: installed-build walkthrough and screenshots from approach through artifact save
  and return; visual details are not accepted as proof of the underlying workflow.

## Voice and safety dependencies

### Step 9 — Land voice input only as reviewed local input
- [ ] Action: Reuse the proven Shaolin pattern (`sounddevice` capture + local faster-whisper),
  but package an Archetypes-owned runtime/model and dependency declaration. Never invoke
  `C:\Shaolin` at product runtime.
- [ ] Action: Bind hold-to-talk only inside an active encounter/capability prompt. Insert the
  transcript into the draft for review; require explicit send. Store temporary audio solely under
  LocalAppData and delete it after transcription unless the player enabled diagnostics.
- [ ] Action: Wire existing local Kokoro/sherpa TTS to encounter/archive/reading outputs through
  the voice arbiter and user volume settings.
- Verification: live microphone witness covering speak → review → edit → send → local reply →
  spoken response; prove no background capture and no automatic send.

## Delivery order and gates

1. Shared journal/consent and capability registry.
2. Oracle archive with a tiny, licensed fixture corpus and source proof.
3. Mentor catalog/narration/bookmarks plus voice arbitration.
4. Architect plan artifacts/workshop.
5. Local STT packaging and live witness across the three rooms.

Each numbered delivery is a separate commit/push only after `cargo test --workspace`,
installed Desktop/Taskbar refresh, and a capability-appropriate real witness. Do not start
the next room while the previous room is a visually attractive stub.
