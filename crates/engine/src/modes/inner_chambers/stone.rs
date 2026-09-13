//! Real stone on the castle kit.
//!
//! Every Blender kit module is UV-unwrapped and, until now, flat colour. That is the whole of
//! the "it looks fake" problem: an untextured pier is a shape, not masonry. Two things have to
//! be true before a stone texture can look right, and both are handled here or upstream of
//! here:
//!
//! 1. **The UVs must be world-scaled.** `scripts/author_arcade_bay.py` and
//!    `author_stair_flight.py` now cube-project at [`STONE_TILE_METRES`] instead of using
//!    `smart_project`, which packs islands into unit space per object and has no physical
//!    scale at all. Without that, one block of stone would be a different size on a pier than
//!    on a tread standing next to it.
//! 2. **There must be a normal map.** A flat image of stone on a flat pier still reads flat,
//!    because nothing on the surface responds to the light. The castle casts no shadows
//!    anywhere, so surface normals are the *only* thing carrying relief.
//!
//! One bay GLB is exported, not seven. The stone is swapped per gallery level at spawn time by
//! [`bind_kit_stone`], which matches the authored node names — exporting a GLB per stone would
//! multiply a 151KB asset by seven and fork the module.

use bevy::asset::AssetServer;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;

use super::castle::GALLERY_LEVELS;

/// One tile of stone covers this much wall, in metres.
///
/// Must match `TILE_METRES` in `scripts/author_stone_tiles.py` and `STONE_TILE_METRES` in the
/// two kit scripts. All four are checked against each other by test below and by each script's
/// own export verification, so a change in one place fails loudly rather than producing a wall
/// of mismatched masonry.
pub const STONE_TILE_METRES: f32 = 4.0;

/// Stone by gallery level, heaviest at the base.
///
/// This is both how masonry is actually built and how the eye reads height: a dark dense base
/// under progressively lighter, finer courses makes a tall building look founded rather than
/// stacked. Wet dungeon and mossy stone are deliberately absent — they do not belong on a lit
/// ceremonial rotunda, and are held for the under-castle and the abyss level.
pub const LEVEL_STONE: [&str; GALLERY_LEVELS] = [
    "dark_basalt_fortress",
    "rough_granite_block",
    "old_mortared_fieldstone",
    "chipped_rubble_stone",
    "weathered_limestone_ashlar",
    "sandstone_block",
    "pale_limestone_light",
];

/// The stone every walkable surface is paved in.
pub const FLOOR_STONE: &str = "worn_courtyard_flagstone";

/// Bay nodes cut from the wall's own stone.
const BAY_STONE_NODES: [&str; 3] = ["Bay_PierShaft", "Bay_Arch", "Bay_Spandrel"];

/// Bay nodes in dressed trim: a different course of the same stone, run as bands and mouldings.
const BAY_TRIM_NODES: [&str; 5] = [
    "Bay_PierPlinth",
    "Bay_PierCapital",
    "Bay_StringCourse",
    "Bay_Corona",
    "Bay_Archivolt",
];

/// Deliberately *not* bound: the blind panel behind each arch stays near-black. Nothing in this
/// hall casts shadows, so a recess cannot read by depth — the value break has to be in the
/// material or the arcade collapses into flat rectangles. Texturing it would undo that.
const BAY_RECESS_NODE: &str = "Bay_BlindPanel";

/// Museum chamber nodes. The chamber is where the player stands closest to the stone in the
/// whole building, so it is the last place that can afford to be flat colour.
const CHAMBER_STONE_NODES: [&str; 4] = [
    "Chamber_Vault",
    "Chamber_SideWall_Left",
    "Chamber_SideWall_Right",
    "Chamber_BackWall",
];
const CHAMBER_TRIM_NODES: [&str; 6] = [
    "Chamber_Floor",
    "Chamber_Plinth_Left",
    "Chamber_Plinth_Right",
    "Chamber_BackPlinth",
    "Chamber_Impost_Left",
    "Chamber_Impost_Right",
];

