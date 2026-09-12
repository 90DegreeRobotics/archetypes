//! Hanging real Chronos2 works, with provenance placards, in the walkable chambers.
//!
//! The works are staged into `assets/museum/` at author time by
//! `scripts/stage_museum_art.py`. **Nothing here reads `C:\chronos2`.** The launcher already
//! treats Chronos as optional and a buyer's machine has no such directory, so a museum that
//! read a sibling product's working folder would be twelve empty rooms on every machine but
//! the one it was built on.
//!
//! A placard states only what the bundle recorded: a title derived from the run's own prompt, a
//! date from its directory name, and its provenance hash. A bundle missing a field gets a
//! placard missing that line — never an invented artist, year, or wall text.

use bevy::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::castle::{all_hanging_positions, hanging_size, HANGINGS_PER_CHAMBER};
use super::world::InnerWorldElement;

/// One staged work, exactly as the manifest records it.
#[derive(Debug, Clone, Deserialize)]
pub struct Exhibit {
    pub run: String,
    pub title: String,
    pub created: String,
    /// Path relative to `assets/museum/`.
    pub image: String,
    pub placard: String,
    /// Staged pixel dimensions. The frame is sized from these, so it can be built before the
    /// texture has finished loading.
    #[serde(default)]
    pub width: u32,
    #[serde(default)]
    pub height: u32,
    #[serde(default)]
    pub prompt: String,
    /// Absent for bundles that never recorded one. The placard omits the line rather than
    /// filling it in.
    #[serde(default)]
    pub integrity_hash: Option<String>,
}

impl Exhibit {
    /// The work's own aspect ratio, as `gallery_exhibit.rs` computes it: `ar = aw/ah`, with the
    /// same 1.46 fallback when an image reports no height.
    pub fn aspect(&self) -> f32 {
        if self.height == 0 {
            1.46
        } else {
            self.width as f32 / self.height as f32
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MuseumManifest {
    #[serde(default)]
    pub schema: String,
    #[serde(default)]
    pub eligible_runs: usize,
    #[serde(default)]
    pub exhibits: Vec<Exhibit>,
}

impl MuseumManifest {
    pub fn empty() -> Self {
        Self {
            schema: String::new(),
            eligible_runs: 0,
            exhibits: Vec::new(),
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Museum {
    pub manifest: MuseumManifest,
}

/// Where the staged art lives on disk, checked beside the executable first so an installed
/// build reads its own copy rather than the repository's.
fn museum_root() -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets").join("museum"));
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("assets").join("museum"));
    }
    candidates.into_iter().find(|path| path.is_dir())
}

/// Reads the staged manifest. A missing or unreadable manifest yields an empty museum — the
/// chambers stand empty rather than the castle failing to load, and the count is reported so
/// "no art" is visible rather than silent.
pub fn load_manifest() -> MuseumManifest {
    let Some(root) = museum_root() else {
        warn!("museum: no staged art directory found; chambers will stand empty");
        return MuseumManifest::empty();
    };
    load_manifest_from(&root)
}

pub fn load_manifest_from(root: &Path) -> MuseumManifest {
    let path = root.join("manifest.json");
    let Ok(body) = std::fs::read_to_string(&path) else {
        warn!("museum: {} unreadable; chambers will stand empty", path.display());
        return MuseumManifest::empty();
    };
    match serde_json::from_str::<MuseumManifest>(&body) {
        Ok(manifest) => manifest,
        Err(error) => {
            error!("museum: {} is not a valid manifest: {error}", path.display());
            MuseumManifest::empty()
        }
    }
}

/// Marks a hung work, for finding and inspecting it later.
#[derive(Component)]
pub struct HungWork {
    pub exhibit: usize,
}

/// Moulding section, from `gallery_exhibit.rs`'s `fr.dimensions = (AW+0.06, 0.055, AH+0.06)`
/// scaled to a chamber-sized frame.
const MOULDING: f32 = 0.11;
const MOULDING_DEPTH: f32 = 0.14;
/// How far the picture plane stands off the wall.
const CANVAS_STANDOFF: f32 = 0.05;

/// Placard plate, proportioned to the 512x168 placard image.
const PLACARD_WIDTH: f32 = 1.10;
const PLACARD_HEIGHT: f32 = PLACARD_WIDTH * 168.0 / 512.0;
/// Below the frame's lower edge, to the placard's centre.
const PLACARD_DROP: f32 = 0.30;

pub struct MuseumPlugin;

impl Plugin for MuseumPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Museum {
            manifest: load_manifest(),
        });
    }
}

