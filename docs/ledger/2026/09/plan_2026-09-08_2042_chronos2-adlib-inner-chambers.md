# Plan: Chronos2 Ad-Lib 3D Generative Integration into Inner Chambers — 2026-09-08 20:42

## Status
SUPERSEDED — preserved as an implementation hypothesis; `C:\archetypes\plan_2026-09-08_archetypes_integration.md` governs product order and requires fixture gameplay before generator integration.

## Goal
This historical plan proposed integrating the Chronos2 TripoSR generative 3D reconstruction engine directly into the Archetypes **Inner Chambers** blockout. It is retained for its technical hypotheses, not as an executable contract. The later integration plan requires a verified generator-contract capture and a fun fixture-prop loop first. Any accepted creation may be appended to the locally verifiable hash-chained ledger; the ledger is not immutable storage.

---

## 1. Context & Architectural Overview

### 1.1 The Current Problem in Inner Chambers
* **Mode Identity:** `GameMode::InnerChambers` (`crates/engine/src/modes/inner_chambers/`) is the sole mode featuring first-person spatial navigation (`PlayerCamera` with WASD movement, Space/Shift vertical flight, and mouse look).
* **Current Primitive State:** The environment is built strictly out of untextured procedural primitives: a flat 90×90 plane, basic cuboid lines for grids/walls, plain spheres, and untextured glowing cubes (`Cuboid::new(1.4, 1.4, 1.4)`) acting as "Truth Nodes." It lacks physical presence, interactive depth, and visual grounding.
* **The Opportunity:** Rather than populating the chambers with static pre-modeled props, we hook the Chronos2 feedforward 3D generator into the gameplay loop. The player's thoughts manifest tangible 3D geometry directly inside the archetype chambers.

### 1.2 The Chronos2 Subsystem Contract
* **Binary Location:** `C:\chronos2\target\release\chronos.exe` (installed sidecar sibling with fallback discovery).
* **Execution Contract:** Mirroring `C:\chronos2\desktop\ui\tab_create.py`:
  ```pwsh
  C:\chronos2\target\release\chronos.exe first-light `
    --prompt "<user_noun>" `
    --out-dir "<out_dir>" `
    --geometry-forge `
    --void
  ```
* **Receipt & Mesh Output:**
  Upon exit code `0`, Chronos2's TripoSR pipeline writes:
  - Output receipt: `<out_dir>\engine_mesh\triposr_artifact.json`
  - Emitted Wavefront geometry: `<out_dir>\0\mesh.obj` (or `<out_dir>\engine_mesh\mesh.obj`)
  - The receipt provides vertex count, face count, SHA-256 hash, and elapsed synthesis time.

---

## 2. End-to-End System Architecture

```mermaid
flowchart TD
    subgraph Client [Archetypes Engine - Inner Chambers]
        A[Player in Inner Chambers] -->|WASD & Mouse Look| B[Enter Archetype Chamber]
        B -->|Press 'T' or Enter| C[Manifestation UI Overlay]
        C -->|Submit Noun: 'chalice'| D[Spawn Ethereal Hologram Proxy]
        D -->|Dispatch Async Task| E[Chronos2Bridge Worker]
    end

    subgraph Sidecar [Chronos2 Subprocess]
        E -->|Non-blocking spawn| F[chronos.exe first-light --prompt 'chalice' --geometry-forge --void]
        F -->|TripoSR Pipeline| G[Synthesize 3D Mesh]
        G -->|Emit Artifacts| H[triposr_artifact.json + mesh.obj]
    end

    subgraph Ingestion [Runtime Pipeline]
        H -->|Poll Completion (rx.try_recv)| I[Verify Receipt & File Integrity]
        I -->|Parse OBJ Lines| J[Bevy Mesh: Positions, Normals, Indices]
        J -->|Apply Archetype Palette| K[PBR StandardMaterial]
        K -->|Despawn Proxy| L[Instantiate 3D Entity in World]
        L -->|Seal Event| M[Hash-Chained Ledger: adlib_object_manifested]
        L -->|Persist Metadata| N[AppData World Memory: manifested_objects.json]
    end
```

---

## 3. Subsystem Specifications

### Subsystem 1: Asynchronous Chronos2 CLI Bridge
* **File:** `crates/engine/src/services/chronos2_bridge.rs`
* **Responsibilities:**
  1. **Binary Discovery:** Locate `chronos.exe` via:
     - Explicit environment variable: `ARCHETYPES_CHRONOS2_BIN`
     - Default release path: `C:\chronos2\target\release\chronos.exe`
     - Installed Program Files / LocalAppData sibling paths
  2. **Execution Directory:** Creates an isolated working directory under:
     `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\adlib\<run_uuid>\`
  3. **Non-Blocking Process Supervision:**
     Uses `std::thread::spawn` with a Rust `mpsc::channel` (`Sender<ManifestationResult>`, `Receiver<ManifestationResult>`) so the 60 FPS Bevy render loop never stutters or drops frames.
  4. **Validation:** Reads and deserializes `triposr_artifact.json`. Verifies that vertex count > 0 and face count > 0.

### Subsystem 2: Native Runtime OBJ Mesh Loader
* **File:** `crates/engine/src/modes/inner_chambers/mesh_loader.rs`
* **Responsibilities:**
  1. **Direct OBJ Parsing:** Pure Rust parser for Wavefront `.obj` files emitted by TripoSR:
     - Geometric vertices: `v x y z` -> `Mesh::ATTRIBUTE_POSITION` (`Vec<[f32; 3]>`)
     - Vertex normals: `vn nx ny nz` -> `Mesh::ATTRIBUTE_NORMAL` (`Vec<[f32; 3]>`)
     - Faces: `f v1//vn1 v2//vn2 v3//vn3` or `f v1 v2 v3` -> `Indices::U32`
     - Fallback normal generation (flat face normals) if TripoSR omits explicit normals.
  2. **Bounding Box & Scale Normalization:**
     TripoSR meshes can vary from 0.2 to 20 units in arbitrary bounding boxes. The loader calculates the Axis-Aligned Bounding Box (AABB), centers the pivot point at the base of the mesh (`y_min = 0.0`), and normalizes the maximum dimension to a configurable chamber scale (default: 1.6 meters height).
  3. **Bevy Mesh Registration:** Inserts the parsed geometry into `Assets<Mesh>`.

### Subsystem 3: In-Chamber Interaction & Manifestation UI
* **Files:** `crates/engine/src/modes/inner_chambers/manifest_ui.rs`, `crates/engine/src/modes/inner_chambers/camera.rs`
* **Responsibilities:**
  1. **State Extension:** Introduce state transitions in `InnerChambersState`:
     - `Navigating`: Standard free-look and WASD movement.
     - `Manifesting`: Text input active; camera rotation locked; blinking caret; prompt buffer.
     - `Synthesizing`: Asynchronous generation underway; player can move around while watching the object forge.
  2. **Spatial Target Calculation:**
     Computes the target spawn coordinates 3.5 units in front of the camera, clamped to the floor plane:
     ```rust
     let forward_flat = Vec3::new(cam_transform.forward().x, 0.0, cam_transform.forward().z).normalize_or_zero();
     let spawn_pos = cam_transform.translation + forward_flat * 3.5;
     let grounded_pos = Vec3::new(spawn_pos.x, 0.0, spawn_pos.z);
     ```
  3. **The Summoning Hologram (Proxy):**
     Immediately spawns a temporary proxy at `grounded_pos`:
     - A rotating archetype-colored wireframe bounding ring / crystal prism.
     - A floating in-world billboard text: `"Forging '<noun>' in Chronos2 TripoSR..."`
     - Emissive pulsing animation driven by `Time::delta_secs()`.

### Subsystem 4: Archetype Styling & Physical Integration
* **File:** `crates/engine/src/modes/inner_chambers/world.rs`
* **Responsibilities:**
  1. **Thematic Material Assignment:**
     Applies the focused room's `ArchetypeTheme` tokens to the manifested object's `StandardMaterial`:
     - **Architect:** Polished blueprint sapphire, `perceptual_roughness: 0.15`, cyan edge glow.
     - **Sentinel:** Heavy dark basalt/obsidian, `metallic: 0.85`, gold perimeter trim.
     - **Mentor:** Warm resonant bronze, `metallic: 0.7`, subtle amber emission.
     - **Explorer:** Verdigris weathered brass, rough surface, orange flare highlights.
     - **Oracle:** Deep indigo glass, `alpha_mode: AlphaMode::Blend`, starlight emission.
     - **Empath:** Luminous rose quartz, soft subsurface feel, pink/white ambient sheen.
     - **Jester:** Chalk/off-white and near-black surfaces with bruise-indigo structure, ash wear, and sparing truth-sting red; no glossy luxury or multichrome treatment.
  2. **Pedestals & Placement:**
     In addition to ad-hoc player spawning, each chamber features a central "Manifestation Plinth" where generated artifacts can snap cleanly.

### Subsystem 5: Ledger Sealing, World Memory & Session Persistence
* **Files:** `crates/engine/src/modes/inner_chambers/seed.rs`, `crates/engine/src/services/memory.rs`
* **Responsibilities:**
  1. **Ledger Integration:** Each completed object seals an `adlib_object_manifested` event:
     ```json
     {
       "event": "adlib_object_manifested",
       "prompt": "chalice",
       "chamber": "Architect",
       "mesh_sha256": "e3b0c44298fc1c149afbf4c8996fb924...",
       "vertex_count": 2840,
       "face_count": 5676,
       "transform": {
         "translation": [12.4, 0.0, -8.2],
         "rotation": [0.0, 0.38, 0.0, 0.92],
         "scale": [1.0, 1.0, 1.0]
       }
     }
     ```
  2. **Session Persistence:**
     Manifested objects are recorded in `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\inner_chambers\manifested_objects.json`.
     When entering `InnerChambersState::Loading`, existing records are read, OBJ files are cached/reloaded, and all previously crafted artifacts populate their original coordinates.

---

## 4. Execution Steps

### Step 1 — Pure Wavefront OBJ Ingestion Module (Detailed Checklist)

