# Mecha Frontend Full Audit for Standard Mode Rewrite - 2026-07-15

## Status
COMPLETE source-canon audit. No game code has been changed in this unit.

## Operator Directive
`C:\mecha\aura-mechanician\frontend\src` is not optional inspiration. It is preserved source canon for the next Archetypes Standard Mode rewrite.

The rewrite target is:

- Archetypes lets the player chat with archetypes directly.
- When the player chats with an archetype, the game behaves like the Mecha frontend.
- When the player chats with an archetype, the game visually reads like the Mecha frontend.
- The current/old Standard Mode is preserved for reference; it is not silently deleted or overwritten.
- Codex does not decide how much of the Mecha folder matters. The full folder was inventoried and classified here so the operator can decide scope explicitly.

## Source Examined
Root: `C:\mecha\aura-mechanician\frontend\src`

Mechanical checks run:

- `rg --files C:\mecha\aura-mechanician\frontend\src`
- file sizes, line counts, and SHA-256 prefixes for every file
- PNG dimensions for every image asset
- endpoint/function/theme scans excluding `three.min.js`
- direct reads of `index.html`, `mindplane.js`, `main.js`, `utils\ThemeManager.js`, `utils\AssetLoader.js`, `components\MindplaneContainer.tsx`, `components\MindplaneOverlay.tsx`, and `components\ProvenanceViewer.tsx`
- current Archetypes ownership check in `crates\engine\src\chamber\*`, `crates\engine\src\modes\*`, `crates\engine\src\services\llm.rs`, and `crates\engine\src\services\ledger.rs`

## Inventory

36 files total: 6 root/source files, 3 React component files, 2 utility files, 7 theme files, 19 PNG assets.

| Relative path | Bytes | Lines | SHA-256 prefix | Classification |
| --- | ---: | ---: | --- | --- |
| `index.html` | 59062 | 1318 | `8EEA9E99BA4F1B9A` | live Electron UI shell, splash, chat, modals, monitor |
| `mindplane.js` | 24634 | 606 | `9683934DAD8E63C1` | live consciousness selector |
| `main.js` | 1081 | 42 | `EE100D144EFF4539` | Electron main process |
| `three.min.js` | 668024 | 7 | `7B1C5D75B28D9DE1` | bundled Three.js r159 vendor runtime |
| `utils\ThemeManager.js` | 3104 | 93 | `DD6A25A598BEF97B` | dynamic archetype theme loader |
| `utils\AssetLoader.js` | 4092 | 138 | `DC19E35721E820D8` | asset path/cache helper |
| `components\MindplaneContainer.tsx` | 1369 | 53 | `5CDA33B921BCFB24` | React mindplane container reference |
| `components\MindplaneOverlay.tsx` | 14301 | 402 | `33079EF05DD78374` | React mindplane overlay reference, currently syntactically damaged |
| `components\ProvenanceViewer.tsx` | 4392 | 105 | `EC7E0B2698730EBF` | provenance DAG viewer reference |
| `themes\architect.css` | 3133 | 112 | `FF9FD3A009CC53A2` | Architect visual signature |
| `themes\empath.css` | 4004 | 159 | `8928B18CA526F89F` | Empath visual signature |
| `themes\explorer.css` | 3801 | 147 | `99FF0802ABCD9BC3` | Explorer visual signature |
| `themes\jester.css` | 5438 | 200 | `08A39290E911EDBB` | Jester visual signature |
| `themes\mechanician.css` | 2354 | 93 | `F7E51F2BF60F091E` | base/mechanician visual signature |
| `themes\mentor.css` | 3389 | 132 | `6E2C3044B76CD0FC` | Mentor visual signature |
| `themes\oracle.css` | 4238 | 152 | `5EBF19FB0FC1FD35` | Oracle visual signature |
| `themes\sentinel.css` | 3448 | 131 | `CB5E63AF6D17A4DE` | Sentinel visual signature |

### Asset Inventory

