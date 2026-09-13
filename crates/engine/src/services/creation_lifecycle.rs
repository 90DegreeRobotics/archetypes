//! The loop a player actually walks, proven end to end.
//!
//! Step 3 of `docs/ledger/2026/09/plan_2026-09-12_1830_persistent_manifester_truth.md`:
//! *"Exercise manifest-to-library, summon, place, duplicate, withdraw, reload, and re-summon
//! against a real artifact fixture."*
//!
//! **Why this exists when the unit tests already pass.** Every stage of this lifecycle is unit
//! tested in its own module, and all of those passed while a debug build wrote manifested objects
//! to a directory its own asset server never read -- because no test ever asked the question that
//! spans two modules: *can what was recorded actually be found again afterwards?* A collection of
//! green unit tests is not a proven loop, and the gap between them is precisely where the player
//! lives.
//!
//! The ledgers here are scratch files. The path-parameterised writers exist so this test cannot
//! touch the operator's own library -- a proof that damages the thing it is proving is not one.

//! Lives in-crate rather than under `tests/` because `engine` is a binary crate with no library
//! target, so an integration test cannot import it. What matters about this file is that its
//! assertions span modules -- the bridge, the ledger and the asset resolver together -- not which
//! directory it sits in.

#![cfg(test)]

use super::artifacts::{self, ArtifactRecord, Provenance};
use super::chronos_receipt;
use std::path::{Path, PathBuf};

struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir()
            .join(format!("archetypes-lifecycle-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("scratch root");
        Scratch { root }
    }

    fn library(&self) -> PathBuf {
        self.root.join("library.jsonl")
    }

    fn placements(&self) -> PathBuf {
        self.root.join("placements.jsonl")
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// A Chronos2 bundle this machine really produced, if one is present.
///
/// Machine-local evidence, not a repository fixture, so its absence skips rather than fails.
fn a_real_bundle() -> Option<PathBuf> {
    std::fs::read_dir("../../artifacts/quality-lab/bundles")
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.join("engine_mesh").join("triposr_artifact.json").is_file())
}

/// Manifest, place, duplicate, withdraw, restart, and summon again.
///
/// Each assertion is the question the player would ask by doing the thing, in the order they
/// would do it. "Reload" is modelled the way a restart actually behaves: nothing is carried over
/// in memory, the ledgers are re-read from disk, and what stands is whatever the fold says.
#[test]
fn a_creation_survives_placement_duplication_withdrawal_and_a_restart() {
    let scratch = Scratch::new("full-loop");

    // 1. Manifest. The row is what makes a GLB on disk *the player's creation* rather than a
    //    file: it carries the request it answered and the provenance of the run that built it.
    let provenance = Provenance {
        sentinel_verdict: "allowed".to_owned(),
        mesh_sha256: "f17709c749b14c93".to_owned(),
        source_image_sha256: "62b2e3b780e1e37e".to_owned(),
        glb_sha256: "aabbccddeeff0011".to_owned(),
        subject_coverage: 0.4456,
    };
    let record = artifacts::record_artifact_in(
        &scratch.library(),
        "20260912_astrolabe",
        "a brass astrolabe",
        provenance.clone(),
    )
    .expect("the creation is recorded");
    assert_eq!(record.asset, "manifested/20260912_astrolabe.glb");

    // 2. The library is reachable, and it is the same creation that comes back.
    let library = artifacts::load_library_from(&scratch.library());
    assert_eq!(library.len(), 1);
    assert_eq!(library[0].prompt, "a brass astrolabe");
    assert_eq!(library[0].provenance, provenance, "provenance did not survive the ledger");

    // 3. Place it, then duplicate it twice. Duplicates are separate placements of ONE artifact:
    //    the operator asked to be able to fill a room with copies, and that must not mean three
    //    copies of the file.
    for (index, x) in [0.0_f32, 0.9, -0.9].iter().enumerate() {
        artifacts::record_placement_in(
            &scratch.placements(),
            &format!("p{index}"),
            &record.id,
            &record.asset,
            [*x, 0.0, -2.2],
            0.0,
            1.0,
        )
        .expect("placement recorded");
    }
    let standing = artifacts::load_placements_from(&scratch.placements());
    assert_eq!(standing.len(), 3, "three placements should stand");
    assert!(
        standing.iter().all(|placement| placement.asset == record.asset),
        "duplicates must share one artifact file, not copy it"
    );

    // 4. Take one back up. The ledger records a withdrawal rather than forgetting it happened.
    artifacts::withdraw_placement_in(&scratch.placements(), "p1").expect("withdrawal recorded");
    let standing = artifacts::load_placements_from(&scratch.placements());
    assert_eq!(standing.len(), 2);
    assert!(!standing.iter().any(|placement| placement.placement == "p1"));

    let ledger = std::fs::read_to_string(scratch.placements()).unwrap();
    assert!(
        ledger.contains("\"kind\":\"Placed\"") && ledger.contains("\"kind\":\"Withdrawn\""),
        "history was rewritten instead of appended to"
    );

    // 5. Restart. Nothing is carried over; both ledgers are re-read from disk.
    let library = artifacts::load_library_from(&scratch.library());
    let standing = artifacts::load_placements_from(&scratch.placements());
    assert_eq!(library.len(), 1, "the creation did not survive the restart");
    assert_eq!(standing.len(), 2, "the decorated room did not survive the restart");

    // 6. Summon it again from the library. The re-summoned record must be the same creation --
    //    same id, same asset, same provenance -- or the library is showing one thing and handing
    //    over another.
    let summoned = library
        .iter()
        .rev()
        .find(|candidate| candidate.id == record.id)
        .expect("the creation is still summonable after a restart");
    assert_eq!(*summoned, record);

    // 7. What still stands names an artifact the library still knows. A placement pointing at a
    //    creation that no longer exists would spawn nothing and look like a broken world.
    for placement in &standing {
        assert!(
            library.iter().any(|candidate| candidate.id == placement.artifact),
            "placement {} names an artifact no longer in the library",
            placement.placement
        );
    }
}

/// The library is the player's, so one damaged row costs one row.
///
/// Appended to by a live game and read at startup, a JSONL file will eventually be caught
/// mid-write by a crash or a power cut. Losing the whole collection to that would be losing
/// everything the player has made.
#[test]
fn a_torn_write_costs_one_creation_and_not_the_collection() {
    let scratch = Scratch::new("torn-write");
    for index in 0..3 {
        artifacts::record_artifact_in(
            &scratch.library(),
            &format!("id-{index}"),
            "a lantern",
            Provenance::default(),
        )
        .unwrap();
    }
    // A row cut off mid-write, exactly as an interrupted append leaves it.
    let mut body = std::fs::read_to_string(scratch.library()).unwrap();
    body.push_str("{\"id\":\"id-3\",\"asset\":\"manif");
    std::fs::write(scratch.library(), body).unwrap();

    let library = artifacts::load_library_from(&scratch.library());
    assert_eq!(library.len(), 3, "a torn final row cost more than itself");
    assert_eq!(library[0].id, "id-0");
}

/// A creation whose file is gone must be visibly unavailable, never silently summoned.
///
/// Every row here resolves to nothing, because nothing was written to an assets root. Summoning
/// one would put an invisible object in the player's hand, which reads as the game being broken
/// rather than as a file being missing.
#[test]
fn a_creation_whose_file_is_gone_does_not_resolve() {
    let scratch = Scratch::new("missing-asset");
    let record = artifacts::record_artifact_in(
        &scratch.library(),
        "never-staged",
        "a lantern",
        Provenance::default(),
    )
    .unwrap();
    assert!(
        artifacts::resolve_asset(&record.asset).is_none(),
        "an artifact with no file on disk must not resolve"
    );
}

/// The bridge end of the loop, against a bundle Chronos2 really wrote.
///
/// The lifecycle above starts from a record; this starts from the thing that produces one, so
/// the two halves meet. Skips where no local bundle exists.
#[test]
fn a_real_bundle_verifies_and_yields_a_recordable_creation() {
    let Some(bundle) = a_real_bundle() else {
        eprintln!("no local Chronos2 bundle; bridge half of the lifecycle unproven here");
        return;
    };
    let prompt = std::fs::read_to_string(bundle.join("human_prompt.txt")).unwrap_or_default();
    let prompt = prompt.trim();

    match chronos_receipt::verify(&bundle, prompt) {
        Ok(artifact) => {
            let scratch = Scratch::new("bridge");
            let record = artifacts::record_artifact_in(
                &scratch.library(),
                "from-real-bundle",
                prompt,
                Provenance {
                    sentinel_verdict: artifact.sentinel_verdict.clone(),
                    mesh_sha256: artifact.mesh_sha256.clone(),
                    source_image_sha256: artifact.source_image_sha256.clone(),
                    // Measured from the verified mesh: this test does not run Blender, so the
                    // mesh stands in for the GLB the import would produce. What is being proven
                    // is that a real measurement reaches the row, not which file it is of.
                    glb_sha256: chronos_receipt::sha256_file(&artifact.mesh).unwrap(),
                    subject_coverage: artifact.subject_coverage,
                },
            )
            .unwrap();

            let reloaded = artifacts::load_library_from(&scratch.library());
            assert_eq!(reloaded.len(), 1);
            assert_eq!(reloaded[0].provenance.sentinel_verdict, "allowed");
            assert_eq!(reloaded[0].provenance.mesh_sha256.len(), 64);
            assert_eq!(reloaded[0].prompt, prompt);
            assert_eq!(reloaded[0], record);
        }
        // The lab spread deliberately includes subjects that must be refused. A refusal here is
        // the contract working; any other fault means the contract is wrong.
        Err(chronos_receipt::ReceiptFault::SubjectLost { coverage }) => {
            assert!(coverage < chronos_receipt::MIN_SUBJECT_COVERAGE);
            eprintln!("bundle correctly refused at {coverage:.4} coverage");
        }
        Err(fault) => panic!("{}: {fault}", bundle.display()),
    }
}

/// Ledger paths stay under the product's data root.
///
/// Pinned from outside the crate as well as inside it: a tool that mirrors only
/// `NeuroCognica/Archetypes` writes a ledger the game never reads, which looks exactly like a
/// placement system that does not work and has already cost one capture run to find.
#[test]
fn the_players_collection_lives_under_the_product_data_root() {
    for path in [artifacts::library_path(), artifacts::placements_path()] {
        let rendered = path.to_string_lossy().replace(std::path::MAIN_SEPARATOR, "/");
        assert!(
            rendered.contains("NeuroCognica/Archetypes/data/artifacts/"),
            "{rendered} is not under the data root the engine reads"
        );
    }
    assert!(Path::new(&artifacts::library_path()).ends_with("library.jsonl"));
}

/// A summoned creation is the *same file*, never a copy.
///
/// The operator asked to be able to fill a room with duplicates. That must mean many placements
/// naming one asset -- if summoning copied the file, a room of twenty duplicates would be twenty
/// meshes in memory and twenty files on disk for one creation.
#[test]
fn summoning_and_duplicating_never_copy_the_artifact_file() {
    let scratch = Scratch::new("no-copies");
    let record = artifacts::record_artifact_in(
        &scratch.library(),
        "one-file",
        "a lantern",
        Provenance::default(),
    )
    .unwrap();

    for index in 0..20 {
        artifacts::record_placement_in(
            &scratch.placements(),
            &format!("p{index}"),
            &record.id,
            &record.asset,
            [index as f32, 0.0, 0.0],
            0.0,
            1.0,
        )
        .unwrap();
    }

    let standing = artifacts::load_placements_from(&scratch.placements());
    assert_eq!(standing.len(), 20);
    let distinct: std::collections::BTreeSet<&str> =
        standing.iter().map(|placement| placement.asset.as_str()).collect();
    assert_eq!(distinct.len(), 1, "duplicates multiplied the file instead of the placement");

    // And one creation, however many times it stands.
    assert_eq!(artifacts::load_library_from(&scratch.library()).len(), 1);
}

/// Guards the record type against a field being added without a default.
///
/// `ArtifactRecord` is read back from a file older builds wrote. A new required field silently
/// drops every existing row on deserialise -- the player's whole collection -- and the symptom is
/// an empty library, not an error.
#[test]
fn a_row_from_an_older_build_still_loads() {
    let scratch = Scratch::new("older-build");
    std::fs::write(
        scratch.library(),
        "{\"id\":\"old\",\"asset\":\"manifested/old.glb\"}\n",
    )
    .unwrap();
    let library = artifacts::load_library_from(&scratch.library());
    assert_eq!(library.len(), 1, "a row from an older build was dropped");
    assert_eq!(library[0], ArtifactRecord {
        id: "old".to_owned(),
        asset: "manifested/old.glb".to_owned(),
        ..Default::default()
    });
}