TripoSR outputs Wavefront `.obj` files with distinct characteristics:
1. **Z-Up Coordinate System:** TripoSR writes with Z as the vertical axis (`MESH_UP_AXIS = "Z"` in `chronos2/tools/triposr_mesh_emitter.py`). Bevy uses Y-Up (`+Y` up, `-Z` forward). The loader must swizzle coordinates: `[x, z, -y]`.
2. **Inline RGB Vertex Colors:** `v` lines contain 6 floats: `v x y z r g b`. The loader extracts RGB into Bevy's `Mesh::ATTRIBUTE_COLOR` (`[r, g, b, 1.0]`).
3. **No Vertex Normals:** `vn` lines are omitted entirely by TripoSR. The loader calculates area-weighted smooth vertex normals from triangle cross products.
4. **1-Indexed Faces:** Faces are written as `f v1 v2 v3` (1-indexed integers). Negative indices and quads (`f v1 v2 v3 v4`) must be handled gracefully.

#### Step 1.1 — Data Structures & Parser Configuration
- [ ] Action: Define the parsing intermediate representations in `crates/engine/src/modes/inner_chambers/mesh_loader.rs`.
  - [ ] Create `ParsedVertex`:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct ParsedVertex {
        pub position: [f32; 3],       // [x, y, z] in Bevy space (+Y up)
        pub color: Option<[f32; 4]>,  // [r, g, b, a] normalized 0.0..=1.0
    }
    ```
  - [ ] Create `BoundingBox`:
    ```rust
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct BoundingBox {
        pub min: Vec3,
        pub max: Vec3,
    }
    impl BoundingBox {
        pub fn new() -> Self {
            Self {
                min: Vec3::splat(f32::INFINITY),
                max: Vec3::splat(f32::NEG_INFINITY),
            }
        }
        pub fn extend(&mut self, p: Vec3) {
            self.min = self.min.min(p);
            self.max = self.max.max(p);
        }
        pub fn height(&self) -> f32 { self.max.y - self.min.y }
        pub fn center(&self) -> Vec3 { (self.min + self.max) * 0.5 }
    }
    ```
  - [ ] Create error type `MeshLoadError` implementing `std::fmt::Display` and `std::error::Error`:
    - `EmptyMesh`: "File contains no valid vertices or faces"
    - `DegenerateMesh`: "Mesh bounding box height is zero"
    - `IndexOutOfBounds(usize)`: "Face index exceeds vertex count"
    - `IoError(String)`: "Failed to read mesh file"

#### Step 1.2 — Line Lexer & Coordinate Swizzling (Z-Up to Y-Up)
- [ ] Action: Implement `parse_obj_lines(content: &str) -> Result<(Vec<ParsedVertex>, Vec<[u32; 3]>), MeshLoadError>`.
  - [ ] Iterate lines with `.lines()`, trimming leading/trailing whitespace.
  - [ ] Skip comment lines starting with `#` and ignore material markers (`mtllib`, `usemtl`, `o`, `g`, `s`).
  - [ ] Parse `v` lines:
    - Split whitespace: tokens `["v", x, y, z, ... optional r, g, b]`.
    - Parse raw floats: `x_raw = tokens[1].parse::<f32>()?`, `y_raw = tokens[2].parse::<f32>()?`, `z_raw = tokens[3].parse::<f32>()?`.
    - **Coordinate Swizzle (TripoSR Z-Up to Bevy Y-Up):**
      ```rust
      let bevy_x = x_raw;
      let bevy_y = z_raw;  // TripoSR Z is vertical
      let bevy_z = -y_raw; // Invert TripoSR Y to maintain right-handed system
      ```
    - Parse optional vertex colors: if `tokens.len() >= 7`, parse `r = tokens[4]`, `g = tokens[5]`, `b = tokens[6]`, clamping to `[0.0, 1.0]`. Pack as `Some([r, g, b, 1.0])`. Otherwise, `None`.
  - [ ] Parse `f` lines:
    - Extract face vertex indices. Handle format `v`, `v/vt`, `v//vn`, or `v/vt/vn` by splitting on `/` and parsing the first integer.
    - Convert 1-based indexing to 0-based indexing: `index - 1`.
    - Support negative indices (e.g. `-1` refers to `vertices.len() - 1`).
    - Triangulate polygons: if a face has 3 vertices `[v0, v1, v2]`, emit triangle `[v0, v1, v2]`. If 4 vertices (quad) `[v0, v1, v2, v3]`, triangulate as `[v0, v1, v2]` and `[v0, v2, v3]`. If >4 vertices, fan-triangulate around `v0`.

#### Step 1.3 — AABB Calculation, Grounding & Height Normalization
- [ ] Action: Implement `normalize_mesh_scale(vertices: &mut [ParsedVertex], target_height: f32) -> Result<BoundingBox, MeshLoadError>`.
  - [ ] Compute initial AABB over all `vertex.position`.
  - [ ] Guard against degenerate geometry: if `bbox.height() < 1e-6`, return `Err(MeshLoadError::DegenerateMesh)`.
  - [ ] Compute uniform scaling factor:
    ```rust
    let scale = target_height / bbox.height();
    ```
  - [ ] Compute translation offsets to center on X/Z and ground on Y (`min_y = 0.0`):
    ```rust
    let center_x = (bbox.min.x + bbox.max.x) * 0.5;
    let center_z = (bbox.min.z + bbox.max.z) * 0.5;
    let offset_x = -center_x * scale;
    let offset_y = -bbox.min.y * scale; // Ground the base to 0.0
    let offset_z = -center_z * scale;
    ```
  - [ ] Apply scale and translation in-place to every vertex position:
    ```rust
    for v in vertices.iter_mut() {
        v.position[0] = v.position[0] * scale + offset_x;
        v.position[1] = v.position[1] * scale + offset_y;
        v.position[2] = v.position[2] * scale + offset_z;
    }
    ```
  - [ ] Recompute normalized bounding box: assert `min.y` is `0.0 ± 0.0001` and `max.y` is `target_height ± 0.0001`.

#### Step 1.4 — Surface Normal Generation (Area-Weighted Vertex Normals)
- [ ] Action: Implement `compute_vertex_normals(positions: &[[f32; 3]], triangles: &[[u32; 3]]) -> Vec<[f32; 3]>`.
  - [ ] Initialize normal accumulator buffer: `let mut normals = vec![Vec3::ZERO; positions.len()];`.
  - [ ] For each triangle `[i0, i1, i2]`:
    - Fetch positions: `p0 = Vec3::from(positions[i0])`, `p1 = Vec3::from(positions[i1])`, `p2 = Vec3::from(positions[i2])`.
    - Compute unnormalized face normal:
      ```rust
      let edge1 = p1 - p0;
      let edge2 = p2 - p0;
      let face_normal = edge1.cross(edge2); // Length is 2 * triangle area
      ```
    - Accumulate face normal into each vertex:
      ```rust
      normals[i0 as usize] += face_normal;
      normals[i1 as usize] += face_normal;
      normals[i2 as usize] += face_normal;
      ```
  - [ ] Normalize all accumulated vertex normals:
    ```rust
    normals.into_iter().map(|n| {
        let normalized = n.normalize_or(Vec3::Y);
        [normalized.x, normalized.y, normalized.z]
    }).collect()
    ```

#### Step 1.5 — Bevy Mesh Asset Assembly
- [ ] Action: Implement `build_bevy_mesh(vertices: &[ParsedVertex], normals: &[[f32; 3]], triangles: &[[u32; 3]]) -> Mesh`.
  - [ ] Construct `Mesh`:
    ```rust
    let mut mesh = Mesh::new(
        bevy::render::render_resource::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    ```
  - [ ] Insert `Mesh::ATTRIBUTE_POSITION`: `positions: Vec<[f32; 3]> = vertices.iter().map(|v| v.position).collect()`.
  - [ ] Insert `Mesh::ATTRIBUTE_NORMAL`: pass calculated `normals`.
  - [ ] If any vertex has a color, extract all colors (defaulting missing colors to `[1.0, 1.0, 1.0, 1.0]`) and insert `Mesh::ATTRIBUTE_COLOR`: `Vec<[f32; 4]>`.
  - [ ] Flatten triangle indices: `indices: Vec<u32> = triangles.iter().flat_map(|tri| *tri).collect()`.
  - [ ] Insert indices: `mesh.insert_indices(bevy::render::mesh::Indices::U32(indices))`.

#### Step 1.6 — Exhaustive Pure Unit Test Suite
- [ ] Action: Write pure unit tests in `crates/engine/src/modes/inner_chambers/mesh_loader.rs` under `#[cfg(test)]`.
  - [ ] **Test 1: Empty and Degenerate Guard:**
    Assert parsing `""` or `"v 0 0 0\n"` returns `Err(MeshLoadError::EmptyMesh)`.
    Assert flat 2D point line returns `Err(MeshLoadError::DegenerateMesh)`.
  - [ ] **Test 2: Basic Triangle Parsing & Coordinate Swizzling:**
    Pass input:
    ```
    v 1.0 0.0 2.0
    v 0.0 1.0 2.0
    v 0.0 0.0 2.0
    f 1 2 3
    ```
    Assert TripoSR Z (`2.0`) maps to Bevy Y (`2.0`).
  - [ ] **Test 3: AABB Normalization & Grounding:**
    Pass a 10.0m tall pyramid with base at `z = -5.0`.
    Normalize to `target_height = 1.6`.
    Assert the resulting vertices have `min_y == 0.0`, `max_y == 1.6`, and center `(x, z) == (0.0, 0.0)`.
  - [ ] **Test 4: Quad Triangulation:**
    Pass `f 1 2 3 4`. Assert exactly 6 indices (2 triangles) are produced: `[0, 1, 2, 0, 2, 3]`.
  - [ ] **Test 5: TripoSR RGB Vertex Color Extraction:**
    Pass `v 1.0 2.0 3.0 0.8039 0.7725 0.7490`.
    Assert parsed color attribute is `[0.8039, 0.7725, 0.7490, 1.0]`.
  - [ ] **Test 6: Computed Normal Direction:**
    Pass a flat horizontal square in the XZ plane with CCW winding.
    Assert all calculated vertex normals point straight up: `[0.0, 1.0, 0.0]`.
  - [ ] **Test 7: Live TripoSR Fixture Verification:**
    Load a 100-line sample snippet from `C:\chronos2\out\object_triposr_only_20260908\engine_mesh\0\mesh.obj`.
    Assert the mesh loads, generates valid non-NaN normals, populates `ATTRIBUTE_COLOR`, and produces a valid Bevy `Mesh`.

#### Step 1.7 — Module Registration & Cargo Gate
- [ ] Action: Expose `mesh_loader` in `crates/engine/src/modes/inner_chambers/mod.rs`:
  ```rust
  pub mod mesh_loader;
  pub use mesh_loader::load_obj_as_bevy_mesh;
  ```