| Asset | Dimensions | Bytes | Use |
| --- | ---: | ---: | --- |
| `assets\architect-icon.png` | 1024x1024 | 1064677 | selector/dropdown icon |
| `assets\architect.png` | 1024x1536 | 2874405 | chat portrait |
| `assets\council000.png` | 1920x1080 | 203604 | council geometry reference |
| `assets\council001.png` | 1920x1080 | 922491 | council geometry/reference image with portraits |
| `assets\council002.png` | 1920x1080 | 848960 | neon council geometry/reference image |
| `assets\empath-icon.png` | 1024x1024 | 1488824 | selector/dropdown icon |
| `assets\empath.png` | 1024x1536 | 2879992 | chat portrait |
| `assets\explorer-icon.png` | 1024x1024 | 518251 | selector/dropdown icon |
| `assets\explorer.png` | 1024x1024 | 2060145 | chat portrait |
| `assets\jester-icon.png` | 1024x1024 | 1439971 | selector/dropdown icon |
| `assets\jester.png` | 1024x1536 | 2848890 | chat portrait |
| `assets\logo.png` | 1024x1024 | 1309795 | splash/watermark |
| `assets\mentor-icon.png` | 1024x1024 | 1081741 | selector/dropdown icon |
| `assets\mentor.png` | 1024x1536 | 2761399 | chat portrait |
| `assets\oracle-icon.png` | 1024x1024 | 1209844 | selector/dropdown icon |
| `assets\oracle.png` | 1024x1536 | 2891771 | chat portrait |
| `assets\sentinel-icon.png` | 1024x1024 | 1186513 | selector/dropdown icon |
| `assets\sentinel.png` | 1024x1024 | 2070940 | chat portrait |
| `assets\uxbacklayer.png` | 2816x1536 | 6461296 | primary consciousness selector background |

## What The Mecha Frontend Actually Does

### `main.js`
Electron shell:

- opens a 1400x900 fullscreen BrowserWindow
- enables Node integration and disables context isolation
- loads `index.html`
- forces DevTools open
- uses `#0a0a0b` as the shell background
- titles the window `AURA - Mechanician Core`

This should not be ported as Electron. In Archetypes, it becomes a native Bevy Standard Mode surface.

### `index.html`
This is the live chat UI shell, not just static markup.

Player-facing surfaces:

- splash screen using `assets\logo.png`
- logo watermark
- left chat panel
- top archetype badge/dropdown
- About menu
- Council button returning to the consciousness selector
- chat transcript area
- text input with Enter send and Shift+Enter newline
- right portrait panel
- active archetype title/subtitle/element display
- system health panel
- live system monitor panel
- build log/status panel
- document modals for AURA, Man-Machine Alliance, AI Bill of Rights, and Credits

Behavior:

- defines `ARCHETYPE_INFO` for the seven archetypes
- uses `API_URL = http://localhost:8000`
- `switchArchetype(archetype)` changes theme, portrait, title/subtitle/element, badge, dropdown state, and chat history
- preloads logo, seven portraits, seven icons, and three council images
- loads chat history with `GET /chat/history?archetype=<id>&limit=100`
- sends messages with `POST /chat` body `{ content, archetype }`
- polls `GET /system/health` every 10 seconds for basic health
- polls `GET /system/health` every 2 seconds for GPU/VRAM/RAM/CPU monitor
- opens local docs via Node `fs` reads
- waits for splash completion, then shows the consciousness selector

### `mindplane.js`
This is the live consciousness selector.

Player-facing surfaces:

- full-screen `uxbacklayer.png`
- dark readability overlay
- mouse parallax
- QSIC pulse at canvas coordinate 150,150
- seven archetype nodes placed against a 2816x1536 coordinate system
- image icons from `assets\<archetype>-icon.png`
- hover glow and hover cards
- hidden Three.js platonic solid per archetype, shown on hover
- SVG connection lines from Sentinel to the other six nodes
- Ctrl+Space returns to selector
- Escape toggles selector/chat visibility

Behavior:

- saves selected archetype to `localStorage`
- fetches `GET /archetypes/<id>` for hover card data
- fetches `GET /system/health` for QSIC/health pulse
- click selects an archetype, delays 800 ms, hides selector, initializes ThemeManager, and switches chat to that archetype

### `utils\ThemeManager.js`
Dynamic theme system:

- validates archetype id against the seven-member list
- creates `link#dynamic-theme`
- sets `themes/<archetype>.css`
- sets body class `theme-<archetype>`
- dispatches `themeChanged` event
- exposes `getCurrentArchetype()`
- fetches `GET /archetypes/<id>` metadata
- prefetches all seven theme files

### `utils\AssetLoader.js`
Asset path/cache helper:

- resolves local Electron `file://` paths
- returns portrait, icon, logo, and council geometry paths
- preloads archetype portraits/icons
- preloads logo
- checks portrait/icon availability

### React Components
These are reference surfaces, not the live Electron path.

- `MindplaneContainer.tsx` reproduces the `uxbacklayer.png` full-screen background with a dark overlay and React `MindplaneOverlay`.
- `MindplaneOverlay.tsx` contains the same archetype positions, Three.js/react-three-fiber platonic solids, system health polling, provenance integration, and AURA online status polling. It is currently syntactically damaged: imports come after code, `useEffect` is broken, and a duplicate JSX tail appears near the end. Preserve the design intent, do not port this file blindly.
- `ProvenanceViewer.tsx` fetches `GET /api/becoming/provenance/<eventId>` and renders a DAG with integrity violation alerts. This is a future/reference audit surface unless the Archetypes game exposes provenance events.

