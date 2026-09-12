//! The contract Archetypes requires of a Chronos2 bundle before anything reaches the altar.
//!
//! **Why this exists.** The bridge used to conclude "it worked" from two facts: the process
//! exited zero, and two files happened to be present on disk. Neither is evidence. A bundle left
//! over from an earlier prompt satisfies both. So does a run the Sentinel refused, because a
//! refusal is a *successful* run of the governance path -- it exits zero and writes its bundle.
//! So does a mesh truncated by a crash mid-write, because a partial `.obj` is still a file.
//!
//! What makes those distinguishable is that Chronos2 already measures and records all of it.
//! Every field checked below is one Chronos2 emits today, verified against a real bundle from
//! `first-light --geometry-forge --void` -- the same command `manifestation.rs` runs. Nothing
//! here is a field this module wishes existed.
//!
//! ## What is deliberately NOT checked
//!
//! **A game-GLB digest.** Chronos2 cannot supply one: the GLB is produced afterwards by
//! *Archetypes'* own Blender import, so no hash of it can exist in a Chronos2 bundle. Archetypes
//! measures it itself once the import returns, and records it on the library row. Asking
//! Chronos2 for it would be asking for a field that can only ever be absent, and a check that
//! can only ever fail is a check nobody keeps.
//!
//! **A "no-recipe" declaration.** There is no such field, and inventing one would be theatre: a
//! bundle attesting to its own innocence proves nothing. No-recipe is enforced where it can
//! actually be enforced -- on the sending side, by
//! `manifestation::tests::test_no_recipe_or_keyword_in_manifestation_dispatch`, which reads this
//! repository's own dispatch source and fails if it branches on prompt content.

use std::fmt;
use std::path::{Path, PathBuf};

/// The receipt version this build understands.
///
/// Pinned rather than accepted-if-present: a future Chronos2 that changes the meaning of these
/// fields must be met with a visible refusal, not with Archetypes silently reading v2 numbers
/// under v1 assumptions.
pub const TRIPOSR_RECEIPT_SCHEMA: &str = "chronosophia.triposr-mesh.v1";

/// Below this share of the reference image, subject extraction has collapsed and TripoSR was
/// handed a nearly blank frame.
///
/// Measured across a seven-subject spread (`scripts/manifest_quality_lab.py`): everything that
/// reconstructed recognisably scored 0.0947 or better, everything that came back as noise scored
/// 0.0006. There is no overlap, so a floor separates them. **n = 7** -- recorded in
/// `docs/ledger/2026/09/report_2026-09-12_object_quality_unsolved.md` as under-calibrated.
///
/// The two that collapsed were a white ceramic figure and a cut-crystal decanter: both close in
/// colour to their own backdrop, both erased by the colour-distance mask before reconstruction
/// began. That cause is fixed in Chronos2 now (commit f763050b), which is why this floor should
/// rarely fire -- but a floor that rarely fires is exactly the one worth keeping, because what
/// it catches is noise being handed to the player as their own work.
///
/// **Deliberately not `subject_match.json`.** Chronos2 emits that guard in the same bundle and
/// it is the obvious thing to gate on, so the measurement is recorded here to stop it being
/// reached for again: it scored **0.260** for the destroyed figure and **0.259** for a good
/// astrolabe. It does not discriminate, and gating on it would reject good work and pass noise.
pub const MIN_SUBJECT_COVERAGE: f64 = 0.02;

/// A verified Chronos2 bundle: what was asked for, what governed it, and what was built.
#[derive(Debug, Clone, PartialEq)]
pub struct GameArtifact {
    /// The prompt as the bundle itself records it, not as the caller remembers it.
    pub prompt: String,
    /// The Sentinel's governance verdict on the request.
    pub sentinel_verdict: String,
    /// The mesh TripoSR produced, digest-verified against its own receipt.
    pub mesh: PathBuf,
    pub mesh_sha256: String,
    /// The reference image the mesh was reconstructed from. This is the image the player sees
    /// lying under the finished object, so binding it here is what makes that pairing true
    /// rather than merely adjacent.
    pub source_image: PathBuf,
    pub source_image_sha256: String,
    pub subject_coverage: f64,
}