- [ ] Action: Run `cargo test -p engine mesh_loader` and assert all 7 tests pass. Validate that no external crates are required beyond standard library and Bevy.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/mesh_loader.rs` (new)
  - `crates/engine/src/modes/inner_chambers/mod.rs`
- Expected outcome: `cargo test -p engine mesh_loader` passes without dependencies on running GPU/Chronos.

### Step 2 — Chronos2 Async Process Bridge (Detailed Checklist)

This checklist breaks down Phase 2 into granular, testable engineering tasks. Each step implements a robust, non-blocking subprocess bridge that communicates with Chronos2 and validates receipt artifacts.

#### Step 2.1 — Strong Types & Receipt Deserialization Models
- [ ] Action: Define data structures and error enums in `crates/engine/src/services/chronos2_bridge.rs`.
  - [ ] Implement `TriposrArtifactReceipt` matching schema `chronosophia.triposr-mesh.v1`:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TriposrArtifactReceipt {
        pub schema: String,
        #[serde(default)]
        pub emitted_at_utc: String,
        pub engine: TriposrEngineRecord,
        pub mesh: TriposrMeshRecord,
        #[serde(default)]
        pub timing: Option<TriposrTimingRecord>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TriposrEngineRecord {
        pub name: String,
        #[serde(default)]
        pub root: Option<String>,
        #[serde(default)]
        pub weights_sha256: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TriposrMeshRecord {
        pub path: String,
        pub sha256: String,
        #[serde(default)]
        pub up_axis: Option<String>,
        pub vertices: usize,
        pub faces: usize,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct TriposrTimingRecord {
        #[serde(default)]
        pub elapsed_secs: f64,
    }
    ```
  - [ ] Implement `Chronos2BridgeError`:
    ```rust
    #[derive(Debug, Clone, PartialEq)]
    pub enum Chronos2BridgeError {
        BinaryNotFound(String),
        SubprocessSpawnFailed(String),
        SubprocessFailed { code: i32, stderr: String },
        TimedOut { timeout_secs: u64 },
        ReceiptNotFound(std::path::PathBuf),
        InvalidReceipt(String),
        MissingMeshFile(std::path::PathBuf),
        DegenerateMesh { vertices: usize, faces: usize },
    }
    ```
  - [ ] Implement `ManifestationRequest` and `ManifestationResult`:
    ```rust
    #[derive(Debug, Clone)]
    pub struct ManifestationRequest {
        pub prompt: String,
        pub chamber_index: usize,
        pub spawn_transform: Transform,
    }

    #[derive(Debug, Clone)]
    pub struct ManifestationResult {
        pub prompt: String,
        pub chamber_index: usize,
        pub spawn_transform: Transform,
        pub mesh_path: std::path::PathBuf,
        pub receipt_path: std::path::PathBuf,
        pub sha256: String,
        pub vertices: usize,
        pub faces: usize,
        pub elapsed_secs: f64,
    }
    ```

#### Step 2.2 — Executable Discovery & Resolution
- [ ] Action: Implement `resolve_chronos2_executable() -> Result<std::path::PathBuf, Chronos2BridgeError>`.
  - [ ] Check explicit environment variable: `ARCHETYPES_CHRONOS2_BIN`. If set and `is_file()`, return it immediately.
  - [ ] Check repository sibling checkout: `C:\chronos2\target\release\chronos.exe`. If `is_file()`, return it.
  - [ ] Check installed user applications: `%LOCALAPPDATA%\Programs\ChronoSophia\chronos.exe`.
  - [ ] Check sidecar dependencies manifest: verify against `dependencies.json`.
  - [ ] Fail-closed error reporting: if all candidates fail, return `Err(Chronos2BridgeError::BinaryNotFound(...))` enumerating all searched locations.

#### Step 2.3 — Working Directory Isolation & Argv Construction
- [ ] Action: Implement `build_chronos2_command(exe: &Path, prompt: &str, out_dir: &Path) -> std::process::Command`.
  - [ ] Create isolated run directory: `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\adlib\<run_uuid>\` using `uuid::Uuid::new_v4()`.
  - [ ] Create parent directory recursively with `std::fs::create_dir_all(out_dir)`.
  - [ ] Assemble argument vector mirroring `C:\chronos2\desktop\ui\tab_create.py`:
    ```rust
    let mut cmd = std::process::Command::new(exe);
    cmd.args([
        "first-light",
        "--prompt",
        prompt,
        "--out-dir",
        out_dir.to_str().ok_or_else(|| Chronos2BridgeError::InvalidReceipt("Bad path".into()))?,
        "--geometry-forge",
        "--void",
    ]);
    ```
  - [ ] On Windows, set process creation flag `CREATE_NO_WINDOW` (`0x08000000`) so background execution does not flash a terminal window over the gameplay:
    ```rust
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    ```

#### Step 2.4 — Worker Thread Supervisor & Timeout Mechanism
- [ ] Action: Implement `spawn_manifestation_worker(request: ManifestationRequest, sender: mpsc::Sender<Result<ManifestationResult, Chronos2BridgeError>>)`.
  - [ ] Spawn named background OS thread: `std::thread::Builder::new().name("chronos2-adlib-worker".into()).spawn(move || { ... })`.
  - [ ] Launch subprocess with `Stdio::piped()` for stderr and stdout.
  - [ ] Supervise child execution loop:
    - Poll `child.try_wait()` at 100ms sleep intervals.
    - Measure wall-clock elapsed time. If elapsed > 60 seconds (configurable timeout):
      - Call `child.kill()`.
      - Send `Err(Chronos2BridgeError::TimedOut { timeout_secs: 60 })`.
  - [ ] If process exits non-zero:
    - Read `stderr` into string.
    - Send `Err(Chronos2BridgeError::SubprocessFailed { code, stderr })`.

#### Step 2.5 — Receipt Discovery, Deserialization & File Validation
- [ ] Action: Implement `parse_and_validate_receipt(out_dir: &Path) -> Result<(TriposrArtifactReceipt, PathBuf), Chronos2BridgeError>`.
  - [ ] Check primary receipt path: `out_dir.join("engine_mesh").join("triposr_artifact.json")`.
  - [ ] Check fallback receipt path: `out_dir.join("triposr_artifact.json")`.
  - [ ] If neither exists, return `Err(Chronos2BridgeError::ReceiptNotFound)`.
  - [ ] Read file to string, deserialize via `serde_json::from_str::<TriposrArtifactReceipt>()`.
  - [ ] Enforce receipt invariants:
    - `receipt.engine.name == "TripoSR"`: reject if engine mismatched.
    - `receipt.mesh.vertices > 0 && receipt.mesh.faces > 0`: reject degenerate mesh.
    - Check `Path::new(&receipt.mesh.path).is_file()`: verify output `.obj` exists on disk.
  - [ ] On success, construct `ManifestationResult` and send `Ok(result)` across channel.

#### Step 2.6 — Bevy Resource & System Integration
- [ ] Action: Define Bevy resources in `crates/engine/src/services/chronos2_bridge.rs`.
  - [ ] `Chronos2BridgeChannel` Resource:
    ```rust
    #[derive(Resource)]
    pub struct Chronos2BridgeChannel {
        pub sender: std::sync::mpsc::Sender<Result<ManifestationResult, Chronos2BridgeError>>,
        pub receiver: std::sync::Mutex<std::sync::mpsc::Receiver<Result<ManifestationResult, Chronos2BridgeError>>>,
    }
    ```
  - [ ] Implement `init_bridge_channel(app: &mut App)`:
    - Call `mpsc::channel()`.
    - Register non-blocking resource in Bevy world.

#### Step 2.7 — Comprehensive Pure & Mock Unit Tests
- [ ] Action: Write unit tests under `#[cfg(test)]` in `chronos2_bridge.rs`.
  - [ ] **Test 1: Argv Construction Contract:** Assert that command construction contains `first-light`, `--prompt`, `--out-dir`, `--geometry-forge`, and `--void`.
  - [ ] **Test 2: Schema Deserialization Success:** Deserialize a live fixture matching `C:\chronos2\out\object_triposr_only_20260908\engine_mesh\triposr_artifact.json`. Assert schema, engine name `"TripoSR"`, vertex count `51240`, face count `102318`, and up-axis `"Z"`.
  - [ ] **Test 3: Reject Degenerate Mesh Receipt:** Pass mock receipt with `vertices: 0, faces: 0`. Assert returns `Err(Chronos2BridgeError::DegenerateMesh)`.
  - [ ] **Test 4: Missing Receipt Detection:** Query a temporary directory without JSON. Assert returns `Err(Chronos2BridgeError::ReceiptNotFound)`.
  - [ ] **Test 5: Cross-Thread Channel Non-Blocking Transfer:** Spawn a mock worker thread that sends a mock `ManifestationResult`. Verify main thread receives it cleanly via `receiver.lock().unwrap().try_recv()` without frame stutter.
- Files touched:
  - `crates/engine/src/services/chronos2_bridge.rs` (new)
  - `crates/engine/src/services/mod.rs`
- Expected outcome: Subprocess invocation is completely decoupled from the Bevy frame tick.

### Step 3 — Inner Chambers Manifestation UI & Controls (Detailed Checklist)

This checklist breaks down Phase 3 into granular, testable engineering tasks. Each step implements a focused in-world manifestation interface, manages keystroke capture, and choreographs camera state transitions between free flight and text input.

#### Step 3.1 — State Machine Extension & Resource Registration
- [ ] Action: Extend `InnerChambersState` in `crates/engine/src/modes/inner_chambers/mod.rs`:
  ```rust
  #[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
  pub enum InnerChambersState {
      #[default]
      Inactive,
      Loading,
      Navigating,
      Manifesting,
      Exiting,
  }
  ```
- [ ] Action: Define the prompt buffer resource in `crates/engine/src/modes/inner_chambers/manifest_ui.rs`:
  ```rust
  #[derive(Resource, Default, Debug, Clone)]
  pub struct ManifestPromptBuffer {
      pub text: String,
      pub error_msg: Option<String>,
  }

  impl ManifestPromptBuffer {
      pub fn clear(&mut self) {
          self.text.clear();
          self.error_msg = None;
      }
      pub fn push_char(&mut self, c: char) {
          if self.text.len() < 40 && !c.is_control() {
              self.text.push(c);
              self.error_msg = None;
          }
      }
      pub fn backspace(&mut self) {
          self.text.pop();
          self.error_msg = None;
      }
  }
  ```
- [ ] Action: Register `ManifestPromptBuffer::default()` in `InnerChambersPlugin`.

