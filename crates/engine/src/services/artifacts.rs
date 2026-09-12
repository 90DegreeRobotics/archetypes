//! The player's library of manifested objects, and where they have put them.
//!
//! **Why this exists.** Manifestation used to write one fixed path,
//! `assets/scenes/manifested_artifact.glb`, and overwrite it every run. Bevy caches by asset
//! path, so every entity spawned from that path is the *same asset*: place two creations in the
//! hall and they are not two objects, they are two views of whatever was made last, and the
//! third one silently changes the first two. Nothing the player makes can be carried, placed or
//! duplicated until each one is its own file.
//!
//! So each manifestation now writes `assets/manifested/<id>.glb` and appends a row here. The id
//! is the run's own identifier, so the file, the library row and the placement all name the same
//! thing.
//!
//! Placements are an **append-only ledger**, the third instance of the pattern
//! `services/build_intent.rs` and `services/encounter_memory.rs` already use: folded from disk
//! on read, never rewritten in place, and a removal recorded as a withdrawal rather than a
//! deletion. What the player built stays auditable, including what they took back down.

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use super::paths::app_data_root;

/// What a creation is beyond its file: what governed it, and the digests binding it to the
/// Chronos2 run it came from.
///
/// Recorded at manifestation time because it cannot be recovered later -- the bundle lives in a
/// temp directory that will not survive, and once it is gone a GLB on disk is just a GLB. The
/// row is the only durable place these numbers exist.
///
/// `glb_sha256` is Archetypes' own measurement. Chronos2 cannot supply it: the GLB is produced
/// afterwards by this repository's Blender import, so no hash of it can exist in a Chronos2
/// bundle. See `services/chronos_receipt.rs`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    #[serde(default)]
    pub sentinel_verdict: String,
    #[serde(default)]
    pub mesh_sha256: String,
    #[serde(default)]
    pub source_image_sha256: String,
    #[serde(default)]
    pub glb_sha256: String,
    #[serde(default)]
    pub subject_coverage: f64,
}

/// One object the player has made. `asset` is relative to the assets root, so it is what the
/// asset server is handed directly.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub id: String,
    pub asset: String,
    #[serde(default)]
    pub prompt: String,
    #[serde(default)]
    pub created: String,
    /// Absent on rows written before provenance was recorded. Those creations are still the
    /// player's and still load; they simply cannot show where they came from.
    #[serde(default)]
    pub provenance: Provenance,
}

/// One row of the placement ledger. Append-only: a `Placed` and a later `Withdrawn` for the
/// same `placement` are both kept, and the fold below resolves them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PlacementEvent {
    Placed {
        placement: String,
        artifact: String,
        asset: String,
        position: [f32; 3],
        yaw: f32,
        scale: f32,
        at: String,
    },
    Withdrawn { placement: String, at: String },
}

/// A placement as it currently stands, after folding the ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct Placement {
    pub placement: String,
    pub artifact: String,
    pub asset: String,
    pub position: [f32; 3],
    pub yaw: f32,
    pub scale: f32,
}

pub fn library_root() -> PathBuf {
    app_data_root().join("artifacts")
}

pub fn library_path() -> PathBuf {
    library_root().join("library.jsonl")
}

pub fn placements_path() -> PathBuf {
    library_root().join("placements.jsonl")
}

/// Where a manifested GLB is written so the asset server can find it.
///
/// Beside the executable, because an installed build must read its own copy rather than the
/// repository's — the same rule the museum's staged art follows.
pub fn manifested_assets_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("assets").join("manifested")))
}

/// The asset-server path for an artifact id. Relative, forward slashes: that is what the asset
/// server expects, and a backslash here silently fails to load on a path it would otherwise
/// find.
pub fn asset_path_for(id: &str) -> String {
    format!("manifested/{id}.glb")
}

fn now_stamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{seconds}")
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| error.to_string())
}