/// Every way a bundle can fail to be evidence.
///
/// Carries the measurement that failed rather than a sentence about it, so the caller renders
/// one message and the tests assert on the fault.
#[derive(Debug, Clone, PartialEq)]
pub enum ReceiptFault {
    Missing { what: &'static str, path: PathBuf },
    Unreadable { what: &'static str, detail: String },
    SchemaMismatch { found: String },
    SentinelRefused { verdict: String, message: String },
    GovernanceMissing,
    PromptMismatch { asked: String, bundled: String },
    DigestMismatch { what: &'static str, declared: String, measured: String },
    IntegrityBroken { detail: String },
    SubjectLost { coverage: f64 },
}

impl ReceiptFault {
    /// Which pipeline stage the altar should show as the point of failure.
    pub fn stage_id(&self) -> &'static str {
        match self {
            ReceiptFault::SentinelRefused { .. } | ReceiptFault::GovernanceMissing => "sentinel",
            ReceiptFault::SubjectLost { .. } => "subject",
            _ => "geometry_forge",
        }
    }
}

impl fmt::Display for ReceiptFault {
    /// Written for the player standing at the altar, not for a log reader. Each one says what
    /// was expected, what was found, and -- where there is one -- what they can do about it.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReceiptFault::Missing { what, path } => write!(
                formatter,
                "Chronos2 returned no {what}, so there is no evidence anything was built. \
                 Expected it at {}.",
                path.display()
            ),
            ReceiptFault::Unreadable { what, detail } => {
                write!(formatter, "Chronos2's {what} could not be read: {detail}")
            }
            ReceiptFault::SchemaMismatch { found } => write!(
                formatter,
                "This Chronos2 speaks receipt version '{found}', and this build understands \
                 '{TRIPOSR_RECEIPT_SCHEMA}'. Refusing rather than reading its numbers under the \
                 wrong assumptions."
            ),
            ReceiptFault::SentinelRefused { verdict, message } => write!(
                formatter,
                "The Sentinel did not clear this request ({verdict}). {message}"
            ),
            ReceiptFault::GovernanceMissing => write!(
                formatter,
                "This bundle carries no Sentinel verdict at all. A run that was never governed \
                 is not a run this altar will stage."
            ),
            ReceiptFault::PromptMismatch { asked, bundled } => write!(
                formatter,
                "This bundle was built for a different request. You asked for \"{asked}\"; the \
                 bundle records \"{bundled}\". Nothing was staged, because it would not have \
                 been yours."
            ),
            ReceiptFault::DigestMismatch { what, declared, measured } => write!(
                formatter,
                "The {what} does not match the hash Chronos2 recorded for it -- declared \
                 {declared}, measured {measured}. The file changed or was written incompletely \
                 after the receipt was sealed."
            ),
            ReceiptFault::IntegrityBroken { detail } => write!(
                formatter,
                "Chronos2's own integrity check on this bundle did not pass: {detail}"
            ),
            ReceiptFault::SubjectLost { coverage } => write!(
                formatter,
                "The subject could not be separated from its background: only {:.2}% of the \
                 reference survived, so the reconstruction was built from a nearly blank frame. \
                 This usually means the subject was close in colour to its backdrop.",
                coverage * 100.0
            ),
        }
    }
}

fn read_json(path: &Path, what: &'static str) -> Result<serde_json::Value, ReceiptFault> {
    if !path.is_file() {
        return Err(ReceiptFault::Missing { what, path: path.to_path_buf() });
    }
    let body = std::fs::read_to_string(path)
        .map_err(|error| ReceiptFault::Unreadable { what, detail: error.to_string() })?;
    serde_json::from_str(&body)
        .map_err(|error| ReceiptFault::Unreadable { what, detail: error.to_string() })
}