#### Step 3.2 — Camera Movement Gating & State Choreography
- [ ] Action: Update `crates/engine/src/modes/inner_chambers/camera.rs`:
  - [ ] Camera Spawning: Keep `setup_camera` running on `OnEnter(InnerChambersState::Loading)`.
  - [ ] Camera Motion Gating: Ensure `camera_movement` system runs exclusively in `Navigating`:
    ```rust
    .add_systems(
        Update,
        camera_movement.run_if(in_state(InnerChambersState::Navigating)),
    )
    ```
  - [ ] Verification: When transitioning `Navigating -> Manifesting`, `PlayerCamera` remains active so the 3D scene continues rendering, but mouse delta (yaw/pitch) and keyboard translation (WASD/Space/Shift) are completely ignored.
  - [ ] Camera Teardown: Keep `teardown_camera` running on `OnEnter(InnerChambersState::Exiting)`.

#### Step 3.3 — HUD Navigation Prompt & Mode Trigger
- [ ] Action: Update hint text and trigger detection in `crates/engine/src/modes/inner_chambers/world.rs` and `extraction.rs`:
  - [ ] In `setup_inner_world`, update default hint banner:
    ```
    INNER CHAMBERS
    Seven minds surround the hub. WASD moves. Mouse looks.
    [T] Manifest 3D Object  •  [E] Read Node Truth  •  [Esc] Main Menu
    ```
  - [ ] In `check_extraction` (or dedicated navigation input system):
    - When `keyboard.just_pressed(KeyCode::KeyT)`:
      - Set `next_state.set(InnerChambersState::Manifesting)`.
      - Reset `ManifestPromptBuffer`.

#### Step 3.4 — In-Chamber Manifestation UI Construction
- [ ] Action: Implement `setup_manifest_ui` on `OnEnter(InnerChambersState::Manifesting)` in `manifest_ui.rs`:
  - [ ] Mark root container with component `ManifestUiRoot`.
  - [ ] Root Node Styling:
    - `position_type: PositionType::Absolute`
    - `left: Val::Percent(20.0)`, `right: Val::Percent(20.0)`, `bottom: Val::Px(80.0)`
    - `padding: UiRect::axes(Val::Px(24.0), Val::Px(16.0))`
    - `background_color: Color::srgba(0.02, 0.025, 0.04, 0.94)` (pure ceremonial dark void)
    - `border: UiRect::all(Val::Px(1.0))`
    - `border_color: Color::srgb(0.85, 0.72, 0.38)` (Archetypes hairline gold)
    - `flex_direction: FlexDirection::Column`
  - [ ] Header Banner:
    - Text: `"MANIFEST 3D FORM IN THE INNER CHAMBER"`
    - Font size: `16.0`, Color: `Color::srgb(0.85, 0.72, 0.38)` (Gold)
  - [ ] Input Field Container:
    - `margin: UiRect::top(Val::Px(10.0))`
    - Child 1: Prompt Text (Component `ManifestInputText`) displaying the user's active string buffer.
    - Child 2: Blinking Caret (Component `ManifestCaretText`) displaying `_`.
  - [ ] Subtitle & Instructions:
    - Text: `"Enter a single noun (e.g. chalice, obelisk, anvil, crown) • [Enter] Forge • [Esc] Cancel"`
    - Font size: `13.0`, Color: `Color::srgb(0.60, 0.65, 0.75)`
  - [ ] Error / Status Label:
    - Component `ManifestErrorText`
    - Font size: `13.0`, Color: `Color::srgb(1.0, 0.40, 0.35)`
- [ ] Action: Implement `teardown_manifest_ui` on `OnExit(InnerChambersState::Manifesting)`:
  - Query all entities `With<ManifestUiRoot>` and call `commands.entity(e).despawn_recursive()`.

#### Step 3.5 — Keystroke Capture & Buffer Management System
- [ ] Action: Implement `handle_manifest_keystrokes` in `manifest_ui.rs` running `run_if(in_state(InnerChambersState::Manifesting))`:
  - [ ] Inject `keyboard: Res<ButtonInput<Key>>`, `mut buffer: ResMut<ManifestPromptBuffer>`, `mut next_state: ResMut<NextState<InnerChambersState>>`, `query_cam: Query<&Transform, With<PlayerCamera>>`, `channel: Res<Chronos2BridgeChannel>`.
  - [ ] Iterate `keyboard.get_just_pressed()`:
    - `Key::Escape`:
      - Discard current draft.
      - Set `next_state.set(InnerChambersState::Navigating)`.
      - Return immediately (cancels manifestation without leaving the room).
    - `Key::Backspace`:
      - Call `buffer.backspace()`.
    - `Key::Space`:
      - If buffer is not empty and does not end with space: `buffer.push_char(' ')`.
    - `Key::Character(text)`:
      - For each char in `text`: `buffer.push_char(c)`.
    - `Key::Enter`:
      - Check trimmed text: `let trimmed = buffer.text.trim();`.
      - If empty:
        - Set `buffer.error_msg = Some("Please enter a noun to manifest.".into())`.
      - If valid:
        - Fetch player camera transform: `let cam_transform = query_cam.single()`.
        - Calculate grounded target point:
          ```rust
          let forward_flat = Vec3::new(cam_transform.forward().x, 0.0, cam_transform.forward().z).normalize_or_zero();
          let spawn_pos = cam_transform.translation + forward_flat * 3.5;
          let grounded_pos = Vec3::new(spawn_pos.x, 0.0, spawn_pos.z);
          ```
        - Identify active chamber index: `nearest_chamber_index(cam_transform.translation)`.
        - Dispatch `ManifestationRequest` via `spawn_manifestation_worker(request, channel.sender.clone())`.
        - Spawn in-chamber summoning proxy at `grounded_pos` (Phase 4 integration).
        - Transition: `next_state.set(InnerChambersState::Navigating)`.

#### Step 3.6 — Visual UI Update Systems (Draft & Caret Animation)
- [ ] Action: Implement `render_manifest_ui` running `run_if(in_state(InnerChambersState::Manifesting))`:
  - [ ] Update `ManifestInputText`:
    - Display current buffer: `text.0 = buffer.text.clone()`.
  - [ ] Update `ManifestCaretText`:
    - Animate blinking: `let visible = (time.elapsed_secs() * 2.0).fract() < 0.5;`
    - Display `" _"` when visible, `"  "` when hidden.
  - [ ] Update `ManifestErrorText`:
    - Display `buffer.error_msg.clone().unwrap_or_default()`.