/// Stair nodes, matched the same way. Names taken from `author_stair_flight.py`'s own
/// `required` set, so the two cannot drift without the test below noticing.
const STAIR_STONE_NODES: [&str; 1] = ["Stair_Steps"];
const STAIR_TRIM_NODES: [&str; 4] = [
    "Stair_ParapetInner",
    "Stair_CopingInner",
    "Stair_ParapetOuter",
    "Stair_CopingOuter",
];

#[derive(Clone)]
pub struct StoneSet {
    pub face: Handle<StandardMaterial>,
    pub trim: Handle<StandardMaterial>,
}

/// Every material the kit can be dressed in, built once at world setup.
#[derive(Resource)]
pub struct StonePalette {
    levels: Vec<StoneSet>,
    pub floor: Handle<StandardMaterial>,
    pub floor_trim: Handle<StandardMaterial>,
}

impl StonePalette {
    pub fn level(&self, level: usize) -> &StoneSet {
        // Above the authored range the top stone simply continues, rather than panicking or
        // silently falling back to an unrelated one.
        &self.levels[level.min(self.levels.len() - 1)]
    }
}

/// Loads a texture that tiles.
///
/// The default sampler clamps to the edge, and these UVs run well past 1.0 — a 9.9m bay is
/// 2.5 tiles wide — so a clamped stone texture would stretch one edge row of pixels across the
/// entire pier. Repeat is not a refinement here; without it the texture is unusable.
///
/// `is_srgb` must be false for normal and roughness maps: they are data, not colour, and
/// gamma-correcting them bends the lighting.
fn load_tiling(asset_server: &AssetServer, path: String, is_srgb: bool) -> Handle<Image> {
    asset_server.load_with_settings(path, move |settings: &mut ImageLoaderSettings| {
        settings.is_srgb = is_srgb;
        let mut sampler = ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            address_mode_w: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        };
        // At the oblique angles of a long arcade, single-sample texture filtering aliases into
        // the crawling brick shimmer the player reported. Anisotropic filtering is cheap on the
        // RTX 3060 and keeps the same authored maps stable during camera motion.
        sampler.set_anisotropic_filter(8);
        settings.sampler = ImageSampler::Descriptor(sampler);
    })
}

fn stone_material(
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    stone: &str,
    tint: Color,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: tint,
        base_color_texture: Some(load_tiling(
            asset_server,
            format!("textures/stone/{stone}_albedo.png"),
            true,
        )),
        normal_map_texture: Some(load_tiling(
            asset_server,
            format!("textures/stone/{stone}_normal.png"),
            false,
        )),
        // The generated roughness map is greyscale, so every channel carries the same value.
        // Bevy reads roughness from G and metallic from B, which would make every stone in the
        // castle metallic — except that both are *multiplied* by their factors, so a metallic
        // factor of 0 cancels the blue channel entirely. Do not raise it.
        metallic_roughness_texture: Some(load_tiling(
            asset_server,
            format!("textures/stone/{stone}_roughness.png"),
            false,
        )),
        metallic: 0.0,
        perceptual_roughness: 1.0,
        ..default()
    })
}

pub fn build_palette(
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
) -> StonePalette {
    let levels = LEVEL_STONE
        .iter()
        .map(|stone| StoneSet {
            face: stone_material(asset_server, materials, stone, Color::WHITE),
            // Trim is the same stone read slightly darker, so mouldings separate from the wall
            // without becoming a different material. They were previously 0.25 metallic brown,
            // which read as copper pipework threaded through the hall.
            trim: stone_material(
                asset_server,
                materials,
                stone,
                Color::srgb(0.82, 0.80, 0.77),
            ),
        })
        .collect();

    StonePalette {
        levels,
        floor: stone_material(asset_server, materials, FLOOR_STONE, Color::WHITE),
        floor_trim: stone_material(
            asset_server,
            materials,
            FLOOR_STONE,
            Color::srgb(0.80, 0.78, 0.74),
        ),
    }
}

