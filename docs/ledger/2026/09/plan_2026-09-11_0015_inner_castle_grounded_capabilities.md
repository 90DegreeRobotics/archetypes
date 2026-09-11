# Plan: Inner Castle Grounded Capabilities — 2026-09-11 00:15

## Status
IN-PROGRESS (Step 1's journal/consent model landed 2026-09-11; Steps 2-9 remain PENDING)

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
- [x] Action: Add a versioned local `EncounterRecord` format under
  `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\encounters\`, including unique ID,
  archetype, UTC timestamps, player-reviewed input, response, source references, capability,
  retention state, and hash/ledger receipt where appropriate.
- [x] Action: Separate transient encounter turns from durable records. A conversation is not
  remembered merely because it was spoken or typed.
- [x] Action: Add explicit `[Remember]`, `[Forget]`, and `[View record]` controls to the
  encounter overlay. Forget creates an auditable withdrawal/tombstone and removes the record
  from recall; it must not silently leave the content available to RAG or future prompts.
- Landed: `services/encounter_memory.rs` — an append-only `journal.jsonl` under
  `data/encounters/`, mirroring the ledger's tamper-evident shape: state is never mutated in
  place, only superseded by a later `Created`/`Remembered`/`Forgotten` event. Recall is
  recomputed by folding the whole file every call (`recallable_records`), so there is no
  in-memory cache that could go stale across a restart — a record is recall-eligible only if
  its latest transition is `Remembered`, and a later `Forgotten` always wins over an earlier
  `Remembered`. Each event is also sealed into the existing hash-chained `services::ledger`
  (kinds `inner_castle_encounter_created`/`_remembered`/`_forgotten`) for tamper evidence, and
  each record carries its own local SHA-256 `content_hash` receipt independent of the ledger.
  `modes/inner_chambers/encounters.rs` now opens one `EncounterRecord` per completed turn as
  `Transient`, and binds `F5` Remember / `F6` Forget / `F7` View record (chosen instead of
  bare letter keys because the typed-draft system already consumes every A-Z keystroke into
  the message box) — F7 renders the record's id, retention state, player line, and archetype
  reply straight from disk. The pre-existing raw `conversation_history/inner_castle.jsonl`
  transcript is untouched and still writes every line unconditionally; it is a debug/support
  log only, nothing reads it back as memory, and any future recall/RAG consumer must read
  exclusively from `recallable_records`, never that file.
- Files: `crates/engine/src/services/encounter_memory.rs` (new),
  `crates/engine/src/modes/inner_chambers/encounters.rs`.
- Verification: `cargo build -p engine` and `cargo test --workspace` pass (114 engine + 19
  launcher + 5 windows_identity; 6 new tests directly prove create→decline stays unrecallable,
  create→remember survives a fresh fold-from-disk "restart", create→remember→forget is
  permanently excluded from recall while the withdrawal itself stays inspectable via
  `view_record`, per-archetype filtering, and content-hash tamper sensitivity). Tests run
  against isolated scratch files rather than the shared `ledger.jsonl`/journal, since the
  ledger is a single process-wide hash chain and parallel test threads writing to it directly
  raced and corrupted the chain on the first attempt — the fix separates the journal write
  from the ledger-sealing call so only the real (non-test) code path touches the shared
  ledger. `scripts\install_shortcut.ps1` rebuilt release and restaged Desktop/Start
  Menu/Taskbar with SHA-256-verified binaries; a real `ARCHETYPES_INNER_CAPTURE=1` run of the
  installed engine produced 8 fresh frames under
  `artifacts/visual-proof/encounter-memory-2026-09-11/` proving Inner Chambers still boots and
  renders cleanly with the new plugin wiring. **Not yet done:** a live conversational
  walkthrough (approach an archetype, type a message, wait for a real local Ollama/TTS reply,
  press F5/F6/F7, and inspect the real `journal.jsonl`) has not been performed — that needs an
  actual round-trip through Ollama and the TTS worker, which no capture-mode run drives. This
  is an honest verification gap, not a claimed pass.

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