/// Records a newly manifested object in the library.
pub fn record_artifact(
    id: &str,
    prompt: &str,
    provenance: Provenance,
) -> Result<ArtifactRecord, String> {
    let record = ArtifactRecord {
        id: id.to_string(),
        asset: asset_path_for(id),
        prompt: prompt.to_string(),
        created: now_stamp(),
        provenance,
    };
    let line = serde_json::to_string(&record).map_err(|error| error.to_string())?;
    append_line(&library_path(), &line)?;
    Ok(record)
}

/// The library row for one artifact id, if the player still has it.
///
/// Picking an object up off the floor knows only what the placement recorded -- an id, an asset
/// path and a scale. Everything that makes it *the player's creation* rather than a mesh (the
/// prompt it came from, what governed it, the digests binding it to its run) lives on the
/// library row, so it is fetched from there rather than synthesised as blanks.
pub fn find_artifact(id: &str) -> Option<ArtifactRecord> {
    load_library().into_iter().rev().find(|record| record.id == id)
}

pub fn load_library() -> Vec<ArtifactRecord> {
    load_library_from(&library_path())
}

pub fn load_library_from(path: &Path) -> Vec<ArtifactRecord> {
    let Ok(body) = fs::read_to_string(path) else {
        return Vec::new();
    };
    body.lines()
        .filter(|line| !line.trim().is_empty())
        // A corrupt row is skipped, not fatal. One bad line must never cost the player their
        // whole library.
        .filter_map(|line| serde_json::from_str::<ArtifactRecord>(line).ok())
        .collect()
}

pub fn record_placement(
    placement: &str,
    artifact: &str,
    asset: &str,
    position: [f32; 3],
    yaw: f32,
    scale: f32,
) -> Result<(), String> {
    let event = PlacementEvent::Placed {
        placement: placement.to_string(),
        artifact: artifact.to_string(),
        asset: asset.to_string(),
        position,
        yaw,
        scale,
        at: now_stamp(),
    };
    let line = serde_json::to_string(&event).map_err(|error| error.to_string())?;
    append_line(&placements_path(), &line)
}

/// Records that a placement was taken back down.
///
/// Written as a withdrawal rather than by removing the original row: the ledger is the record of
/// what the player did, and "they put this here and later took it away" is a different fact from
/// "this never happened". `encounter_memory`'s Forget works the same way.
pub fn withdraw_placement(placement: &str) -> Result<(), String> {
    let event = PlacementEvent::Withdrawn {
        placement: placement.to_string(),
        at: now_stamp(),
    };
    let line = serde_json::to_string(&event).map_err(|error| error.to_string())?;
    append_line(&placements_path(), &line)
}

pub fn load_placements() -> Vec<Placement> {
    load_placements_from(&placements_path())
}