#### Step 3.7 — Comprehensive Pure Unit Tests
- [ ] Action: Write unit tests under `#[cfg(test)]` in `crates/engine/src/modes/inner_chambers/manifest_ui.rs`:
  - [ ] **Test 1 (`buffer_push_and_backspace`):**
    Test typing `"chalice"`, pushing backspace twice -> asserts buffer equals `"chal"`.
  - [ ] **Test 2 (`buffer_length_limit_and_control_char_filter`):**
    Attempt to push 45 characters. Assert buffer length caps strictly at 40 characters.
    Attempt to push control characters (`\n`, `\t`, `\x00`). Assert they are rejected.
  - [ ] **Test 3 (`empty_submission_sets_error_msg`):**
    Simulate Enter with empty buffer `""` and whitespace buffer `"   "`. Assert error message is populated and state remains `Manifesting`.
  - [ ] **Test 4 (`valid_submission_produces_grounded_target`):**
    Given camera at `Vec3::new(10.0, 2.0, 5.0)` looking in direction `Vec3::new(0.0, 0.0, -1.0)`:
    Assert calculated spawn position has `x = 10.0`, `y = 0.0` (grounded), `z = 1.5` (forward 3.5m).
  - [ ] **Test 5 (`escape_key_clears_buffer`):**
    Simulate Escape key pressed with active draft `"meteorite"`. Assert buffer clears and returns to `Navigating`.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/manifest_ui.rs` (new)
  - `crates/engine/src/modes/inner_chambers/camera.rs`
  - `crates/engine/src/modes/inner_chambers/mod.rs`
- Expected outcome: Pressing 'T' cleanly opens the input field, freezes the camera, and allows typing a noun prompt.

### Step 4 — In-Chamber Proxy Spawning & Object Instantiation (Detailed Checklist)

This checklist breaks down Phase 4 into granular, testable engineering tasks. Each step manages in-world proxy representation, real-time animation, asynchronous mesh reception, archetype PBR material shading, and physical instantiation into the archetype chamber.

#### Step 4.1 — Summoning Proxy & Manifested Entity Data Architecture
- [ ] Action: Implement core ECS components in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] Implement `SummoningProxy`:
    ```rust
    #[derive(Component, Debug, Clone)]
    pub struct SummoningProxy {
        pub prompt: String,
        pub chamber_index: usize,
        pub started_at: f64,
        pub target_position: Vec3,
        pub target_rotation: Quat,
        pub failed: bool,
        pub error_msg: Option<String>,
        pub despawn_timer: Option<Timer>,
    }

    impl SummoningProxy {
        pub fn new(prompt: String, chamber_index: usize, target_position: Vec3, target_rotation: Quat, started_at: f64) -> Self {
            Self {
                prompt,
                chamber_index,
                started_at,
                target_position,
                target_rotation,
                failed: false,
                error_msg: None,
                despawn_timer: None,
            }
        }

        pub fn mark_failed(&mut self, err: String) {
            self.failed = true;
            self.error_msg = Some(err);
            self.despawn_timer = Some(Timer::from_seconds(6.0, TimerMode::Once));
        }
    }
    ```
  - [ ] Implement `ManifestedEntity`:
    ```rust
    #[derive(Component, Debug, Clone)]
    pub struct ManifestedEntity {
        pub prompt: String,
        pub chamber_index: usize,
        pub mesh_path: std::path::PathBuf,
        pub sha256: String,
        pub spawned_at: f64,
        pub vertices: usize,
        pub faces: usize,
    }
    ```
  - [ ] Implement child marker components for isolated query updates:
    ```rust
    #[derive(Component, Debug, Default)]
    pub struct ProxyVisualRing {
        pub spin_speed: f32,
    }

    #[derive(Component, Debug, Default)]
    pub struct ProxyVisualCore {
        pub tumble_axes: Vec3,
    }

    #[derive(Component, Debug, Default)]
    pub struct ProxyBillboardText;

    #[derive(Component, Debug, Default)]
    pub struct ProxyPulsingLight {
        pub base_intensity: f32,
        pub pulse_amplitude: f32,
        pub frequency: f32,
    }

    #[derive(Component, Debug)]
    pub struct ArrivalBurstLight {
        pub timer: Timer,
        pub initial_intensity: f32,
    }
    ```

#### Step 4.2 — Ethereal Hologram Proxy Visual Hierarchy Spawning
- [ ] Action: Implement `spawn_summoning_proxy` in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] Function signature:
    ```rust
    pub fn spawn_summoning_proxy(
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<StandardMaterial>,
        request: &ManifestationRequest,
        resolved_transform: Transform,
        time_now: f64,
    ) -> Entity
    ```
  - [ ] Retrieve active chamber archetype:
    ```rust
    let spec = &council_chambers()[request.chamber_index];
    let theme = spec.archetype.theme();
    ```
  - [ ] Construct holographic translucent material with high alpha blending and vibrant emissive radiance:
    ```rust
    let proxy_mat = materials.add(StandardMaterial {
        base_color: theme.accent_primary.with_alpha(0.60),
        emissive: LinearRgba::from(theme.accent_primary) * 2.2,
        unlit: false,
        alpha_mode: AlphaMode::Blend,
        perceptual_roughness: 0.1,
        metallic: 0.3,
        ..default()
    });
    ```
  - [ ] Spawn root proxy entity at `resolved_transform`:
    - Assign `SummoningProxy::new(request.prompt.clone(), request.chamber_index, resolved_transform.translation, resolved_transform.rotation, time_now)`.
    - Assign `InnerWorldElement` to guarantee clean teardown on chamber exit.
    - Assign `resolved_transform`.
    - Assign `Visibility::default()`.
    - Assign `Name::new(format!("Proxy_{}_{}", request.chamber_index, request.prompt))`.
  - [ ] Attach children to root entity:
    - **Outer Summoning Cylinder / Ground Ring:**
      - Geometry: `meshes.add(Cylinder::new(1.35, 0.04))`.
      - Offset: `Transform::from_xyz(0.0, 0.02, 0.0)`.
      - Component: `ProxyVisualRing { spin_speed: 1.2 }`.
    - **Inner Counter-Spinning Glyph Disc:**
      - Geometry: `meshes.add(Cylinder::new(0.85, 0.03))`.
      - Offset: `Transform::from_xyz(0.0, 0.03, 0.0)`.
      - Component: `ProxyVisualRing { spin_speed: -1.8 }`.
    - **Floating Crystalline Core:**
      - Geometry: `meshes.add(Cuboid::new(0.55, 0.55, 0.55))`.
      - Offset: `Transform::from_xyz(0.0, 0.85, 0.0)`.
      - Component: `ProxyVisualCore { tumble_axes: Vec3::new(0.9, 1.3, 0.7) }`.
    - **Pulsing Archetype Point Light:**
      - Light:
        ```rust
        PointLight {
            color: theme.accent_primary,
            intensity: 45_000.0,
            range: 9.0,
            shadows_enabled: false,
            ..default()
        }
        ```
      - Offset: `Transform::from_xyz(0.0, 1.2, 0.0)`.
      - Component: `ProxyPulsingLight { base_intensity: 30_000.0, pulse_amplitude: 40_000.0, frequency: 3.2 }`.
    - **In-World Billboard Status Text:**
      - Offset: `Transform::from_xyz(0.0, 2.3, 0.0)`.
      - Component: `ProxyBillboardText`.
      - Text: `format!("FORGING '{}' IN TRIPOSR...", request.prompt.to_uppercase())`.
      - Styling: Gold hairline color `Color::srgb(0.92, 0.82, 0.45)`, font size `16.0`.

#### Step 4.3 — Real-Time Proxy Animation & Camera-Facing Billboard Dynamics
- [ ] Action: Implement `animate_summoning_proxies` running in `Update` restricted to `run_if(in_state(InnerChambersState::Navigating))`:
  - [ ] System signature:
    ```rust
    pub fn animate_summoning_proxies(
        mut commands: Commands,
        time: Res<Time>,
        mut query_proxies: Query<(Entity, &mut SummoningProxy)>,
        mut query_rings: Query<(&mut Transform, &ProxyVisualRing), Without<ProxyVisualCore>>,
        mut query_cores: Query<(&mut Transform, &ProxyVisualCore), Without<ProxyVisualRing>>,
        mut query_lights: Query<(&mut PointLight, &ProxyPulsingLight)>,
        mut query_billboards: Query<&mut Transform, (With<ProxyBillboardText>, Without<ProxyVisualRing>, Without<ProxyVisualCore>)>,
        query_cam: Query<&Transform, (With<PlayerCamera>, Without<ProxyBillboardText>)>,
    )
    ```
  - [ ] Rotate summoning ground rings:
    - For each `(mut transform, ring)`:
      `transform.rotate_y(time.delta_secs() * ring.spin_speed);`
  - [ ] Counter-tumble floating crystalline core:
    - For each `(mut transform, core)`:
      ```rust
      let dt = time.delta_secs();
      transform.rotate_local_x(dt * core.tumble_axes.x);
      transform.rotate_local_y(dt * core.tumble_axes.y);
      transform.rotate_local_z(dt * core.tumble_axes.z);
      ```
  - [ ] Modulate point light intensity with smooth sine wave breathing:
    - For each `(mut light, pulser)`:
      ```rust
      let wave = ((time.elapsed_secs() * pulser.frequency).sin() * 0.5 + 0.5);
      light.intensity = pulser.base_intensity + wave * pulser.pulse_amplitude;
      ```
  - [ ] Billboard alignment to player camera:
    - Fetch player camera transform: `let cam_transform = query_cam.single()`.
    - For each `mut billboard_transform`:
      ```rust
      let to_cam = (cam_transform.translation - billboard_transform.translation).normalize_or_zero();
      let yaw = to_cam.x.atan2(to_cam.z);
      billboard_transform.rotation = Quat::from_rotation_y(yaw);
      ```
  - [ ] Fail-Visible Reaction & Auto-Despawn Lifecycle:
    - For each `(entity, mut proxy)`:
      - If `proxy.failed == true`:
        - If let Some(ref mut timer) = proxy.despawn_timer:
          - Tick timer: `timer.tick(time.delta());`
          - When timer finished:
            `commands.entity(entity).despawn_recursive();`

#### Step 4.4 — Pedestal Snapping Engine & Spatial Grounding Alignment
- [ ] Action: Implement `resolve_manifest_target` in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] Function signature:
    ```rust
    pub fn resolve_manifest_target(target_ground: Vec3, chamber_index: usize) -> Transform
    ```
  - [ ] Canonical chamber focal points:
    - Look up `spec = &council_chambers()[chamber_index]`.
    - Define chamber altar center: `let pedestal_center = spec.origin + Vec3::new(0.0, 0.40, 0.0);`
    - Define plinth magnetic snap radius: `const SNAP_RADIUS: f32 = 2.8;`
  - [ ] Distance threshold evaluation:
    - Compute 2D horizontal distance:
      ```rust
      let delta = Vec3::new(target_ground.x - pedestal_center.x, 0.0, target_ground.z - pedestal_center.z);
      let distance = delta.length();
      ```
  - [ ] Snap or Freeform Branch:
    - **Within Plinth Radius (`distance <= SNAP_RADIUS`):**
      - Snap translation directly to plinth center: `let snapped_pos = pedestal_center;`.
      - Orient facing outward toward central hub:
        ```rust
        let to_hub = (Vec3::ZERO - spec.origin).normalize_or(Vec3::Z);
        let facing_yaw = to_hub.x.atan2(to_hub.z);
        let snapped_rot = Quat::from_rotation_y(facing_yaw);
        Transform::from_translation(snapped_pos).with_rotation(snapped_rot)
        ```
    - **Outside Plinth Radius (`distance > SNAP_RADIUS`):**
      - Retain freeform player placement grounded at floor:
        ```rust
        let grounded_pos = Vec3::new(target_ground.x, 0.0, target_ground.z);
        let to_origin = (spec.origin - grounded_pos).normalize_or(Vec3::Z);
        let free_rot = Quat::from_rotation_y(to_origin.x.atan2(to_origin.z));
        Transform::from_translation(grounded_pos).with_rotation(free_rot)
        ```

#### Step 4.5 — Archetype-Themed PBR Material Generation Matrix
- [ ] Action: Implement `build_archetype_material` in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] Function signature:
    ```rust
    pub fn build_archetype_material(
        archetype: Archetype,
        has_vertex_colors: bool,
        materials: &mut Assets<StandardMaterial>,
    ) -> Handle<StandardMaterial>
    ```
  - [ ] Query archetype theme tokens: `let theme = archetype.theme();`.
  - [ ] Full 7-Archetype PBR Parameter Matrix:
    - If `has_vertex_colors == true`:
      - Set `base_color: Color::WHITE` to allow TripoSR per-vertex colors (`Mesh::ATTRIBUTE_COLOR`) to render without muddying or color tint cancellation.
      - Apply distinctive archetype surface textures:
        - **Architect (Rational Blueprint):** `metallic: 0.20, perceptual_roughness: 0.15, emissive: LinearRgba::from(theme.accent_primary) * 0.12` (Polished drafting marble).
        - **Sentinel (Obsidian Aegis):** `metallic: 0.88, perceptual_roughness: 0.22, emissive: LinearRgba::from(theme.accent_primary) * 0.08` (Reflective titanium-obsidian armor).
        - **Mentor (Scholastic Relic):** `metallic: 0.72, perceptual_roughness: 0.38, emissive: LinearRgba::from(theme.accent_primary) * 0.10` (Antique brushed bronze & gold leaf).
        - **Explorer (Expedition Brass):** `metallic: 0.45, perceptual_roughness: 0.60, emissive: LinearRgba::from(theme.accent_primary) * 0.06` (Weathered field compass alloy).
        - **Oracle (Astral Crystalline):** `metallic: 0.28, perceptual_roughness: 0.08, emissive: LinearRgba::from(theme.accent_primary) * 0.28` (Starlit astral glass).
        - **Empath (Living Chalcedony):** `metallic: 0.08, perceptual_roughness: 0.40, emissive: LinearRgba::from(theme.accent_primary) * 0.16` (Soft luminous rose quartz).
        - **Jester (Quicksilver Chrome):** `metallic: 0.96, perceptual_roughness: 0.04, emissive: LinearRgba::from(theme.accent_primary) * 0.22` (Mirror-sheen iridescent liquid chrome).
    - If `has_vertex_colors == false`:
      - Set `base_color: theme.accent_primary` (monochrome fall-back).
      - Maintain identical metallic and roughness settings.

#### Step 4.6 — Asynchronous Result Polling, Mesh Ingestion & Arrival Burst VFX
- [ ] Action: Implement `poll_manifestation_results` in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] System signature:
    ```rust
    pub fn poll_manifestation_results(
        mut commands: Commands,
        channel: Res<Chronos2BridgeChannel>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut query_proxies: Query<(Entity, &mut SummoningProxy, &Transform)>,
        mut query_billboard_texts: Query<(&Parent, &mut Text, &mut TextColor), With<ProxyBillboardText>>,
        mut query_lights: Query<(&Parent, &mut PointLight), With<ProxyPulsingLight>>,
        time: Res<Time>,
    )
    ```
  - [ ] Non-blocking channel drain:
    ```rust
    let Ok(guard) = channel.receiver.lock() else { return; };
    while let Ok(result_res) = guard.try_recv() {
        match result_res {
            Ok(manifest_result) => {
                // Success: find matching proxy by prompt & chamber index
                if let Some((proxy_entity, _, proxy_transform)) = query_proxies.iter().find(|(_, p, _)| {
                    p.prompt == manifest_result.prompt && p.chamber_index == manifest_result.chamber_index
                }) {
                    // 1. Parse OBJ mesh via native loader
                    match load_obj_from_path(&manifest_result.mesh_path, 1.6) {
                        Ok(mesh) => {
                            let has_colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some();
                            let mesh_handle = meshes.add(mesh);
                            let spec = &council_chambers()[manifest_result.chamber_index];
                            let mat_handle = build_archetype_material(spec.archetype, has_colors, &mut materials);

                            // 2. Despawn proxy hierarchy cleanly
                            commands.entity(proxy_entity).despawn_recursive();

                            // 3. Spawn permanent ManifestedEntity
                            let entity_pos = proxy_transform.translation;
                            let entity_rot = proxy_transform.rotation;
                            commands.spawn((
                                Mesh3d(mesh_handle),
                                MeshMaterial3d(mat_handle),
                                Transform::from_translation(entity_pos).with_rotation(entity_rot),
                                ManifestedEntity {
                                    prompt: manifest_result.prompt.clone(),
                                    chamber_index: manifest_result.chamber_index,
                                    mesh_path: manifest_result.mesh_path.clone(),
                                    sha256: manifest_result.sha256.clone(),
                                    spawned_at: time.elapsed_secs_f64(),
                                    vertices: manifest_result.vertices,
                                    faces: manifest_result.faces,
                                },
                                InnerWorldElement,
                                Name::new(format!("Manifested_{}", manifest_result.prompt)),
                            ));

                            // 4. Spawn dramatic arrival light burst
                            commands.spawn((
                                PointLight {
                                    intensity: 140_000.0,
                                    range: 12.0,
                                    color: Color::srgb(1.0, 0.96, 0.88),
                                    shadows_enabled: false,
                                    ..default()
                                },
                                Transform::from_translation(entity_pos + Vec3::Y * 1.0),
                                ArrivalBurstLight {
                                    timer: Timer::from_seconds(1.2, TimerMode::Once),
                                    initial_intensity: 140_000.0,
                                },
                                InnerWorldElement,
                            ));
                        }
                        Err(e) => {
                            // Parsing error: fail-visible transition
                            if let Some((_, mut proxy, _)) = query_proxies.iter_mut().find(|(_, p, _)| {
                                p.prompt == manifest_result.prompt && p.chamber_index == manifest_result.chamber_index
                            }) {
                                proxy.mark_failed(format!("OBJ Parse Error: {e}"));
                            }
                        }
                    }
                }
            }
            Err(err) => {
                // Subprocess failure / timeout: transition proxy to fail-visible state
                for (proxy_entity, mut proxy, _) in query_proxies.iter_mut() {
                    if !proxy.failed {
                        proxy.mark_failed(err.to_string());

                        // Update billboard text to red error warning
                        for (parent, mut text, mut color) in query_billboard_texts.iter_mut() {
                            if parent.get() == proxy_entity {
                                text.0 = format!("FAILED: {}", err);
                                color.0 = Color::srgb(1.0, 0.25, 0.15);
                            }
                        }
                        // Update light to static emergency beacon
                        for (parent, mut light) in query_lights.iter_mut() {
                            if parent.get() == proxy_entity {
                                light.color = Color::srgb(1.0, 0.22, 0.12);
                                light.intensity = 70_000.0;
                            }
                        }
                    }
                }
            }
        }
    }
    ```
  - [ ] Implement `tick_arrival_bursts` in `Update`:
    ```rust
    pub fn tick_arrival_bursts(
        mut commands: Commands,
        time: Res<Time>,
        mut query: Query<(Entity, &mut PointLight, &mut ArrivalBurstLight)>,
    ) {
        for (entity, mut light, mut burst) in query.iter_mut() {
            burst.timer.tick(time.delta());
            let remaining_fraction = 1.0 - burst.timer.fraction();
            light.intensity = burst.initial_intensity * remaining_fraction;
            if burst.timer.finished() {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
    ```

#### Step 4.7 — Exhaustive Unit Test Suite (7 Pure Test Cases)
- [ ] Action: Write pure unit tests under `#[cfg(test)]` in `crates/engine/src/modes/inner_chambers/world.rs`:
  - [ ] **Test 1 (`pedestal_snapping_within_threshold_aligns_center_and_facing`):**
    Given target position `Vec3::new(1.0, 0.0, 1.0)` within 1.5m of Architect plinth center:
    Assert returned transform translation exactly equals `pedestal_center`.
    Assert returned rotation quaternion aligns facing toward `Vec3::ZERO` (hub).
  - [ ] **Test 2 (`pedestal_snapping_outside_threshold_preserves_ground_plane`):**
    Given target position `Vec3::new(6.0, 0.0, 6.0)` (5.5m away from plinth):
    Assert returned transform translation has `x = 6.0`, `y = 0.0` (grounded), `z = 6.0`.
  - [ ] **Test 3 (`archetype_pbr_material_table_coverage`):**
    Iterate all 7 variants of `Archetype`. Construct material with `has_vertex_colors: true`.
    Assert `metallic` and `perceptual_roughness` match design specifications for all 7 variants.
  - [ ] **Test 4 (`vertex_color_preservation_avoids_tint_mud`):**
    For `Archetype::Mentor`, assert `has_vertex_colors: true` produces `base_color == Color::WHITE`.
    Assert `has_vertex_colors: false` produces `base_color == theme.accent_primary`.
  - [ ] **Test 5 (`fail_visible_proxy_state_transition`):**
    Create `SummoningProxy::new(...)`. Call `proxy.mark_failed("CUDA Out of Memory".into())`.
    Assert `proxy.failed == true`, `proxy.error_msg == Some("CUDA Out of Memory".into())`, and `despawn_timer` is initialized to 6.0s.
  - [ ] **Test 6 (`arrival_burst_light_decay_math`):**
    Create `ArrivalBurstLight` with 140,000.0 initial intensity and 1.2s duration.
    Tick timer forward by 0.6s (fraction = 0.5).
    Assert remaining intensity calculates to exactly `70_000.0`.
  - [ ] **Test 7 (`proxy_child_hierarchy_invariants`):**
    Verify child components: `ProxyVisualRing`, `ProxyVisualCore`, `ProxyBillboardText`, and `ProxyPulsingLight` all instantiate with valid defaults and non-zero parameters.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/modes/inner_chambers/extraction.rs`
- Expected outcome: Player submits a noun -> sees a summoning hologram -> model loads and replaces hologram with archetype styling.

### Step 5 — World Memory Persistence & Ledger Sealing (Detailed Checklist)

This checklist breaks down Phase 5 into granular, testable engineering tasks. Each step implements persistent session storage for manifested 3D geometry and guarantees tamper-evident auditability via hash-chained ledger events sealed through the Sentinel capability mediator.

#### Step 5.1 — Persistent Manifestation Data Schema & AppData Models
- [ ] Action: Define serialization models in `crates/engine/src/modes/inner_chambers/seed.rs`:
  - [ ] Implement `PersistentTransform`:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PersistentTransform {
        pub translation: [f32; 3],
        pub rotation: [f32; 4], // [x, y, z, w]
        pub scale: [f32; 3],
    }

    impl From<Transform> for PersistentTransform {
        fn from(t: Transform) -> Self {
            Self {
                translation: [t.translation.x, t.translation.y, t.translation.z],
                rotation: [t.rotation.x, t.rotation.y, t.rotation.z, t.rotation.w],
                scale: [t.scale.x, t.scale.y, t.scale.z],
            }
        }
    }

    impl From<&PersistentTransform> for Transform {
        fn from(pt: &PersistentTransform) -> Self {
            Transform {
                translation: Vec3::from_array(pt.translation),
                rotation: Quat::from_array(pt.rotation),
                scale: Vec3::from_array(pt.scale),
            }
        }
    }
    ```
  - [ ] Implement `PersistentManifestedObject`:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct PersistentManifestedObject {
        pub id: String,
        pub prompt: String,
        pub chamber_index: usize,
        pub archetype_name: String,
        pub mesh_path: String,
        pub sha256: String,
        pub transform: PersistentTransform,
        pub spawned_at_utc: String,
        pub vertices: usize,
        pub faces: usize,
    }
    ```
  - [ ] Implement container `ChamberManifestationStore`:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
    pub struct ChamberManifestationStore {
        pub version: u32,
        pub objects: Vec<PersistentManifestedObject>,
    }
    ```
  - [ ] Implement storage path resolver:
    ```rust
    pub fn manifestation_store_path() -> std::path::PathBuf {
        app_data_root().join("inner_chambers").join("manifested_objects.json")
    }
    ```

#### Step 5.2 — Atomic Disk Persistence Engine
- [ ] Action: Implement robust, crash-resilient disk I/O in `crates/engine/src/modes/inner_chambers/seed.rs`:
  - [ ] Implement `load_manifested_objects() -> Result<Vec<PersistentManifestedObject>, String>`:
    - Path check: `let path = manifestation_store_path();`
    - If `!path.exists()`, return `Ok(Vec::new())`.
    - Read string via `std::fs::read_to_string(&path)`.
    - Deserialize via `serde_json::from_str::<ChamberManifestationStore>(&content)`.
    - Gracefully handle schema migrations if `store.version < CURRENT_VERSION`.
    - Return `Ok(store.objects)`.
  - [ ] Implement `save_manifested_object(object: PersistentManifestedObject) -> Result<(), String>`:
    - Resolve path: `let path = manifestation_store_path();`
    - Ensure directory structure exists: `std::fs::create_dir_all(path.parent().unwrap())`.
    - Load existing store: `let mut objects = load_manifested_objects().unwrap_or_default();`
    - Upsert by ID:
      ```rust
      if let Some(existing) = objects.iter_mut().find(|o| o.id == object.id) {
          *existing = object;
      } else {
          objects.push(object);
      }
      let store = ChamberManifestationStore { version: 1, objects };
      ```
    - Atomic File Write via `.tmp` swap:
      ```rust
      let tmp_path = path.with_extension("tmp");
      let serialized = serde_json::to_string_pretty(&store).map_err(|e| e.to_string())?;
      std::fs::write(&tmp_path, serialized.as_bytes()).map_err(|e| e.to_string())?;
      std::fs::rename(&tmp_path, &path).map_err(|e| e.to_string())?;
      ```

#### Step 5.3 — Cryptographic Hash-Chained Ledger Sealing
- [ ] Action: Implement ledger event serialization and Sentinel capability mediation:
  - [ ] Event Type: `"adlib_object_manifested"`.
  - [ ] Implement payload generator `manifestation_ledger_payload(obj: &PersistentManifestedObject) -> serde_json::Value`:
    ```rust
    pub fn manifestation_ledger_payload(obj: &PersistentManifestedObject) -> serde_json::Value {
        serde_json::json!({
            "schema": "archetypes.inner_chambers.adlib_manifested.v1",
            "object_id": obj.id,
            "prompt": obj.prompt,
            "chamber_index": obj.chamber_index,
            "archetype": obj.archetype_name,
            "mesh_path": obj.mesh_path,
            "sha256": obj.sha256,
            "vertices": obj.vertices,
            "faces": obj.faces,
            "transform": {
                "translation": obj.transform.translation,
                "rotation": obj.transform.rotation,
                "scale": obj.transform.scale,
            },
            "spawned_at_utc": obj.spawned_at_utc,
        })
    }
    ```
  - [ ] Implement `seal_manifestation_to_ledger(obj: &PersistentManifestedObject) -> Result<(), String>`:
    - Construct payload using `manifestation_ledger_payload(obj)`.
    - Dispatch to canonical ledger:
      ```rust
      append_to_ledger(
          GameMode::InnerChambers,
          "adlib_object_manifested",
          payload,
      )
      ```
    - Sentinel mediation: Verify that `sentinel::mediate("memory.write", "archetypes://ledger", ...)` validates the transaction before appending.
    - Chained hash computation: Verify entry contains `previous_hash` pointing to predecessor and valid SHA-256 digest in `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\ledger.jsonl`.

#### Step 5.4 — Runtime Manifestation Hook Integration
- [ ] Action: Hook persistence and ledger sealing into `poll_manifestation_results` in `world.rs`:
  - [ ] Immediately after spawning `ManifestedEntity`:
    - Generate unique artifact ID: `let object_id = uuid::Uuid::new_v4().to_string();`
    - Timestamp in UTC: `let timestamp = chrono::Utc::now().to_rfc3339();`
    - Construct `PersistentManifestedObject`:
      ```rust
      let spec = &council_chambers()[manifest_result.chamber_index];
      let persistent_record = PersistentManifestedObject {
          id: object_id,
          prompt: manifest_result.prompt.clone(),
          chamber_index: manifest_result.chamber_index,
          archetype_name: spec.archetype.theme().name.to_string(),
          mesh_path: manifest_result.mesh_path.to_string_lossy().to_string(),
          sha256: manifest_result.sha256.clone(),
          transform: PersistentTransform::from(Transform::from_translation(entity_pos).with_rotation(entity_rot)),
          spawned_at_utc: timestamp,
          vertices: manifest_result.vertices,
          faces: manifest_result.faces,
      };
      ```
    - Save to disk:
      ```rust
      if let Err(e) = save_manifested_object(persistent_record.clone()) {
          warn!("Failed to persist manifested object to disk store: {e}");
      }
      ```
    - Seal to ledger:
      ```rust
      if let Err(e) = seal_manifestation_to_ledger(&persistent_record) {
          warn!("Failed to seal manifestation event to ledger: {e}");
      }
      ```

#### Step 5.5 — Chamber World Initialization & Object Reload Loop
- [ ] Action: Modify `setup_inner_world` on `OnEnter(InnerChambersState::Loading)` in `world.rs`:
  - [ ] Load stored artifacts: `let saved_objects = load_manifested_objects().unwrap_or_default();`
  - [ ] Iterate through `saved_objects`:
    - Check if mesh file exists: `let mesh_path = std::path::Path::new(&obj.mesh_path);`
    - If `!mesh_path.exists()`, log warning: `warn!("Manifested mesh missing from disk at {:?}, skipping", mesh_path);` and continue.
    - Parse mesh: `let Ok(mesh) = load_obj_from_path(mesh_path, 1.6) else { continue; };`
    - Check for vertex colors: `let has_colors = mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_some();`
    - Register mesh: `let mesh_handle = meshes.add(mesh);`
    - Generate material:
      ```rust
      let spec = &council_chambers()[obj.chamber_index];
      let mat_handle = build_archetype_material(spec.archetype, has_colors, &mut materials);
      ```
    - Spawn entity into world:
      ```rust
      let spawn_transform = Transform::from(&obj.transform);
      commands.spawn((
          Mesh3d(mesh_handle),
          MeshMaterial3d(mat_handle),
          spawn_transform,
          ManifestedEntity {
              prompt: obj.prompt.clone(),
              chamber_index: obj.chamber_index,
              mesh_path: std::path::PathBuf::from(&obj.mesh_path),
              sha256: obj.sha256.clone(),
              spawned_at: 0.0,
              vertices: obj.vertices,
              faces: obj.faces,
          },
          InnerWorldElement,
          Name::new(format!("Restored_{}_{}", obj.chamber_index, obj.prompt)),
      ));
      ```
  - [ ] Log summary: `info!("Restored {} manifested 3D objects across Inner Chambers.", count);`

#### Step 5.6 — Ledger Audit Trail Verification & Integrity Validator
- [ ] Action: Implement `verify_manifestation_audit_trail` in `crates/engine/src/services/ledger.rs`:
  - [ ] Function signature:
    ```rust
    pub fn verify_manifestation_audit_trail() -> Result<ManifestationAuditReport, String>
    ```
  - [ ] Struct `ManifestationAuditReport`:
    ```rust
    #[derive(Debug, Clone, PartialEq)]
    pub struct ManifestationAuditReport {
        pub total_entries_verified: usize,
        pub manifestation_events_count: usize,
        pub unbroken_hash_chain: bool,
        pub matched_store_records: usize,
    }
    ```
  - [ ] Verify unbroken cryptographic hash chain from entry 0 to tip.
  - [ ] Filter entries with `mode == "inner_chambers"` and `kind == "adlib_object_manifested"`.
  - [ ] Cross-reference `object_id` and `sha256` against `manifested_objects.json` to prove zero-loss auditability.

#### Step 5.7 — Exhaustive Unit Test Suite (7 Pure Test Cases)
- [ ] Action: Write pure unit tests under `#[cfg(test)]` in `crates/engine/src/modes/inner_chambers/seed.rs`:
  - [ ] **Test 1 (`persistent_transform_serde_roundtrip`):**
    Construct `PersistentTransform` with non-trivial values. Serialize to JSON string, deserialize back, assert exact float array match.
  - [ ] **Test 2 (`store_json_serialization_roundtrip`):**
    Populate `ChamberManifestationStore` with multiple objects across chambers. Serialize to string, deserialize back, assert identical vector length and fields.
  - [ ] **Test 3 (`atomic_write_creates_valid_store_file`):**
    Write to a mock temp file using `save_manifested_object`. Assert `.tmp` is cleaned up and target `.json` file exists and is valid JSON.
  - [ ] **Test 4 (`store_append_and_id_deduplication`):**
    Save object with ID `"test-uuid-1"`. Save updated position with same ID -> assert count is 1. Save with `"test-uuid-2"` -> assert count is 2.
  - [ ] **Test 5 (`manifestation_ledger_payload_schema_conformance`):**
    Build payload using `manifestation_ledger_payload`. Assert all 11 required keys (`schema`, `object_id`, `prompt`, `chamber_index`, `archetype`, `mesh_path`, `sha256`, `vertices`, `faces`, `transform`, `spawned_at_utc`) exist and have correct types.
  - [ ] **Test 6 (`ledger_hash_chain_integrity_with_manifestation_events`):**
    Append simulated `"adlib_object_manifested"` event to a temporary ledger file using `append_to_path`. Call `verify_ledger_path` and assert unbroken SHA-256 chain.
  - [ ] **Test 7 (`missing_mesh_file_graceful_handling`):**
    Pass a stored record pointing to a non-existent `/tmp/missing_file.obj` to the restore loop. Assert that system logs a warning, skips cleanly, and does not panic.
- Files touched:
  - `crates/engine/src/modes/inner_chambers/seed.rs`
  - `crates/engine/src/modes/inner_chambers/world.rs`
  - `crates/engine/src/services/ledger.rs`
- Expected outcome: Manifested objects persist across game restarts and leave a verifiable audit trail in the ledger.

### Step 6 — Workspace Verification & Desktop Staging (Detailed Checklist)

This checklist breaks down Phase 6 into granular, testable engineering tasks. Each step enforces AGENTS.md verification laws (Section 3), ensures zero regressions across the dual-binary Rust workspace, restages the self-contained Desktop distribution, and verifies the live end-to-end player experience.

#### Step 6.1 — Static Analysis & Compilation Gate (`cargo check` & `cargo clippy`)
- [ ] Action: Run full static analysis checks across the entire dual-binary workspace:
  - [ ] Workspace type checking:
    ```pwsh
    cargo check --workspace --all-targets
    ```
    Assert zero compile errors across `crates/engine`, `crates/launcher`, and tests.
  - [ ] Workspace linter enforcement:
    ```pwsh
    cargo clippy --workspace --all-targets -- -D warnings
    ```
    Assert zero compiler warnings, zero unused imports, zero dead-code warnings, and zero Clippy lints.

#### Step 6.2 — Full Workspace Test Gate (`cargo test --workspace`)
- [ ] Action: Execute the complete headless automated test suite:
  ```pwsh
  cargo test --workspace
  ```
  - [ ] **Phase 1 Tests (`crates/engine/src/modes/inner_chambers/mesh_loader.rs`):**
    - `test_empty_and_degenerate_mesh_guard` — asserts error on empty file or zero-height geometry.
    - `test_triposr_z_to_bevy_y_coordinate_swizzle` — asserts `[x, z, -y]` mapping.
    - `test_aabb_normalization_and_grounding` — asserts base `y = 0.0` and height clamped to 1.6m.
    - `test_polygon_quad_fan_triangulation` — asserts 4-vertex quad converts to 2 valid triangles.
    - `test_triposr_inline_rgb_color_extraction` — asserts `v x y z r g b` maps to `Mesh::ATTRIBUTE_COLOR`.
    - `test_surface_normal_area_weighted_generation` — asserts calculated normals point outward with unit length.
    - `test_live_triposr_mesh_fixture_parsing` — asserts real 100-line TripoSR snippet parses cleanly.
  - [ ] **Phase 2 Tests (`crates/engine/src/services/chronos2_bridge.rs`):**
    - `test_resolve_chronos2_executable_precedence` — asserts env var, sibling, and AppData search precedence.
    - `test_build_chronos2_command_flags_and_no_window` — asserts `--geometry-forge --void` and `CREATE_NO_WINDOW`.
    - `test_parse_and_validate_triposr_receipt` — asserts schema `chronosophia.triposr-mesh.v1` deserialization.
    - `test_worker_supervisor_timeout_trigger` — asserts 60s timeout kills child and emits `TimedOut`.
    - `test_missing_mesh_file_error_emission` — asserts error if receipt references nonexistent mesh.
  - [ ] **Phase 3 Tests (`crates/engine/src/modes/inner_chambers/manifest_ui.rs`):**
    - `test_manifest_prompt_buffer_push_and_backspace` — asserts string editing dynamics.
    - `test_manifest_prompt_buffer_max_length_and_sanitization` — asserts 40-char cap and control char rejection.
    - `test_empty_submission_validation_error` — asserts whitespace input shows error and keeps state.
    - `test_grounded_target_calculation_from_camera` — asserts forward-projected ground coordinate calculation.
    - `test_escape_key_clears_and_returns_to_navigating` — asserts cancel returns camera look control.
  - [ ] **Phase 4 Tests (`crates/engine/src/modes/inner_chambers/world.rs`):**
    - `test_pedestal_snapping_within_magnetic_radius` — asserts snap to plinth within 2.8m and outward facing.
    - `test_pedestal_snapping_outside_threshold_preserves_ground` — asserts freeform floor placement at >2.8m.
    - `test_archetype_pbr_material_generation_matrix` — asserts unique metallic/roughness across all 7 archetypes.
    - `test_vertex_color_preservation_with_white_base` — asserts `base_color == WHITE` when vertex colors exist.
    - `test_fail_visible_proxy_amber_alert_transition` — asserts error message display and 6s timer.
    - `test_arrival_burst_light_decay_linear_math` — asserts 140k lm flash fades linearly to zero over 1.2s.
    - `test_proxy_child_hierarchy_invariants` — asserts rings, core, light, and billboard marker components.
  - [ ] **Phase 5 Tests (`crates/engine/src/modes/inner_chambers/seed.rs` & `ledger.rs`):**
    - `test_persistent_transform_serde_roundtrip` — asserts precision float roundtrip.
    - `test_store_json_serialization_roundtrip` — asserts multiobject store serialization.
    - `test_atomic_write_creates_valid_store_file` — asserts `.tmp` swap pattern prevents file corruption.
    - `test_store_append_and_id_deduplication` — asserts idempotent upsert by UUID.
    - `test_manifestation_ledger_payload_schema_conformance` — asserts 11 schema keys present and typed.
    - `test_ledger_hash_chain_integrity_with_manifestation_events` — asserts unbroken SHA-256 chain.
    - `test_missing_mesh_file_graceful_handling` — asserts missing mesh gracefully skips without crashing.
  - [ ] **Existing Workspace Tests:**
    - Standard Mecha chat tests pass.
    - Oracle Riddle scoring and clue pool tests pass.
    - Living Engine metabolic sim tests pass.
    - Launcher Windows identity and single-instance tests pass.

#### Step 6.3 — Release Packaging & Binary Compilation
- [ ] Action: Build optimized release binaries for the entire workspace:
  ```pwsh
  cargo build --release --workspace
  ```
  - [ ] Confirm successful compilation of `target\release\engine.exe` and `target\release\launcher.exe`.
  - [ ] Assert PE binary timestamp is strictly newer than modified source files.
  - [ ] Confirm embedded application icon (`assets/icons/archetypes.ico`) via Windows PE resource headers.

#### Step 6.4 — Desktop Surface Restaging (`scripts\install_shortcut.ps1`)
- [ ] Action: Execute the canonical desktop distribution restager:
  ```pwsh
  pwsh -File scripts\install_shortcut.ps1
  ```
  - [ ] **Distribution Tree Update (`C:\archetypes\dist\`):**
    - Verify `dist\engine.exe` is updated and matches `target\release\engine.exe`.
    - Verify `dist\launcher.exe` is updated and matches `target\release\launcher.exe`.
    - Verify `dist\archetypes.ico` is copied from `assets\icons\archetypes.ico`.
    - Verify `dist\assets\` is recursively synced while preserving `dist\assets\standard_mecha\renders\`.
  - [ ] **Offline Voice Dependency Check:**
    - Confirms `scripts\setup_windows.ps1 -InstallRoot $DistRoot -NonInteractive` verifies Kokoro models and sherpa-onnx runtime.
  - [ ] **Shortcut Verification (Rule 13 Compliance):**
    - Desktop shortcut: `[Environment]::GetFolderPath('Desktop')\Archetypes.lnk` -> points to `C:\archetypes\dist\launcher.exe`.
    - Start Menu shortcut: `Programs\NeuroCognica\Archetypes.lnk` -> points to `C:\archetypes\dist\launcher.exe`.
    - Working directory: `C:\archetypes\dist`.

#### Step 6.5 — End-to-End Live Launch & Manifestation Verification
- [ ] Action: Drive the live application through the Desktop entry point (Rule 10 & 13):
  - [ ] Launch `dist\launcher.exe` (or click Desktop `Archetypes.lnk`).
  - [ ] Verify Launcher supervisor checks: Ollama, Chronos Director, and offline TTS readiness pass.
  - [ ] Verify Engine opens maximized with no terminal window flashing (`CREATE_NO_WINDOW` / `windows_subsystem = "windows"`).
  - [ ] Navigate from Lore Menu into **Inner Chambers** (`GameMode::InnerChambers`).
  - [ ] Walk into Architect chamber:
    - Press `KeyT` -> Verify in-chamber manifest UI appears, camera rotation locks, gold cursor blinks.
    - Type `"chalice"` -> Verify characters appear in buffer.
    - Press `Enter` -> Verify UI closes, camera look resumes, and summoning hologram proxy spawns at ground target.
  - [ ] Verify proxy animation:
    - Gold outer ring rotates clockwise, inner ring counter-clockwise.
    - Crystal core tumbles on 3 axes.
    - Archetype point light breathes with sine wave.
    - Status billboard faces player camera displaying `"FORGING 'CHALICE' IN TRIPOSR..."`.
    - Bevy engine frame rate remains at solid 60 FPS while Chronos2 runs on background OS thread.
  - [ ] Verify arrival swap:
    - TripoSR generates mesh -> proxy recursively despawns.
    - Final 3D chalice materializes on chamber pedestal with Architect polished drafting marble PBR shader.
    - 140,000 lm arrival light burst flashes and decays smoothly over 1.2s.
  - [ ] Verify persistence across sessions:
    - Press `Esc` to return to Lore Menu.
    - Re-enter Inner Chambers -> Verify chalice is restored from disk at the exact pedestal coordinate.

#### Step 6.6 — Ledger Audit Verification & SHA-256 Proof
- [ ] Action: Verify cryptographic ledger integrity:
  - [ ] Inspect `%LOCALAPPDATA%\NeuroCognica\Archetypes\data\ledger.jsonl`.
  - [ ] Verify unbroken SHA-256 hash chaining from genesis to tip.
  - [ ] Confirm latest entry has `mode == "inner_chambers"`, `kind == "adlib_object_manifested"`, and schema `archetypes.inner_chambers.adlib_manifested.v1`.
  - [ ] Confirm `sha256` in ledger matches actual hash of generated `.obj` mesh.

#### Step 6.7 — Documentation, STATUS.md & Clean Tree Sign-Off
- [ ] Action: Complete AGENTS.md documentation compliance (Rule 4, 11):
  - [ ] Update `STATUS.md` recording feature completion, passing verification gates, and release timestamp.
  - [ ] Update `docs/ledger/CURRENT_HANDOFF.md`.
  - [ ] Mark plan documents `COMPLETED`.
  - [ ] Verify `git status` is completely clean:
    ```pwsh
    git status
    ```
  - [ ] Push immediately to `origin/main`:
    ```pwsh
    git push origin main
    ```
- Files touched:
  - `dist/` (gitignored restage)
  - `STATUS.md`
  - `docs/ledger/CURRENT_HANDOFF.md`
- Expected outcome: Full passing gates, clean working tree, and verified Desktop ad-lib 3D generation.

---

## 5. Risk Analysis & Safety Fences

1. **VRAM Contention (RTX 3060 12GB):**
   * *Risk:* TripoSR uses PyTorch / CUDA, while Bevy runs DirectX12 / Vulkan. Concurrent memory spikes could cause out-of-memory errors.
   * *Mitigation:* Chronos2 already manages VRAM offloading (`--void` avoids spinning up ComfyUI canvas pipelines; TripoSR feedforward inference uses ~3.2 GB peak). Archetypes runs in low-VRAM mode during Inner Chambers.
2. **Subprocess Fail-Visible (Rule 8):**
   * *Risk:* If `chronos.exe` fails, crashes, or is missing weights, the player could be left waiting indefinitely.
   * *Mitigation:* The bridge enforces a 45-second execution timeout. If the subprocess exits non-zero or times out, the summoning proxy shifts from blue to warning amber, displays the failure reason directly on screen, and despawns after 5 seconds.
3. **Scale & Geometry Degeneracy:**
   * *Risk:* A generated mesh with inverted normals or extreme dimensions could block the camera or be invisible.
   * *Mitigation:* `mesh_loader.rs` enforces strict AABB normalization (max dimension clamped to 1.6m) and recalculates vertex normals if the file lacks normal vectors.

---

## 6. Verification Records (To be filled during execution)
- `cargo test --workspace`:
- Mesh Loader Unit Tests:
- Chronos2 Bridge Mock Tests:
- Desktop Restage:
- Live Generation Frame Evidence:
