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

use super::castle::{all_hanging_positions, HANGINGS_PER_CHAMBER};
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
    #[serde(default)]
    pub prompt: String,
    /// Absent for bundles that never recorded one. The placard omits the line rather than
    /// filling it in.
    #[serde(default)]
    pub integrity_hash: Option<String>,
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

/// Marks a hung frame so its canvas and placard can be dressed once its scene has loaded.
#[derive(Component)]
pub struct HungWork {
    pub exhibit: usize,
    pub image: Handle<Image>,
    pub placard: Handle<Image>,
    pub bound: bool,
}

const CANVAS_NODE: &str = "Frame_Canvas";
const PLACARD_NODE: &str = "Frame_Placard";

pub struct MuseumPlugin;

impl Plugin for MuseumPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Museum {
            manifest: load_manifest(),
        })
        .add_systems(
            Update,
            bind_hung_works.run_if(
                in_state(super::InnerChambersState::Loading)
                    .or(in_state(super::InnerChambersState::Navigating)),
            ),
        );
    }
}

/// Hangs every staged work at its position in the circuit.
///
/// Exhibits map onto hanging positions in the manifest's own order, which is chronological by
/// run. Chamber 0 holds the five oldest works, chamber 1 the next five, and so on — so walking
/// the circuit walks the library in the order it was made.
pub fn spawn_hung_works(
    commands: &mut Commands,
    asset_server: &AssetServer,
    museum: &Museum,
) -> usize {
    let positions = all_hanging_positions();
    let mut hung = 0;

    for (index, exhibit) in museum.manifest.exhibits.iter().enumerate() {
        let Some((position, yaw)) = positions.get(index).copied() else {
            // More staged works than walls to hang them on. Not an error: the staging script
            // can be run with a larger limit, and the extras simply wait for more chambers.
            break;
        };

        let image = asset_server.load(format!("museum/{}", exhibit.image));
        let placard = asset_server.load(format!("museum/{}", exhibit.placard));

        commands.spawn((
            SceneRoot(asset_server.load("scenes/art_frame.glb#Scene0")),
            Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw)),
            HungWork {
                exhibit: index,
                image,
                placard,
                bound: false,
            },
            InnerWorldElement,
            Name::new(format!(
                "Exhibit_{index:02}_{}",
                exhibit.run
            )),
        ));
        hung += 1;
    }

    info!(
        "museum: hung {hung} works across {} chambers ({} positions available, {} eligible runs \
         in the staged library)",
        positions.len() / HANGINGS_PER_CHAMBER,
        positions.len(),
        museum.manifest.eligible_runs
    );
    hung
}

/// Applies each work's own image and placard once the frame scene's children exist.
fn bind_hung_works(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut frames: Query<(Entity, &mut HungWork)>,
    children: Query<&Children>,
    names: Query<&Name>,
    meshes: Query<(), With<Mesh3d>>,
) {
    for (root, mut frame) in &mut frames {
        if frame.bound {
            continue;
        }
        let mut bound_any = false;

        for descendant in children.iter_descendants(root) {
            let Ok(name) = names.get(descendant) else {
                continue;
            };
            let texture = match name.as_str() {
                CANVAS_NODE => frame.image.clone(),
                PLACARD_NODE => frame.placard.clone(),
                _ => continue,
            };

            // Unlit would be simpler and would be wrong: an artwork that ignores the room's
            // light is a lightbox on a wall, not a picture in a gallery. The chamber's own warm
            // source is what reveals it.
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                base_color_texture: Some(texture),
                perceptual_roughness: 0.68,
                metallic: 0.0,
                ..default()
            });

            if meshes.get(descendant).is_ok() {
                commands
                    .entity(descendant)
                    .insert(MeshMaterial3d(material.clone()));
                bound_any = true;
            }
            for inner in children.iter_descendants(descendant) {
                if meshes.get(inner).is_ok() {
                    commands
                        .entity(inner)
                        .insert(MeshMaterial3d(material.clone()));
                    bound_any = true;
                }
            }
        }

        if bound_any {
            frame.bound = true;
        }
    }
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