/// Folds the ledger into what currently stands, in placement order.
pub fn load_placements_from(path: &Path) -> Vec<Placement> {
    let Ok(body) = fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut standing: Vec<Placement> = Vec::new();
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(event) = serde_json::from_str::<PlacementEvent>(line) else {
            continue;
        };
        match event {
            PlacementEvent::Placed {
                placement,
                artifact,
                asset,
                position,
                yaw,
                scale,
                ..
            } => {
                // A repeated id replaces rather than duplicates, so a re-placed object does not
                // stack copies of itself.
                standing.retain(|existing| existing.placement != placement);
                standing.push(Placement {
                    placement,
                    artifact,
                    asset,
                    position,
                    yaw,
                    scale,
                });
            }
            PlacementEvent::Withdrawn { placement, .. } => {
                standing.retain(|existing| existing.placement != placement);
            }
        }
    }
    standing
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "archetypes-artifacts-{name}-{}.jsonl",
            std::process::id()
        ))
    }

    /// Pins the *whole* path, not just the product name.
    ///
    /// `app_data_root()` ends in `data`, with `config` as its sibling. A tool that mirrors only
    /// `NeuroCognica/Archetypes` writes a ledger the game never reads — which looks exactly like
    /// a placement system that does not work, and cost a capture run to find.
    #[test]
    fn the_library_lives_under_the_product_data_root() {
        for path in [library_path(), placements_path()] {
            let rendered = path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
            assert!(
                rendered.contains("NeuroCognica/Archetypes/data/artifacts/"),
                "{rendered} is not under the data root the engine reads"
            );
        }
        assert!(library_path().ends_with("library.jsonl"));
        assert!(placements_path().ends_with("placements.jsonl"));
    }

    /// The seeding tool has to mirror that path exactly, or it writes somewhere harmless and
    /// silent.
    #[test]
    fn the_seeding_tool_mirrors_the_same_data_root() {
        let source = std::fs::read_to_string("../../scripts/seed_object_demo.py")
            .expect("seed script readable");
        assert!(
            source.contains(r#""NeuroCognica" / "Archetypes" / "data""#),
            "seed_object_demo.py no longer mirrors the engine's data root"
        );
    }

    /// The whole point of the library: two artifacts are two files. One shared path was the
    /// reason nothing could be carried, placed or duplicated.
    #[test]
    fn every_artifact_gets_its_own_asset_path() {
        let a = asset_path_for("20260912_1");
        let b = asset_path_for("20260912_2");
        assert_ne!(a, b);
        assert!(a.starts_with("manifested/") && a.ends_with(".glb"));
        // Forward slashes: a backslash here fails to load on a path the asset server would
        // otherwise find.
        assert!(!a.contains('\\'));
    }

    /// Rows written before provenance existed must still load. The player's earlier creations
    /// are theirs; a new field is not a reason to lose them.
    #[test]
    fn creations_recorded_before_provenance_existed_still_load() {
        let path = scratch("legacy-rows");
        fs::write(
            &path,
            "{\"id\":\"old-1\",\"asset\":\"manifested/old-1.glb\",\"prompt\":\"a lantern\",\"created\":\"1\"}
",
        )
        .unwrap();
        let rows = load_library_from(&path);
        assert_eq!(rows.len(), 1, "a pre-provenance row was dropped");
        assert_eq!(rows[0].prompt, "a lantern");
        assert_eq!(rows[0].provenance, Provenance::default());
        let _ = fs::remove_file(&path);
    }

    /// Provenance survives the round trip, so what the library shows is what was measured at
    /// manifestation rather than a default that merely looks plausible.
    #[test]
    fn provenance_survives_the_round_trip() {
        let path = scratch("provenance-round-trip");
        let record = ArtifactRecord {
            id: "a".into(),
            asset: asset_path_for("a"),
            prompt: "a brass astrolabe".into(),
            created: "1".into(),
            provenance: Provenance {
                sentinel_verdict: "allowed".into(),
                mesh_sha256: "f17709c7".into(),
                source_image_sha256: "62b2e3b7".into(),
                glb_sha256: "aabbccdd".into(),
                subject_coverage: 0.4456,
            },
        };
        fs::write(&path, serde_json::to_string(&record).unwrap() + "
").unwrap();
        assert_eq!(load_library_from(&path), vec![record]);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn a_missing_ledger_is_an_empty_world_not_a_crash() {
        assert!(load_library_from(Path::new("does-not-exist.jsonl")).is_empty());
        assert!(load_placements_from(Path::new("does-not-exist.jsonl")).is_empty());
    }

    #[test]
    fn a_corrupt_row_costs_one_row_and_no_more() {
        let path = scratch("corrupt");
        let good = serde_json::to_string(&ArtifactRecord {
            id: "a".into(),
            asset: asset_path_for("a"),
            prompt: "lantern".into(),
            created: "1".into(),
            provenance: Provenance::default(),
        })
        .unwrap();
        fs::write(&path, format!("{good}\nnot json at all\n{good}\n")).unwrap();
        assert_eq!(load_library_from(&path).len(), 2);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn the_placement_ledger_folds_placements_and_withdrawals() {
        let path = scratch("placements");
        let rows = [
            PlacementEvent::Placed {
                placement: "p1".into(),
                artifact: "a".into(),
                asset: asset_path_for("a"),
                position: [1.0, 2.0, 3.0],
                yaw: 0.5,
                scale: 1.0,
                at: "1".into(),
            },
            PlacementEvent::Placed {
                placement: "p2".into(),
                artifact: "a".into(),
                asset: asset_path_for("a"),
                position: [4.0, 5.0, 6.0],
                yaw: 1.5,
                scale: 2.0,
                at: "2".into(),
            },
            PlacementEvent::Withdrawn {
                placement: "p1".into(),
                at: "3".into(),
            },
        ];
        let body: String = rows
            .iter()
            .map(|row| serde_json::to_string(row).unwrap() + "\n")
            .collect();
        fs::write(&path, body).unwrap();

        let standing = load_placements_from(&path);
        assert_eq!(standing.len(), 1, "the withdrawn placement still stands");
        assert_eq!(standing[0].placement, "p2");
        assert_eq!(standing[0].position, [4.0, 5.0, 6.0]);
        assert_eq!(standing[0].scale, 2.0);
        let _ = fs::remove_file(&path);
    }

    /// A withdrawal must not erase the history. The ledger records what the player did,
    /// including taking something back down.
    #[test]
    fn a_withdrawal_is_recorded_rather_than_deleting_the_original_row() {
        let path = scratch("withdraw-keeps-history");
        let placed = serde_json::to_string(&PlacementEvent::Placed {
            placement: "p1".into(),
            artifact: "a".into(),
            asset: asset_path_for("a"),
            position: [0.0; 3],
            yaw: 0.0,
            scale: 1.0,
            at: "1".into(),
        })
        .unwrap();
        let withdrawn = serde_json::to_string(&PlacementEvent::Withdrawn {
            placement: "p1".into(),
            at: "2".into(),
        })
        .unwrap();
        fs::write(&path, format!("{placed}\n{withdrawn}\n")).unwrap();

        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("\"kind\":\"Placed\""), "the original row was removed");
        assert!(body.contains("\"kind\":\"Withdrawn\""));
        assert!(load_placements_from(&path).is_empty());
        let _ = fs::remove_file(&path);
    }

    /// Re-placing the same object moves it rather than leaving a copy behind.
    #[test]
    fn re_placing_an_object_moves_it_instead_of_stacking_copies() {
        let path = scratch("replace");
        let first = serde_json::to_string(&PlacementEvent::Placed {
            placement: "p1".into(),
            artifact: "a".into(),
            asset: asset_path_for("a"),
            position: [1.0, 0.0, 0.0],
            yaw: 0.0,
            scale: 1.0,
            at: "1".into(),
        })
        .unwrap();
        let again = serde_json::to_string(&PlacementEvent::Placed {
            placement: "p1".into(),
            artifact: "a".into(),
            asset: asset_path_for("a"),
            position: [9.0, 0.0, 0.0],
            yaw: 0.0,
            scale: 1.0,
            at: "2".into(),
        })
        .unwrap();
        fs::write(&path, format!("{first}\n{again}\n")).unwrap();

        let standing = load_placements_from(&path);
        assert_eq!(standing.len(), 1);
        assert_eq!(standing[0].position, [9.0, 0.0, 0.0]);
        let _ = fs::remove_file(&path);
    }

    /// Duplicates are separate placements of the same artifact. The operator asked for filling
    /// a room with copies, so the ledger has to allow many placements naming one asset.
    #[test]
    fn many_placements_may_share_one_artifact() {
        let path = scratch("duplicates");
        let body: String = (0..8)
            .map(|index| {
                serde_json::to_string(&PlacementEvent::Placed {
                    placement: format!("p{index}"),
                    artifact: "a".into(),
                    asset: asset_path_for("a"),
                    position: [index as f32, 0.0, 0.0],
                    yaw: 0.0,
                    scale: 1.0,
                    at: "1".into(),
                })
                .unwrap()
                    + "\n"
            })
            .collect();
        fs::write(&path, body).unwrap();

        let standing = load_placements_from(&path);
        assert_eq!(standing.len(), 8);
        assert!(standing.iter().all(|p| p.asset == asset_path_for("a")));
        let _ = fs::remove_file(&path);
    }
}