/// **A Chronos2 bundle carries two different digest algorithms, and using the wrong one silently
/// fails every check.**
///
/// `manifest.json`'s `bundle_files` map is written by Chronos2's Rust side, whose `hex_bytes` is
/// `blake3::hash` (`chronos_cli/src/governed_artifact.rs`). `engine_mesh/triposr_artifact.json`
/// is written by the Python emitter, which uses `hashlib.sha256`. Both are 64 lowercase hex
/// characters, so nothing about a hash's *appearance* reveals which one it is, and comparing a
/// blake3 digest to a sha256 one just looks like a corrupt bundle.
///
/// This was found by `the_contract_matches_bundles_chronos2_actually_wrote` running against a
/// real bundle -- every hand-built fixture agreed with itself and passed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Digest {
    /// `engine_mesh/triposr_artifact.json` -- written by the Python TripoSR emitter.
    Sha256,
    /// `manifest.json` `bundle_files` -- written by Chronos2's Rust side.
    Blake3,
}

impl Digest {
    fn of(self, path: &Path) -> Result<String, String> {
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        Ok(match self {
            Digest::Sha256 => {
                use sha2::{Digest as _, Sha256};
                hex::encode(Sha256::digest(&bytes))
            }
            Digest::Blake3 => blake3::hash(&bytes).to_hex().to_string(),
        })
    }
}

/// Hex sha256 of a file, lowercase -- the casing Chronos2's Python side writes.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    Digest::Sha256.of(path)
}

/// Confirms a file's bytes still hash to what the receipt declared, under the algorithm that
/// receipt is written with.
///
/// Compared case-insensitively: Chronos2 writes lowercase and the launcher's own helper writes
/// uppercase, and a digest check that fails on casing teaches everyone to disable it.
fn check_digest(
    what: &'static str,
    path: &Path,
    declared: &str,
    algorithm: Digest,
) -> Result<String, ReceiptFault> {
    let measured = algorithm.of(path).map_err(|detail| ReceiptFault::Unreadable { what, detail })?;
    if !measured.eq_ignore_ascii_case(declared) {
        return Err(ReceiptFault::DigestMismatch {
            what,
            declared: declared.to_string(),
            measured,
        });
    }
    Ok(measured)
}

/// Where TripoSR's mesh lands. Chronos2 has written both layouts across versions.
fn locate_mesh(bundle: &Path) -> Option<PathBuf> {
    let nested = bundle.join("engine_mesh").join("0").join("mesh.obj");
    if nested.is_file() {
        return Some(nested);
    }
    let direct = bundle.join("engine_mesh").join("mesh.obj");
    direct.is_file().then_some(direct)
}

