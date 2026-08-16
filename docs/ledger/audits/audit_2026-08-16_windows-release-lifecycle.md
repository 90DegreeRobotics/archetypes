# Audit: Windows release lifecycle, modes, and AURA council — 2026-08-16

**Initiated by:** operator  
**Scope:** full repository read (architecture, STATUS, Sentinel, scripts, launcher, all four GameMode lanes, parked council ritual, Windows metabolism)  
**Verdict:** **not release-ready.** The Desktop shortcut is a working developer stage, not a Windows product lifecycle. Two of four modes are playable. The AURA conversational council is implemented and real, but it is **not** the default Standard Mode the operator launches.

Related plan: `docs/ledger/2026/08/plan_2026-08-16_0710_windows-release-audit.md`

---

## 1. The finding that would have been missed by a surface read

`README.md` and `docs/architecture/COUNCIL_WORLD_ENGINE.md` still describe Archetypes as a council-driven ritual: Witness profile → portal offering → three-voice deliberation → speaking choreography → Witness verdict → Chronos artifact return.

The **Desktop product** does something else.

- Main menu `STANDARD MODE` inserts `TriggerStandardMecha` (`crates/engine/src/chamber/boot.rs`).
- That path is 1:1 Mecha chat: Seed-of-Life selector → single archetype → Ollama reply → Chronos/Comfy image (`crates/engine/src/modes/standard_mecha/mod.rs`).
- The seven-voice AURA council (`crates/engine/src/chamber/council.rs`, `speech.rs`, `ritual.rs`) still exists, still talks to real Ollama, still synthesizes verdict speech, and still requests Chronos artifacts.
- That ritual is **not reachable from the menu.** It is capture/legacy-env only (`ARCHETYPES_CAPTURE`, `ARCHETYPES_LEGACY_CHAMBER`).

If the release goal is “a game with multiple modes that showcases the AURA conversational council,” shipping the current Desktop Standard Mode would showcase the wrong system. Mecha chat is a real AURA *persona* surface. It is not the AURA *council*.

`STATUS.md` already recorded this split on 2026-07-15. `README.md` did not catch up. That doc drift is itself a release defect.

---

## 2. Windows lifecycle vs contract

### Documented contract

`docs/architecture/WINDOWS_METABOLISM.md` and `AGENTS.md` rule 12 require:

| Contract | Live truth |
| --- | --- |
| `%ProgramFiles%\Archetypes\archetypes_launcher.exe` | `C:\archetypes\dist\launcher.exe` |
| `%ProgramFiles%\Archetypes\archetypes_engine.exe` | `C:\archetypes\dist\engine.exe` |
| Versioned installer / update lane | **Absent.** Staging is `scripts\install_shortcut.ps1` |
| `%ProgramFiles%\Archetypes\scripts\` | Never staged |
| Mutable data under `%LOCALAPPDATA%\NeuroCognica\Archetypes` | **Honored** (data, logs, Sentinel seed, audio cache, Mecha history) |
| Dependencies declared in `scripts/dependencies.json` | Ollama + `qwen2.5:7b-instruct` + sherpa-onnx + Kokoro only |
| Chronos Foundry + ComfyUI | **Undeclared sidecars.** Launcher hard-gates them. August 4 launch refusal was Chronos `:7777` down, then Chronos policy 403 for `archetypes-launcher` |
| Uninstall / Add/Remove Programs | **None** |
| Help files (CHM / RTF / HTML / in-app Help) | **None** |
| Sidecar downloader UI | **None.** Bootstrap is PowerShell CLI |
| LICENSE | **None in repo** |
| Product version | `0.1.0` on both crates |

`AGENTS.md` rule 13 is honest about the operator surface: Desktop `Archetypes.lnk` → `C:\archetypes\dist\launcher.exe`. Rule 12 and Rule 13 currently contradict each other. Rule 13 is what actually runs.

`STATUS.md` and `README.md` already list “full Windows installer” as remaining work. This audit confirms that claim is still true, and that uninstall/help/sidecar-UI were never started.

### What does exist (and is real)

- Dual binary: `launcher.exe` supervises `engine.exe`.
- Desktop + Start Menu shortcuts with `archetypes.ico`.
- Release binaries are `windows_subsystem = "windows"` (no console flash), maximized window, embedded icon via winres.
- Single-instance guard on localhost:47615.
- TTS sidecar download with URL + SHA-256 in `dependencies.json`, extracted beside the install root (`setup_windows.ps1`).
- AppData isolation for mutable state.
- Sentinel launch gate: Chronos must be `readiness: ready` with authority `mode: enforce`; launcher signs a body-bound `archetypes_launch_requested` Codex append.
- Engine crash / Sentinel refusal → `%LOCALAPPDATA%\NeuroCognica\Archetypes\logs\last-failure.txt` opened in Notepad.

### Lifecycle gaps that will bite a non-developer install

1. **No installer.** There is no MSI, Inno, NSIS, or MSIX. No ARP entry. No versioned update. `dist/` is gitignored developer staging.
2. **No uninstaller.** Shortcuts, `dist\`, AppData, and speech trees are left forever unless an agent deletes them by hand.
3. **No help.** No Start Menu Help, no CHM, no in-game Help. Failures tell the operator to “re-run the Archetypes installer,” which does not exist.
4. **No sidecar UI.** `setup_windows.ps1` can install Ollama (winget), pull the model, and download TTS. It cannot install, start, or declare Chronos/Comfy. The Founder cannot be asked to run PowerShell; the product has no button for this.
5. **Release GUI fail-visible is incomplete.** Sentinel refusal and engine crash open Notepad. Missing Ollama / TTS / Chronos-after-Sentinel / duplicate instance only `eprintln!` + “Press Enter” — invisible when there is no console.
6. **Hardcoded operator machine path.** `services/chronos.rs` defaults Comfy output to `C:\Users\m\Documents\ComfyUI\output`. That is not a product path.
7. **Stale failure file.** Successful launches do not clear `last-failure.txt` (recorded 2026-08-04).
8. **Binary names** do not match the metabolism contract (`launcher.exe` / `engine.exe` vs `archetypes_*.exe`).
9. **Sentinel certification FAIL.** `docs/security/SENTINEL_CERTIFICATION_REPORT.md`: `adoption_readiness` fail. Protected actions for `system.install`, `installer.update`, `chat.respond`, `game.respond`, `memory.write`, identity, and artifact paths remain blocked. Launch gate is a foothold, not a certified product.

---

## 3. Modes of play

Registry (`crates/engine/src/modes/game_mode.rs`): four contracts. Two available, two locked without fake playability (honest).

| Mode | Menu | Implementation | Showcase value | Release state |
| --- | --- | --- | --- | --- |
| Standard | Playable | Mecha 1:1 chat with 7 personas, real Ollama, real Comfy, JSONL + ledger | AURA *characters*, not AURA *council* | Playable, identity-wrong vs canon |
| Oracle Riddle | Playable | Hidden 3-word prompt → Chronos image → order-insensitive scoring → Insight tiers → ledger | Machine as puzzle; uses Chronos | Playable core; competitive shells unbuilt |
| Inner Chambers | Locked | Parked prototype: black plane + 6 cubes, WASD camera, proximity E extracts fixed `Order/Structure/Grid` | Architect interior contract not met | Must not unlock as-is |
| Living Engine | Locked | **Zero files** under `modes/living_engine/` | Metabolic instrument; Viren entropy | Registry-only |

### Standard (default Desktop)

Real: selector, Cinzel/black-gold Seed-of-Life UI, per-archetype persona + art style, Ollama chat, Chronos concept-thumbnail, inline artifacts, scrollable transcript, phased wait copy, Esc back to lore menu, QUIT on main menu.

Missing vs council vertical slice: Witness onboarding, portal offering, multi-voice deliberation, speaking choreography, TTS, verdict collapse, artifact return to the table, chamber memory/consequence.

`ActiveGameMode` is defined and sometimes removed; it is **never inserted**. Ledger tags default to Standard even when they should not.

### Oracle Riddle

Real loop against Lane A core. Gaps vs `CODEX_LANE_A_ORACLE.md`: Daily/Speed/Hardcore/Infinite/Versus shells; no Inner Chambers → Oracle prompt seeding. Lane doc still says “WAITING ON LANE 0” (stale).

### Inner Chambers

Correctly locked after the 2026-07-15 safety pass (exit teardown, proximity extraction, no fake availability). Remaining work is the actual Architect world, not unlocking cubes. Unlocking now would despawn `AuthoritativeCouncilChamber` / `PortalTable` names that the default lore chamber does not use — another reason it is not production-safe.

### Living Engine

`CODEX_LANE_C_LIVING_ENGINE.md` is a full design (resonance, breaths, harmony score, 3-layer Aura, Viren infections, artifact implants). Theme registry already encodes Viren Ember Covenant. Gameplay code does not exist.

---

## 4. AURA conversational council — implemented, orphaned

The council stack is not a stub:

- Three constitutional voices per offering (framer / counter / deepener) so all seven rotate (`council.rs`).
- Fourth Ollama call collapses a Witness verdict.
- Kokoro/sherpa-onnx voices; verdict is fully synthesized; per-line TTS still uses signature voice + latency (STATUS blocker).
- Camera focus + environment tint (not a navigable interior).
- Chronos artifact on authorization.
- Hash-chained JSONL ledger.

Default launch loads `assets/scenes/lore_chamber.glb` as a **menu backdrop**. Legacy `uiscene1.glb` + `table.glb` + spheres/star/portal only appear with `ARCHETYPES_LEGACY_CHAMBER`.

Canon vertical slice (`COUNCIL_WORLD_ENGINE.md` 10 steps) is incomplete even on the ritual path: no distinct archetype interiors, no hexagram-alignment flight, no world memory/lineage, no chamber consequence after artifacts.

**Release implication:** restoring the council as a first-class playable mode (either as Standard, or as a fifth explicit “Council Chamber” entry with Mecha kept as a separate mode) is a product decision, not a polish pass. Leaving it parked means the shipped game does not demonstrate AURA as a polity.

---

## 5. Tighten-for-release list (non-mode)

1. Rewrite `README.md` to match Desktop truth, or restore the ritual to the menu so the README becomes true again. Do not ship both lies.
2. Route every launcher failure through `fail_visible` (Ollama, TTS, Chronos, duplicate instance).
3. Clear `last-failure.txt` on successful engine start.
4. Remove `C:\Users\m\Documents\ComfyUI\output` default; resolve via Chronos/Director or AppData.
5. Declare Chronos Foundry + Comfy in `dependencies.json` and give the launcher a **UI** repair/bootstrap path (not a script the Founder must run).
6. Inno/MSI (or equivalent) to `%ProgramFiles%\Archetypes`, ARP, Start Menu, Desktop icon, silent repair, uninstall that removes binaries/shortcuts and offers to keep AppData.
7. In-product Help (and/or Start Menu help) covering: what Archetypes is, how to start Ollama/Chronos, where logs live, how modes work.
8. LICENSE + third-party notices (Ollama, Kokoro, sherpa-onnx, Bevy, Chronos).
9. Product version beyond `0.1.0`; PE version resources in winres, not icon-only.
10. Refresh `docs/ledger/CURRENT_HANDOFF.md` (was still pointing at 2026-07-11).
11. Mark Lane A/B/C CODEX docs to current status so the next agent does not treat them as “waiting.”
12. Sentinel: either complete in-engine mediation to certification, or explicitly scope launch-gate-only for this product and stop claiming release certification.

---

## 6. Recommended finish sequence

This is the order that matches the operator’s stated aims (Windows lifecycle, finished modes, AURA council showcase). It is a recommendation, not work started in this unit.

1. **Product identity:** Put the AURA council ritual back on the main menu as a playable mode. Keep Mecha as a second playable mode if both are wanted. Do not call 1:1 chat “Standard” while canon says council.
2. **Windows lifecycle:** installer + uninstall + help + in-launcher sidecar bootstrap (Ollama/TTS/Chronos/Comfy) with fail-visible repair. Until that exists, “full Windows lifecycle” is false.
3. **Inner Chambers:** replace cubes with a real Architect interior, wire extracted truth into Oracle, then unlock.
4. **Living Engine:** implement from `CODEX_LANE_C_LIVING_ENGINE.md` as a simulation-first mode; Viren already has theme law.
5. **Oracle shells and council depth:** competitive Oracle modes; hexagram camera; one real archetype interior on the council path; world memory.

Do not unlock Inner Chambers or Living Engine as a labeling change. That would be a stub presented as the product.

---

## 7. What is already rock-solid (keep)

- Fail-closed Chronos/Sentinel launch (once Chronos is up and trusts `archetypes-launcher`).
- Honest locked-mode labels.
- Oracle scoring fairness (concrete triples, aliases, lexical fallback).
- Mecha chat persistence and visible service wait states.
- LocalAppData metabolism for user data.
- TTS pin-by-hash.
- Workspace Rust gate habit and Desktop restage rule (Rule 13) — for the *developer* surface, this is real.

Those are necessary. They are not sufficient for a Windows product release that showcases AURA as a council and ships four finished modes.