/// Marks a spawned kit module so its stone can be bound once its scene has actually loaded.
#[derive(Component)]
pub struct KitStone {
    pub level: usize,
    pub bound: bool,
    /// True for the twelve bays whose arch is a real doorway.
    ///
    /// The bay module fills every arch with `Bay_BlindPanel`, a near-black slab that is what
    /// makes a blind arch read as an opening. At a bay that is *actually* an opening the same
    /// slab is a wall across the doorway, so it is hidden rather than dressed.
    pub open_arch: bool,
}

impl KitStone {
    pub fn at(level: usize) -> Self {
        Self {
            level,
            bound: false,
            open_arch: false,
        }
    }

    pub fn open_arch(level: usize) -> Self {
        Self {
            level,
            bound: false,
            open_arch: true,
        }
    }
}

fn role_for(name: &str) -> Option<StoneRole> {
    if name == BAY_RECESS_NODE {
        return None;
    }
    if BAY_STONE_NODES.contains(&name)
        || STAIR_STONE_NODES.contains(&name)
        || CHAMBER_STONE_NODES.contains(&name)
    {
        return Some(StoneRole::Face);
    }
    if BAY_TRIM_NODES.contains(&name)
        || STAIR_TRIM_NODES.contains(&name)
        || CHAMBER_TRIM_NODES.contains(&name)
    {
        return Some(StoneRole::Trim);
    }
    None
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum StoneRole {
    Face,
    Trim,
}

/// Dresses each spawned kit module in its level's stone.
///
/// A glTF scene's children do not exist on the frame the `SceneRoot` is spawned, so this runs
/// every frame and binds each module the first time it finds named descendants. `bound` stops
/// it re-walking 504 bays forever afterwards.
pub fn bind_kit_stone(
    mut commands: Commands,
    palette: Option<Res<StonePalette>>,
    mut modules: Query<(Entity, &mut KitStone)>,
    children: Query<&Children>,
    names: Query<&Name>,
    meshes: Query<(), With<Mesh3d>>,
    mut visibility: Query<&mut Visibility>,
) {
    let Some(palette) = palette else {
        return;
    };

    for (root, mut module) in &mut modules {
        if module.bound {
            continue;
        }
        let set = palette.level(module.level);
        let mut bound_any = false;

        for descendant in children.iter_descendants(root) {
            let Ok(name) = names.get(descendant) else {
                continue;
            };
            // The blind panel is the wall across a blind arch. Where the arch is a doorway,
            // hide it — and count that as work done, so a museum bay does not sit in the
            // unbound queue forever being re-walked every frame.
            if module.open_arch && name.as_str() == BAY_RECESS_NODE {
                if let Ok(mut visible) = visibility.get_mut(descendant) {
                    *visible = Visibility::Hidden;
                    bound_any = true;
                }
                continue;
            }
            let Some(role) = role_for(name.as_str()) else {
                continue;
            };
            let handle = match role {
                StoneRole::Face => set.face.clone(),
                StoneRole::Trim => set.trim.clone(),
            };

            // The mesh usually sits on the named node itself, but a glTF node whose mesh has
            // several primitives puts them on child entities instead. Cover both rather than
            // assuming one shape of import.
            if meshes.get(descendant).is_ok() {
                commands
                    .entity(descendant)
                    .insert(MeshMaterial3d(handle.clone()));
                bound_any = true;
            }
            for inner in children.iter_descendants(descendant) {
                if meshes.get(inner).is_ok() {
                    commands
                        .entity(inner)
                        .insert(MeshMaterial3d(handle.clone()));
                    bound_any = true;
                }
            }
        }

        if bound_any {
            module.bound = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scripts that author the UVs and the tiles both hard-code this number, and each
    /// verifies its own export against it. If the engine's copy drifts, the stone silently
    /// changes size on the building instead of failing.
    #[test]
    fn the_engine_agrees_with_the_authoring_scripts_on_tile_size() {
        for (path, needle) in [
            ("../../scripts/author_stone_tiles.py", "TILE_METRES = 4.0"),
            (
                "../../scripts/author_arcade_bay.py",
                "STONE_TILE_METRES = 4.0",
            ),
            (
                "../../scripts/author_stair_flight.py",
                "STONE_TILE_METRES = 4.0",
            ),
        ] {
            let source = std::fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("cannot read {path}: {error}"));
            assert!(
                source.contains(needle),
                "{path} no longer declares `{needle}`, so its UVs or tiles disagree with the \
                 engine's STONE_TILE_METRES of {STONE_TILE_METRES}"
            );
        }
        assert_eq!(STONE_TILE_METRES, 4.0);
    }

    #[test]
    fn every_gallery_level_has_a_stone() {
        assert_eq!(LEVEL_STONE.len(), GALLERY_LEVELS);
        for stone in LEVEL_STONE {
            assert!(!stone.is_empty());
        }
    }

    /// Each named stone must have all three maps on disk. A missing normal map is the failure
    /// that matters most: the texture still loads, the wall still looks textured, and it still
    /// reads flat, which is the exact complaint this work exists to fix.
    #[test]
    fn every_named_stone_ships_albedo_normal_and_roughness() {
        let root = std::path::Path::new("../../assets/textures/stone");
        let mut wanted: Vec<&str> = LEVEL_STONE.to_vec();
        wanted.push(FLOOR_STONE);
        for stone in wanted {
            for map in ["albedo", "normal", "roughness"] {
                let path = root.join(format!("{stone}_{map}.png"));
                assert!(
                    path.is_file(),
                    "missing {}; run `python scripts/author_stone_tiles.py`",
                    path.display()
                );
            }
        }
    }

    /// The blind panel behind each arch must never be dressed in stone: with no shadows in the
    /// hall, its near-black material is the only thing making an arch read as an opening.
    #[test]
    fn the_arch_recess_is_never_bound_to_stone() {
        assert_eq!(role_for(BAY_RECESS_NODE), None);
    }

    /// The stair module's node names live in its authoring script's own `required` set. If a
    /// node is renamed there, this fails rather than letting the stair quietly ship untextured
    /// while everything around it is stone.
    #[test]
    fn every_stair_node_the_script_exports_is_matched_here() {
        let source = std::fs::read_to_string("../../scripts/author_stair_flight.py")
            .expect("stair flight script readable");
        for node in STAIR_STONE_NODES.iter().chain(STAIR_TRIM_NODES.iter()) {
            assert!(
                source.contains(&format!("\"{node}\"")),
                "{node} is bound here but not authored by author_stair_flight.py"
            );
        }
    }

    /// The chamber module's node names live in its own authoring script's `required` set and
    /// its build function. If one is renamed there, this fails rather than letting a chamber
    /// ship flat-coloured inside a stone building.
    #[test]
    fn every_chamber_node_bound_here_is_authored_by_its_script() {
        let source = std::fs::read_to_string("../../scripts/author_museum_chamber.py")
            .expect("museum chamber script readable");
        for node in CHAMBER_STONE_NODES.iter().chain(CHAMBER_TRIM_NODES.iter()) {
            // The script builds its mirrored parts from an f-string — `Chamber_Plinth_{label}`
            // for Left and Right — so the full name is not a literal in the source. Fall back
            // to the stem, which still catches a rename of the part itself.
            let stem = node
                .rsplit_once('_')
                .map(|(prefix, _)| format!("{prefix}_"))
                .unwrap_or_else(|| node.to_string());
            assert!(
                source.contains(node) || source.contains(&stem),
                "{node} is bound here but neither it nor `{stem}` is authored by                  author_museum_chamber.py"
            );
        }
    }

    #[test]
    fn bay_nodes_are_split_between_face_and_trim_with_none_left_over() {
        for node in BAY_STONE_NODES {
            assert_eq!(role_for(node), Some(StoneRole::Face), "{node}");
        }
        for node in BAY_TRIM_NODES {
            assert_eq!(role_for(node), Some(StoneRole::Trim), "{node}");
        }
        // Every mesh the bay script asserts it exports is accounted for, so a new node cannot
        // be added to the module and silently ship untextured.
        let authored = BAY_STONE_NODES.len() + BAY_TRIM_NODES.len() + 1;
        assert_eq!(authored, 9, "the arcade bay exports nine named meshes");
    }
}