/// Verifies a finished Chronos2 bundle against the prompt that was actually sent.
///
/// Ordered so the *cause* is reported rather than a downstream symptom: governance before
/// content, identity before digests, digests before quality. A refused request should say the
/// Sentinel refused it, not that its mesh is missing.
pub fn verify(bundle: &Path, asked_prompt: &str) -> Result<GameArtifact, ReceiptFault> {
    // 1. Chronos2's own verdict on its own bundle. If it does not vouch for it, nothing further
    //    is worth measuring.
    let integrity = read_json(&bundle.join("integrity_report.json"), "integrity report")?;
    let valid = integrity.get("valid").and_then(|value| value.as_bool()).unwrap_or(false);
    let broken = integrity.get("broken_links").and_then(|value| value.as_i64()).unwrap_or(0);
    let corrupted =
        integrity.get("corrupted_count").and_then(|value| value.as_i64()).unwrap_or(0);
    if !valid || broken != 0 || corrupted != 0 {
        return Err(ReceiptFault::IntegrityBroken {
            detail: format!("valid={valid}, broken_links={broken}, corrupted_events={corrupted}"),
        });
    }

    let manifest = read_json(&bundle.join("manifest.json"), "run manifest")?;
    if !manifest.get("codex_verify_valid").and_then(|value| value.as_bool()).unwrap_or(false) {
        return Err(ReceiptFault::IntegrityBroken {
            detail: "Chronos2's Codex ledger did not verify for this run".to_string(),
        });
    }

    // 2. Governance. A refusal is a *successful* run of the governance path -- it exits zero and
    //    writes a bundle -- so exit status can never distinguish it, and this check is the only
    //    thing that does.
    let decisions = read_json(&bundle.join("decisions.json"), "governance decisions")?;
    let governance = decisions
        .get("decisions")
        .and_then(|value| value.as_array())
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get("kind").and_then(|kind| kind.as_str()) == Some("governance"))
        })
        .ok_or(ReceiptFault::GovernanceMissing)?;
    let verdict = governance
        .get("verdict")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if verdict != "allowed" {
        return Err(ReceiptFault::SentinelRefused {
            verdict,
            message: governance
                .get("msg")
                .and_then(|value| value.as_str())
                .unwrap_or("No reason was recorded.")
                .to_string(),
        });
    }

    // 3. Identity: is this bundle the answer to the question that was asked?
    //
    //    The bundle directory is reused across runs and a crash can leave a previous run's
    //    contents in place. Without this, the altar happily stages the last thing that worked
    //    and the player watches their prompt produce someone else's object.
    let prompt_path = bundle.join("human_prompt.txt");
    if !prompt_path.is_file() {
        return Err(ReceiptFault::Missing { what: "recorded prompt", path: prompt_path });
    }
    let bundled_prompt = std::fs::read_to_string(&prompt_path)
        .map_err(|error| ReceiptFault::Unreadable {
            what: "recorded prompt",
            detail: error.to_string(),
        })?
        .trim()
        .to_string();
    if bundled_prompt != asked_prompt.trim() {
        return Err(ReceiptFault::PromptMismatch {
            asked: asked_prompt.trim().to_string(),
            bundled: bundled_prompt,
        });
    }
    if let Some(declared) = manifest
        .get("bundle_files")
        .and_then(|files| files.get("human_prompt.txt"))
        .and_then(|value| value.as_str())
    {
        check_digest("recorded prompt", &prompt_path, declared, Digest::Blake3)?;
    }

    // 4. The reconstruction receipt, and the bytes it speaks for.
    let receipt_path = bundle.join("engine_mesh").join("triposr_artifact.json");
    let receipt = read_json(&receipt_path, "TripoSR receipt")?;
    let schema = receipt.get("schema").and_then(|value| value.as_str()).unwrap_or_default();
    if schema != TRIPOSR_RECEIPT_SCHEMA {
        return Err(ReceiptFault::SchemaMismatch { found: schema.to_string() });
    }

    let mesh = locate_mesh(bundle).ok_or_else(|| ReceiptFault::Missing {
        what: "reconstructed mesh",
        path: bundle.join("engine_mesh").join("0").join("mesh.obj"),
    })?;
    let declared_mesh = receipt
        .get("mesh")
        .and_then(|value| value.get("sha256"))
        .and_then(|value| value.as_str())
        .ok_or(ReceiptFault::Unreadable {
            what: "TripoSR receipt",
            detail: "no mesh digest recorded".to_string(),
        })?;
    let mesh_sha256 = check_digest("reconstructed mesh", &mesh, declared_mesh, Digest::Sha256)?;

    let source_image = bundle.join("reference_input.png");
    let declared_image = receipt
        .get("source_image")
        .and_then(|value| value.get("sha256"))
        .and_then(|value| value.as_str())
        .ok_or(ReceiptFault::Unreadable {
            what: "TripoSR receipt",
            detail: "no source-image digest recorded".to_string(),
        })?;
    if !source_image.is_file() {
        return Err(ReceiptFault::Missing { what: "reference image", path: source_image });
    }
    let source_image_sha256 =
        check_digest("reference image", &source_image, declared_image, Digest::Sha256)?;

    // 5. Quality, last: a real object built badly is a different failure from no object, and the
    //    player is owed the distinction.
    let subject_coverage = receipt
        .get("preparation")
        .and_then(|value| value.get("subject_coverage"))
        .and_then(|value| value.as_f64())
        .unwrap_or(1.0);
    if subject_coverage < MIN_SUBJECT_COVERAGE {
        return Err(ReceiptFault::SubjectLost { coverage: subject_coverage });
    }

    Ok(GameArtifact {
        prompt: bundled_prompt,
        sentinel_verdict: verdict,
        mesh,
        mesh_sha256,
        source_image,
        source_image_sha256,
        subject_coverage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// A bundle laid out exactly as `first-light --geometry-forge --void` writes one.
    ///
    /// Built from real bytes and real hashes rather than from fixtures with the digests typed
    /// in, so a test that passes proves the digest path actually computed something.
    struct Bundle {
        root: PathBuf,
    }

    impl Bundle {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "archetypes-receipt-{name}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(root.join("engine_mesh").join("0")).unwrap();
            let bundle = Bundle { root };
            bundle.write_good("a brass astrolabe", 0.4456);
            bundle
        }

        fn path(&self) -> &Path {
            &self.root
        }

        fn write_good(&self, prompt: &str, coverage: f64) {
            let root = &self.root;
            fs::write(root.join("integrity_report.json"), r#"{"valid":true,"broken_links":0,"corrupted_count":0}"#).unwrap();
            fs::write(root.join("decisions.json"), r#"{"decisions":[{"id":"village_protection","kind":"governance","msg":"cleared","verdict":"allowed"}]}"#).unwrap();

            fs::write(root.join("human_prompt.txt"), prompt).unwrap();
            // The manifest's bundle_files map is blake3 on the real thing, so the fixture
            // must be too -- a sha256 here would make every test agree with a contract
            // Chronos2 does not implement.
            let prompt_hash = Digest::Blake3.of(&root.join("human_prompt.txt")).unwrap();

            let mesh = root.join("engine_mesh").join("0").join("mesh.obj");
            fs::write(&mesh, "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
            let mesh_hash = sha256_file(&mesh).unwrap();

            let image = root.join("reference_input.png");
            fs::write(&image, b"not really a png, but real bytes with a real hash").unwrap();
            let image_hash = sha256_file(&image).unwrap();

            fs::write(
                root.join("manifest.json"),
                format!(
                    r#"{{"codex_verify_valid":true,"bundle_files":{{"human_prompt.txt":"{prompt_hash}"}}}}"#
                ),
            )
            .unwrap();
            fs::write(
                root.join("engine_mesh").join("triposr_artifact.json"),
                format!(
                    r#"{{"schema":"{TRIPOSR_RECEIPT_SCHEMA}","source_image":{{"sha256":"{image_hash}"}},"mesh":{{"sha256":"{mesh_hash}"}},"preparation":{{"subject_coverage":{coverage}}}}}"#
                ),
            )
            .unwrap();
        }

        fn put(&self, relative: &str, body: &str) {
            fs::write(self.root.join(relative), body).unwrap();
        }
    }

    impl Drop for Bundle {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn a_complete_bundle_verifies_and_reports_what_it_contains() {
        let bundle = Bundle::new("good");
        let artifact = verify(bundle.path(), "a brass astrolabe").expect("bundle verifies");
        assert_eq!(artifact.prompt, "a brass astrolabe");
        assert_eq!(artifact.sentinel_verdict, "allowed");
        assert_eq!(artifact.subject_coverage, 0.4456);
        assert!(artifact.mesh.ends_with("mesh.obj"));
        assert_eq!(artifact.mesh_sha256.len(), 64);
        assert_eq!(artifact.source_image_sha256.len(), 64);
    }

    /// The prompt is compared after trimming, because the bridge sends a trimmed prompt and
    /// Chronos2 writes the file with a trailing newline on some paths.
    ///
    /// Written through `write_good` so the manifest digest covers the same bytes -- editing the
    /// file alone would be a state Chronos2 cannot produce, and would be caught by the digest
    /// check instead, which is a different test.
    #[test]
    fn surrounding_whitespace_is_not_a_different_request() {
        let bundle = Bundle::new("whitespace");
        bundle.write_good("a brass astrolabe\n", 0.4456);
        assert!(verify(bundle.path(), "  a brass astrolabe  ").is_ok());
    }

    /// The manifest's own digest for the prompt is enforced, so a bundle whose prompt file was
    /// edited after sealing is refused even when the edited text is what was asked for.
    #[test]
    fn a_prompt_file_edited_after_sealing_is_refused() {
        let bundle = Bundle::new("edited-prompt");
        bundle.put("human_prompt.txt", "a brass astrolabe");
        fs::write(bundle.path().join("human_prompt.txt"), "a brass astrolabe ").unwrap();
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(
            matches!(&fault, ReceiptFault::DigestMismatch { what, .. } if *what == "recorded prompt"),
            "{fault:?}"
        );
    }

    /// The failure that motivated the whole module: a stale bundle from an earlier prompt has
    /// every file the old check looked for, and passes it.
    #[test]
    fn a_bundle_left_over_from_another_prompt_is_refused() {
        let bundle = Bundle::new("stale");
        let fault = verify(bundle.path(), "a cut crystal decanter").unwrap_err();
        assert!(
            matches!(&fault, ReceiptFault::PromptMismatch { bundled, .. } if bundled == "a brass astrolabe"),
            "{fault:?}"
        );
        // The player is told whose object it would have been, not just that something failed.
        let rendered = fault.to_string();
        assert!(rendered.contains("a cut crystal decanter") && rendered.contains("a brass astrolabe"));
    }

    /// A refusal is a *successful* process exit. Exit status cannot distinguish it; this can.
    #[test]
    fn a_sentinel_refusal_is_reported_as_a_refusal_not_as_a_missing_mesh() {
        let bundle = Bundle::new("refused");
        bundle.put(
            "decisions.json",
            r#"{"decisions":[{"id":"village_protection","kind":"governance","msg":"This request was declined.","verdict":"refused"}]}"#,
        );
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(matches!(fault, ReceiptFault::SentinelRefused { .. }), "{fault:?}");
        assert_eq!(fault.stage_id(), "sentinel");
        assert!(fault.to_string().contains("This request was declined."));
    }

    /// An ungoverned run is not a run this altar stages, and that is a different fault from a
    /// refusal -- one has a verdict, the other has none.
    #[test]
    fn a_bundle_with_no_governance_verdict_at_all_is_refused() {
        let bundle = Bundle::new("ungoverned");
        bundle.put("decisions.json", r#"{"decisions":[{"id":"triposr","kind":"engine","verdict":"chosen"}]}"#);
        assert_eq!(verify(bundle.path(), "a brass astrolabe").unwrap_err(), ReceiptFault::GovernanceMissing);
    }

    /// A mesh truncated by a crash mid-write is still a file, and the old presence check passed
    /// it. Its bytes no longer hash to what the sealed receipt declared.
    #[test]
    fn a_mesh_that_changed_after_the_receipt_was_sealed_is_refused() {
        let bundle = Bundle::new("truncated");
        fs::write(bundle.path().join("engine_mesh").join("0").join("mesh.obj"), "v 0 0 0\n").unwrap();
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(
            matches!(&fault, ReceiptFault::DigestMismatch { what, .. } if *what == "reconstructed mesh"),
            "{fault:?}"
        );
    }

    /// The reference image is what the player sees lying under the finished object. If it is not
    /// the image the mesh was built from, that pairing is a lie.
    #[test]
    fn a_reference_image_that_is_not_the_one_the_mesh_came_from_is_refused() {
        let bundle = Bundle::new("swapped-image");
        fs::write(bundle.path().join("reference_input.png"), b"a completely different picture").unwrap();
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(
            matches!(&fault, ReceiptFault::DigestMismatch { what, .. } if *what == "reference image"),
            "{fault:?}"
        );
    }

    /// Reading v2 numbers under v1 assumptions is the silent failure this pin exists to prevent.
    #[test]
    fn an_unknown_receipt_version_is_refused_rather_than_guessed_at() {
        let bundle = Bundle::new("v2");
        bundle.put(
            "engine_mesh/triposr_artifact.json",
            r#"{"schema":"chronosophia.triposr-mesh.v2","mesh":{"sha256":"x"},"source_image":{"sha256":"y"}}"#,
        );
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(matches!(fault, ReceiptFault::SchemaMismatch { .. }), "{fault:?}");
        assert!(fault.to_string().contains("v2"));
    }

    #[test]
    fn chronos_declining_to_vouch_for_its_own_bundle_is_refused() {
        let bundle = Bundle::new("integrity");
        bundle.put("integrity_report.json", r#"{"valid":false,"broken_links":2,"corrupted_count":1}"#);
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(matches!(fault, ReceiptFault::IntegrityBroken { .. }), "{fault:?}");
        assert!(fault.to_string().contains("broken_links=2"));
    }

    /// The measured floor from the quality lab. Noise must not reach the altar dressed as the
    /// player's work.
    #[test]
    fn a_reconstruction_built_from_a_blank_frame_is_refused_with_its_measurement() {
        let bundle = Bundle::new("blank");
        bundle.write_good("a brass astrolabe", 0.0006);
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert_eq!(fault, ReceiptFault::SubjectLost { coverage: 0.0006 });
        assert_eq!(fault.stage_id(), "subject");
        assert!(fault.to_string().contains("0.06%"), "{fault}");
    }

    /// Everything that reconstructed recognisably in the seven-subject spread must still pass.
    /// A gate that also rejects good work is not a gate, it is an outage.
    #[test]
    fn the_coverage_floor_passes_every_subject_that_measured_well() {
        let bundle = Bundle::new("floor");
        for coverage in [0.0947, 0.1298, 0.1403, 0.1932, 0.2280, 0.4456, 0.6024] {
            bundle.write_good("a brass astrolabe", coverage);
            assert!(
                verify(bundle.path(), "a brass astrolabe").is_ok(),
                "coverage {coverage} reconstructed recognisably and must not be refused"
            );
        }
    }

    /// **The test that proves this contract is real.**
    ///
    /// Every other test in this module builds its own fixture, so all of them would still pass
    /// if every field name here were wrong -- they would be consistently wrong on both sides.
    /// This one reads bundles Chronos2 actually wrote, via the same
    /// `first-light --geometry-forge --void` the bridge runs, and is the only thing standing
    /// between this module and a contract agreed with nobody.
    ///
    /// Skips rather than fails where the lab bundles are absent: they are machine-local
    /// evidence, not repository fixtures, and a checkout without them is not a broken build.
    #[test]
    fn the_contract_matches_bundles_chronos2_actually_wrote() {
        let lab = Path::new("../../artifacts/quality-lab/bundles");
        let Ok(entries) = fs::read_dir(lab) else {
            eprintln!("no local Chronos2 bundles at {}; contract unverified here", lab.display());
            return;
        };

        let mut checked = 0;
        for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
            let bundle = entry.path();
            let Ok(prompt) = fs::read_to_string(bundle.join("human_prompt.txt")) else {
                continue;
            };
            match verify(&bundle, prompt.trim()) {
                Ok(artifact) => {
                    assert_eq!(artifact.sentinel_verdict, "allowed");
                    assert_eq!(artifact.mesh_sha256.len(), 64, "mesh digest not read from a real receipt");
                    assert!(artifact.subject_coverage > 0.0);
                }
                // The two known-bad subjects in the lab spread are *supposed* to be refused, and
                // refused for the measured reason. Any other fault means a field name is wrong.
                Err(ReceiptFault::SubjectLost { coverage }) => {
                    assert!(coverage < MIN_SUBJECT_COVERAGE);
                }
                Err(fault) => panic!("{}: {fault:?}", bundle.display()),
            }
            checked += 1;
        }
        assert!(checked > 0, "bundle directory exists but held nothing to check");
        eprintln!("contract verified against {checked} real Chronos2 bundle(s)");
    }

    /// An absent bundle must name what was missing. "Manifestation failed" sends the operator
    /// reading logs; naming the file does not.
    #[test]
    fn a_missing_bundle_names_the_file_it_wanted() {
        let fault = verify(Path::new("no-such-bundle-anywhere"), "anything").unwrap_err();
        assert!(matches!(&fault, ReceiptFault::Missing { what, .. } if *what == "integrity report"), "{fault:?}");
        assert!(fault.to_string().contains("integrity_report.json"));
    }
}