### Themes
Seven CSS files define per-archetype visual signatures. These should be transcribed into Rust theme constants for the Mecha Standard Mode UI rather than hand-waved.

| Archetype | CSS primary accent | Background family | Motion | Display font |
| --- | --- | --- | --- | --- |
| Architect | `#3b82f6` | blueprint blue/black | 160 ms snap, 400 ms heavy | Inter |
| Sentinel | `#dc2626` | red/black security grid | 80 ms snap, 200 ms heavy | default mono/sans |
| Mentor | `#059669` | emerald/wood wisdom | 200 ms snap, 500 ms heavy | Merriweather |
| Explorer | `#f59e0b` | amber frontier | 100 ms snap, 350 ms heavy | Rubik |
| Oracle | `#8b5cf6` | violet/starfield | 180 ms snap, 600 ms heavy | Cinzel |
| Empath | `#ec4899` | rose/moonlit | 220 ms snap, 550 ms heavy | Lora |
| Jester | `#10b981` | electric green/chaos amber | 90 ms snap, 300 ms heavy | Space Mono |
| Mechanician | `#f59e0b` | black/slate/amber | 120 ms snap, 320 ms heavy | default mono/sans |

## Archetype Contract Extracted From Mecha

| Id | Subtitle | Element | Selector role | Selector color | Solid | Portrait | Icon |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `architect` | Systems Designer | Structure & Blueprint | Structure | `#3b82f6` | tetrahedron | `architect.png` | `architect-icon.png` |
| `sentinel` | Guardian of Boundaries | Security & Protection | Master Watcher | `#dc2626` | octahedron | `sentinel.png` | `sentinel-icon.png` |
| `mentor` | Keeper of Wisdom | Knowledge & Context | Wisdom | `#8b5cf6` | cube | `mentor.png` | `mentor-icon.png` |
| `explorer` | Seeker of Frontiers | Discovery & Patterns | Discovery | `#f59e0b` | octahedron | `explorer.png` | `explorer-icon.png` |
| `oracle` | Steward of Foresight | Prophecy & Vision | Foresight | `#10b981` | icosahedron | `oracle.png` | `oracle-icon.png` |
| `empath` | Heart of AURA | Emotion & Continuity | Connection | `#ec4899` | dodecahedron | `empath.png` | `empath-icon.png` |
| `jester` | Law 14 Enforcer | Chaos & Truth | Disruption | `#a855f7` | tetrahedron | `jester.png` | `jester-icon.png` |

Important source mismatch: the selector colors in `mindplane.js` differ from CSS theme colors for Mentor, Oracle, and Jester. This is not a Codex choice to silently "fix." It must be treated as a source fact and resolved deliberately during implementation.

## Required Archetypes Game Feature Contract

The new Standard Mode must be a hard-coded game feature, not an embedded webview and not a vague approximation.

Required player flow:

1. Boot remains the current black `ARCHETYPES` / `A GAME BY MICHAEL HOLT` sequence unless the operator changes that direction.
2. Standard Mode opens into a Mecha-style consciousness selector using `uxbacklayer.png`.
3. All seven archetypes are selectable.
4. Each archetype has its Mecha portrait, icon, metadata, selector position, theme, and hover/active treatment.
5. Selecting an archetype opens a Mecha-style chat surface.
6. Chat is scoped to the active archetype.
7. Switching archetypes updates the theme, portrait, title, subtitle, element, badge/dropdown state, and transcript/history.
8. Chat is real: local LLM route or explicit backend failure. No canned personality stub.
9. History/persistence is real: use the existing Archetypes ledger/app-data lane or a real local history store, not a fake in-memory transcript presented as durable history.
10. System health should map to the local launcher/runtime readiness truth. If GPU/VRAM/RAM/CPU are unavailable, the UI must say unavailable honestly.
11. Old Standard Mode remains referenceable and recoverable.

## Implementation Mapping Into Archetypes

Current Archetypes owner facts:

- `GameMode::Standard` is available in `crates\engine\src\modes\game_mode.rs`.
- Standard currently routes from `chamber\boot.rs` into `ChamberState::Onboarding` or `ChamberState::IdleAtTable`.
- Existing ritual visuals, camera, star, panels, and table live in `crates\engine\src\chamber\*`.
- Oracle Riddle is isolated under `crates\engine\src\modes\oracle_riddle\*`.
- Inner Chambers exists but is locked.
- Local LLM calls already exist in `crates\engine\src\services\llm.rs`.
- Hash-chained event storage already exists in `crates\engine\src\services\ledger.rs`.

Recommended rewrite shape:

- Add a new hard-coded Standard implementation under `crates\engine\src\modes\standard_mecha\`.
- Route `GameMode::Standard` into the Mecha Standard state machine instead of the rejected chamber ritual path.
- Keep the existing `chamber\*` Standard ritual code in the repository as reference. Do not delete it.
- Add a Rust `MechaArchetypeRegistry` equivalent to Mecha's `ARCHETYPE_INFO`, selector positions, image paths, theme tokens, and platonic solid identifiers.
- Copy/import the full required Mecha asset set into the Archetypes asset tree with names intact, then verify packaging through `dist\assets`.
- Build Bevy UI states for:
  - `MechaSelector`
  - `MechaChat`
  - `MechaSwitching`
  - optional `MechaDocs`
  - optional `MechaProvenance`
- Use native Bevy UI and Bevy image assets. Do not depend on Electron, Node `fs`, DOM events, or browser `localStorage`.
- Replace `GET /chat/history` and `POST /chat` semantics with local Rust service equivalents that call the same local Ollama lane and append/read real local history.
- Preserve the fail-visible rule: if Ollama or any local service is unavailable, the player sees a real error and no pretend chat.

## Known Source Risks To Correct During Port

- `mindplane.js` calls `window.themeManager.switchTo(archetypeName)`, but `ThemeManager` exposes `switchTheme()`. Port the intended switch behavior, not the bug.
- `index.html` uses `themeManager.getCurrentArchetype()` inside `sendMessage()` instead of `window.themeManager.getCurrentArchetype()`. In a native port, active archetype state must be explicit.
- `MindplaneOverlay.tsx` is broken as TypeScript/JSX. Preserve concepts, not raw syntax.
- `addMessage()` uses `innerHTML`; the port must render user/LLM text safely.
- Several CSS consumers use `--accent-primary-rgb`, but only some themes define it. Rust theme generation must supply RGB values for all themes.
- Electron path logic and BrowserWindow settings do not apply to Bevy. Assets must travel through the Archetypes install path.
- Backend URLs assume `localhost:8000`; Archetypes currently uses Ollama on `127.0.0.1:11434`, Chronos/Comfy ports, and launcher readiness checks. Endpoint names must be mapped to local services honestly.
- The Credits modal says `Michelle Holt`; verify with the operator before carrying that name forward.
- The source has multiple docs/doctrine modals. They should be preserved as feature candidates, not silently omitted.

## Upstream / Downstream Impact Check

Question: will this affect the system negatively?

It can if implemented carelessly. The safe boundary is:

- Do not change Oracle Riddle scoring, prompt pools, result UI, or ledger payloads.
- Do not unlock Inner Chambers or Living Engine.
- Do not remove the existing chamber ritual code.
- Do not break the boot intro timing already corrected on 2026-07-14/2026-07-15.
- Do not make Standard depend on Electron, Node, browser storage, or the old Mecha backend.
- Do not stage a desktop shortcut refresh until the new Standard Mode runs from the same path the desktop shortcut launches.
- Do not present partial Mecha parity as complete.

What must change to make the feature work:

- Standard Mode routing.
- New Standard Mode state/UI code.
- Asset import and packaging.
- Archetype metadata/theme registry.
- Direct archetype chat service.
- Chat history persistence.
- Tests for routing, registry completeness, and persistence payloads.
- Screenshot/witness verification from the actual desktop-launched game.

What should not change:

- Oracle Riddle.
- locked mode availability.
- ledger hash-chain verification.
- local-first LLM failure behavior.
- the preserved old Standard/chamber code until the operator explicitly approves cleanup.

## Verification Gate For The Actual Rewrite

This audit is not the implementation. The implementation is not done until all of this passes:

- `cargo test --workspace`
- all 7 Mecha asset pairs found by the packaged runtime
- Standard Mode route opens Mecha selector, not the rejected star-table view
- each archetype can be selected
- each archetype shows the correct portrait/icon/title/subtitle/element
- chat sends to a real local LLM/persona route or fails visibly
- history persists and reloads for at least one archetype
- Oracle Riddle still launches and scores
- Inner Chambers and Living Engine remain locked
- desktop-launched app shows the new Standard Mode
- screenshots prove selector, at least two archetype chat views, archetype switch, chat send/failure, and return-to-selector

## Immediate Next Unit

Build the Mecha Standard Mode lane:

1. Add a focused implementation plan.
2. Import or mirror the Mecha assets into the Archetypes asset tree without dropping files by preference.
3. Add the Rust registry and tests proving all seven archetypes are represented.
4. Route Standard Mode into the new selector/chat surface while keeping current chamber code as reference.
5. Wire real archetype chat and history.
6. Run tests, build, refresh desktop, and witness the real app.