/// Hangs every staged work at its position in the circuit.
///
/// Exhibits map onto hanging positions in the manifest's own order, which is chronological by
/// run, so walking the circuit walks the library in the order it was made.
///
/// **Each frame is built here, per work, sized from that work's own aspect ratio.** That is what
/// `gallery_exhibit.rs::build_gallery_hall_script` does: it loads the image, takes `ar = aw/ah`,
/// fixes a height and derives the width, then builds the moulding around the result. A single
/// fixed-size frame asset cannot do that, and trying to make one fit is what forced every work
/// onto a 4:3 mat and made the paintings small.
pub fn spawn_hung_works(
    commands: &mut Commands,
    asset_server: &AssetServer,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    museum: &Museum,
) -> usize {
    let positions = all_hanging_positions();
    let mut hung = 0;

    // Ported from `gallery_exhibit.rs`: a near-black frame and a pale placard plate. The gilt
    // moulding in the first pass was invented.
    let moulding = materials.add(StandardMaterial {
        base_color: Color::srgb(0.02, 0.02, 0.022),
        perceptual_roughness: 0.45,
        metallic: 0.0,
        ..default()
    });
    let plate = materials.add(StandardMaterial {
        base_color: Color::srgb(0.90, 0.89, 0.86),
        perceptual_roughness: 0.75,
        ..default()
    });

    for (index, exhibit) in museum.manifest.exhibits.iter().enumerate() {
        let Some((position, yaw)) = positions.get(index).copied() else {
            // More staged works than walls to hang them on. Not an error: the staging script can
            // be run with a larger limit, and the extras wait for more chambers.
            break;
        };

        let size = hanging_size(exhibit.aspect());
        let half_w = size.x * 0.5;
        let half_h = size.y * 0.5;
        let rotation = Quat::from_rotation_y(yaw);

        // Every part below is a child of the frame root, so its transform is **local**: +X
        // across the wall, +Y up, +Z out into the room. Handing a child a world position and a
        // world rotation double-applies the parent's transform and flings the whole frame out
        // of the room, which is exactly what happened on the first attempt.
        let local = |dx: f32, dy: f32, dz: f32| Transform::from_xyz(dx, dy, dz);

        let root = commands
            .spawn((
                Transform::from_translation(position).with_rotation(rotation),
                Visibility::default(),
                HungWork { exhibit: index },
                InnerWorldElement,
                Name::new(format!("Exhibit_{index:02}_{}", exhibit.run)),
            ))
            .id();

        // Four moulding bars rather than one ring, so the mitres read and each run catches its
        // own highlight.
        for (name, offset_x, offset_y, dim_x, dim_y) in [
            ("Top", 0.0, half_h + MOULDING * 0.5, size.x + MOULDING * 2.0, MOULDING),
            ("Bottom", 0.0, -(half_h + MOULDING * 0.5), size.x + MOULDING * 2.0, MOULDING),
            ("Left", -(half_w + MOULDING * 0.5), 0.0, MOULDING, size.y),
            ("Right", half_w + MOULDING * 0.5, 0.0, MOULDING, size.y),
        ] {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(dim_x, dim_y, MOULDING_DEPTH))),
                MeshMaterial3d(moulding.clone()),
                local(offset_x, offset_y, MOULDING_DEPTH * 0.5 - 0.01),
                InnerWorldElement,
                ChildOf(root),
                Name::new(format!("Exhibit_{index:02}_Moulding{name}")),
            ));
        }

        // The picture. `gallery_exhibit.rs` shades its canvases with an emission shader at
        // strength 1.0 - a work in a dim hall has to carry some of its own light or it is a grey
        // rectangle. Its own spot and the chamber's warm source do the rest.
        let artwork: Handle<Image> = asset_server.load(format!("museum/{}", exhibit.image));
        let canvas = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(artwork.clone()),
            emissive_texture: Some(artwork),
            emissive: LinearRgba::rgb(0.85, 0.85, 0.85),
            perceptual_roughness: 0.68,
            metallic: 0.0,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(size.x, size.y))),
            MeshMaterial3d(canvas),
            local(0.0, 0.0, CANVAS_STANDOFF),
            InnerWorldElement,
            ChildOf(root),
            Name::new(format!("Exhibit_{index:02}_Canvas")),
        ));

        let placard_y = -(half_h + MOULDING + PLACARD_DROP);
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                PLACARD_WIDTH + 0.05,
                PLACARD_HEIGHT + 0.05,
                0.05,
            ))),
            MeshMaterial3d(plate.clone()),
            local(0.0, placard_y, 0.025),
            InnerWorldElement,
            ChildOf(root),
            Name::new(format!("Exhibit_{index:02}_PlacardPlate")),
        ));
        let placard_image: Handle<Image> =
            asset_server.load(format!("museum/{}", exhibit.placard));
        let placard = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(placard_image.clone()),
            emissive_texture: Some(placard_image),
            emissive: LinearRgba::rgb(0.55, 0.55, 0.55),
            perceptual_roughness: 0.58,
            ..default()
        });
        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(PLACARD_WIDTH, PLACARD_HEIGHT))),
            MeshMaterial3d(placard),
            local(0.0, placard_y, 0.055),
            InnerWorldElement,
            ChildOf(root),
            Name::new(format!("Exhibit_{index:02}_Placard")),
        ));

        // One warm spot per work, as the gallery hall gives each of its five.
        commands.spawn((
            SpotLight {
                intensity: 260_000.0,
                range: 14.0,
                color: Color::srgb(1.0, 0.90, 0.74),
                outer_angle: 0.62,
                inner_angle: 0.30,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, half_h + 1.5, 2.6)
                .looking_at(Vec3::new(0.0, 0.0, CANVAS_STANDOFF), Vec3::Y),
            InnerWorldElement,
            ChildOf(root),
            Name::new(format!("Exhibit_{index:02}_Spot")),
        ));

        hung += 1;
    }

    info!(
        "museum: hung {hung} works across {} chambers ({} positions, {} eligible runs staged)",
        positions.len() / HANGINGS_PER_CHAMBER,
        positions.len(),
        museum.manifest.eligible_runs
    );
    hung
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staged_root() -> PathBuf {
        PathBuf::from("../../assets/museum")
    }

    #[test]
    fn the_staged_manifest_loads_and_carries_works() {
        let manifest = load_manifest_from(&staged_root());
        assert_eq!(manifest.schema, "archetypes.museum.v1");
        assert!(
            !manifest.exhibits.is_empty(),
            "no works staged; run `python scripts/stage_museum_art.py`"
        );
    }

    /// Every exhibit must point at files that are actually in the repo. The whole reason for
    /// staging is that the game cannot reach the Chronos2 library at runtime.
    #[test]
    fn every_exhibit_points_at_a_file_that_shipped() {
        let root = staged_root();
        let manifest = load_manifest_from(&root);
        for exhibit in &manifest.exhibits {
            for relative in [&exhibit.image, &exhibit.placard] {
                let path = root.join(relative);
                assert!(
                    path.is_file(),
                    "{} is listed in the manifest but not staged",
                    path.display()
                );
            }
        }
    }

    /// A placard may omit a line, but it may never carry an invented one.
    #[test]
    fn every_exhibit_carries_only_what_its_bundle_recorded() {
        let manifest = load_manifest_from(&staged_root());
        for exhibit in &manifest.exhibits {
            assert!(!exhibit.title.is_empty(), "{} has no title", exhibit.run);
            assert_eq!(
                exhibit.created.len(),
                10,
                "{} has a malformed date {:?}",
                exhibit.run,
                exhibit.created
            );
            // The date must be derived from the run directory, not from anywhere else.
            let expected = format!(
                "{}-{}-{}",
                &exhibit.run[0..4],
                &exhibit.run[4..6],
                &exhibit.run[6..8]
            );
            assert_eq!(exhibit.created, expected, "{} date is not its own", exhibit.run);
            if let Some(hash) = &exhibit.integrity_hash {
                assert!(
                    hash.len() >= 12 && hash.chars().all(|c| c.is_ascii_hexdigit()),
                    "{} carries a hash that is not a hash: {hash:?}",
                    exhibit.run
                );
            }
        }
    }

    /// There must be somewhere to hang everything staged, or works are silently dropped.
    #[test]
    fn the_staged_library_fits_the_walls_that_exist() {
        let manifest = load_manifest_from(&staged_root());
        let positions = all_hanging_positions();
        assert!(
            manifest.exhibits.len() <= positions.len(),
            "{} works staged against {} hanging positions; the extras would never be seen",
            manifest.exhibits.len(),
            positions.len()
        );
        assert_eq!(positions.len() % HANGINGS_PER_CHAMBER, 0);
    }

    /// A missing library must leave empty chambers, not a failed boot. Chronos is optional.
    #[test]
    fn a_missing_library_yields_an_empty_museum_rather_than_a_crash() {
        let manifest = load_manifest_from(Path::new("../../assets/does-not-exist"));
        assert!(manifest.exhibits.is_empty());
        assert_eq!(manifest.eligible_runs, 0);
    }

    /// The staging script's limit and the wall count are two numbers that have to agree.
    #[test]
    fn the_staging_script_targets_the_number_of_walls_that_exist() {
        let source = std::fs::read_to_string("../../scripts/stage_museum_art.py")
            .expect("staging script readable");
        let expected = format!("DEFAULT_LIMIT = {}", all_hanging_positions().len());
        assert!(
            source.contains(&expected),
            "stage_museum_art.py no longer stages {} works, which is how many walls exist",
            all_hanging_positions().len()
        );
    }
}
