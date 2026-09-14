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
pub const MULTIVIEW_RECEIPT_SCHEMA: &str = "chronosophia.multiview-mesh.v1";
/// Hunyuan3D-2 single-image lane (operator-local evaluation, 2026-09-13). Same evidence shape as
/// the TripoSR receipt: one digest-bound source image, a mesh digest, and subject coverage.
pub const HUNYUAN_RECEIPT_SCHEMA: &str = "chronosophia.hunyuan3d-mesh.v1";

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
/// This is independent of `subject_match.json`: coverage catches a missing silhouette, while
/// subject match catches a completed reconstruction that Chronos2 itself judged to be the wrong
/// object. Both have to pass. A weak historical scorer is a reason to improve the scorer, not a
/// reason to import an artifact whose machine-readable verdict is explicitly `matches=false`.
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
    SubjectRejected {
        checked: bool,
        matches: bool,
        score: Option<f64>,
        reason: String,
    },
    ReconstructionContract { detail: String },
}

impl ReceiptFault {
    /// Which pipeline stage the altar should show as the point of failure.
    pub fn stage_id(&self) -> &'static str {
        match self {
            ReceiptFault::SentinelRefused { .. } | ReceiptFault::GovernanceMissing => "sentinel",
            ReceiptFault::SubjectLost { .. } | ReceiptFault::SubjectRejected { .. } => "subject",
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
                 '{TRIPOSR_RECEIPT_SCHEMA}', '{MULTIVIEW_RECEIPT_SCHEMA}' and \
                 '{HUNYUAN_RECEIPT_SCHEMA}'. Refusing rather than reading its numbers under the \
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
            ReceiptFault::SubjectRejected {
                checked,
                matches,
                score,
                reason,
            } => write!(
                formatter,
                "Chronos2 rejected its own reconstruction: checked={checked}, matches={matches}, \
                 score={}. {reason} Nothing was imported or saved to your object library.",
                score
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "not recorded".to_string())
            ),
            ReceiptFault::ReconstructionContract { detail } => write!(
                formatter,
                "Chronos2 did not prove a complete three-view reconstruction: {detail}"
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

    // 4. The finished-object judgment. Generation is still attempted for every Sentinel-cleared
    //    prompt; this is a post-generation acceptance gate. `matches=false` used to be advisory,
    //    which allowed an artifact Chronos2 had explicitly rejected to enter the permanent game
    //    library. Missing or unchecked evidence fails closed for the same reason: absence of a
    //    passing judgment is not evidence of a passing object.
    let subject_match = read_json(&bundle.join("subject_match.json"), "subject-match judgment")?;
    if let Some(declared) = manifest
        .get("bundle_files")
        .and_then(|files| files.get("subject_match.json"))
        .and_then(|value| value.as_str())
    {
        check_digest(
            "subject-match judgment",
            &bundle.join("subject_match.json"),
            declared,
            Digest::Blake3,
        )?;
    }
    let checked = subject_match
        .get("checked")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let matches = subject_match
        .get("matches")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    if !checked || !matches {
        return Err(ReceiptFault::SubjectRejected {
            checked,
            matches,
            score: subject_match.get("score").and_then(|value| value.as_f64()),
            reason: subject_match
                .get("reason")
                .and_then(|value| value.as_str())
                .unwrap_or("No passing subject-match reason was recorded.")
                .to_string(),
        });
    }

    // 5. The reconstruction receipt, and the bytes it speaks for.
    let hunyuan_path = bundle.join("engine_mesh").join("hunyuan_artifact.json");
    let multiview_path = bundle.join("engine_mesh").join("multiview_artifact.json");
    let (receipt_path, receipt_name) = if hunyuan_path.is_file() {
        (hunyuan_path, "Hunyuan3D reconstruction receipt")
    } else if multiview_path.is_file() {
        (multiview_path, "multi-view reconstruction receipt")
    } else {
        (
            bundle.join("engine_mesh").join("triposr_artifact.json"),
            "legacy TripoSR receipt",
        )
    };
    let receipt = read_json(&receipt_path, receipt_name)?;
    let schema = receipt.get("schema").and_then(|value| value.as_str()).unwrap_or_default();
    // Hunyuan3D is single-image like TripoSR, so it shares the single-source branch below:
    // one digest-bound reference and a recorded subject coverage.
    if schema != TRIPOSR_RECEIPT_SCHEMA
        && schema != MULTIVIEW_RECEIPT_SCHEMA
        && schema != HUNYUAN_RECEIPT_SCHEMA
    {
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
            what: receipt_name,
            detail: "no mesh digest recorded".to_string(),
        })?;
    let mesh_sha256 = check_digest("reconstructed mesh", &mesh, declared_mesh, Digest::Sha256)?;

    let (source_image, source_image_sha256, subject_coverage) =
        if schema == MULTIVIEW_RECEIPT_SCHEMA {
            let view_count = receipt.get("view_count").and_then(|value| value.as_u64());
            let consumed = receipt
                .get("consumed_view_count")
                .and_then(|value| value.as_u64());
            let closed = receipt
                .get("mesh")
                .and_then(|value| value.get("closed"))
                .and_then(|value| value.as_bool());
            let boundary_edges = receipt
                .get("mesh")
                .and_then(|value| value.get("boundary_edges"))
                .and_then(|value| value.as_u64());
            if view_count != Some(3)
                || consumed != Some(3)
                || closed != Some(true)
                || boundary_edges != Some(0)
            {
                return Err(ReceiptFault::ReconstructionContract {
                    detail: format!(
                        "view_count={view_count:?}, consumed_view_count={consumed:?}, closed={closed:?}, boundary_edges={boundary_edges:?}"
                    ),
                });
            }
            let mut minimum_coverage = 1.0_f64;
            let mut front = None;
            for name in ["front", "right", "top"] {
                let view = receipt
                    .get("source_views")
                    .and_then(|views| views.get(name))
                    .ok_or_else(|| ReceiptFault::ReconstructionContract {
                        detail: format!("the {name} source view is absent from the receipt"),
                    })?;
                let declared = view
                    .get("sha256")
                    .and_then(|value| value.as_str())
                    .ok_or_else(|| ReceiptFault::ReconstructionContract {
                        detail: format!("the {name} source view has no digest"),
                    })?;
                let path = bundle.join(format!("reference_{name}.png"));
                let measured = check_digest(
                    match name {
                        "front" => "front reconstruction view",
                        "right" => "right reconstruction view",
                        _ => "top reconstruction view",
                    },
                    &path,
                    declared,
                    Digest::Sha256,
                )?;
                minimum_coverage = minimum_coverage.min(
                    view.get("subject_coverage")
                        .and_then(|value| value.as_f64())
                        .unwrap_or(0.0),
                );
                if name == "front" {
                    front = Some((path, measured));
                }
            }
            let (path, digest) = front.expect("front is one of the fixed reconstruction views");
            (path, digest, minimum_coverage)
        } else {
            let source_image = bundle.join("reference_input.png");
            let declared_image = receipt
                .get("source_image")
                .and_then(|value| value.get("sha256"))
                .and_then(|value| value.as_str())
                .ok_or(ReceiptFault::Unreadable {
                    what: receipt_name,
                    detail: "no source-image digest recorded".to_string(),
                })?;
            let digest = check_digest(
                "reference image",
                &source_image,
                declared_image,
                Digest::Sha256,
            )?;
            let coverage = receipt
                .get("preparation")
                .and_then(|value| value.get("subject_coverage"))
                .and_then(|value| value.as_f64())
                .unwrap_or(1.0);
            (source_image, digest, coverage)
        };

    // 6. Quality, last: a real object built badly is a different failure from no object, and the
    //    player is owed the distinction.
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
            fs::write(
                root.join("subject_match.json"),
                r#"{"schema":"chronos.subject_match.v1","checked":true,"matches":true,"score":0.88,"reason":"all reconstruction views passed"}"#,
            )
            .unwrap();

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

    /// The Hunyuan3D lane writes its own receipt beside the mesh. It is single-image, so it must
    /// verify through the same source-image digest and coverage checks as TripoSR, and its
    /// digests must still be enforced.
    #[test]
    fn a_hunyuan3d_receipt_verifies_and_its_mesh_digest_is_enforced() {
        let bundle = Bundle::new("hunyuan");
        let engine = bundle.path().join("engine_mesh");
        let mesh_hash = sha256_file(&engine.join("0").join("mesh.obj")).unwrap();
        let image_hash = sha256_file(&bundle.path().join("reference_input.png")).unwrap();
        fs::remove_file(engine.join("triposr_artifact.json")).unwrap();
        fs::write(
            engine.join("hunyuan_artifact.json"),
            format!(
                r#"{{"schema":"{HUNYUAN_RECEIPT_SCHEMA}","source_image":{{"sha256":"{image_hash}"}},"mesh":{{"sha256":"{mesh_hash}"}},"preparation":{{"subject_coverage":0.3395}}}}"#
            ),
        )
        .unwrap();
        let artifact = verify(bundle.path(), "a brass astrolabe").expect("Hunyuan3D bundle verifies");
        assert_eq!(artifact.subject_coverage, 0.3395);

        fs::write(engine.join("0").join("mesh.obj"), "v 9 9 9\n").unwrap();
        assert!(matches!(
            verify(bundle.path(), "a brass astrolabe").unwrap_err(),
            ReceiptFault::DigestMismatch { what: "reconstructed mesh", .. }
        ));
    }

    #[test]
    fn a_completed_artifact_with_matches_false_is_never_imported() {
        let bundle = Bundle::new("subject-false");
        bundle.put(
            "subject_match.json",
            r#"{"schema":"chronos.subject_match.v1","checked":true,"matches":false,"score":0.5024,"reason":"render did not preserve the requested subject"}"#,
        );
        let fault = verify(bundle.path(), "a brass astrolabe").unwrap_err();
        assert!(
            matches!(
                &fault,
                ReceiptFault::SubjectRejected {
                    checked: true,
                    matches: false,
                    score: Some(score),
                    ..
                } if (*score - 0.5024).abs() < f64::EPSILON
            ),
            "{fault:?}"
        );
        assert_eq!(fault.stage_id(), "subject");
        assert!(fault.to_string().contains("Nothing was imported or saved"));
    }

    #[test]
    fn an_unchecked_or_missing_subject_judgment_fails_closed() {
        let unchecked = Bundle::new("subject-unchecked");
        unchecked.put(
            "subject_match.json",
            r#"{"schema":"chronos.subject_match.v1","checked":false,"matches":true}"#,
        );
        assert!(matches!(
            verify(unchecked.path(), "a brass astrolabe").unwrap_err(),
            ReceiptFault::SubjectRejected { checked: false, .. }
        ));

        let missing = Bundle::new("subject-missing");
        fs::remove_file(missing.path().join("subject_match.json")).unwrap();
        assert!(matches!(
            verify(missing.path(), "a brass astrolabe").unwrap_err(),
            ReceiptFault::Missing { what: "subject-match judgment", .. }
        ));
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
                // Historical bundles that already recorded `matches=false` are now expected
                // rejections. Keeping them in the lab proves this gate closes the exact hole
                // that used to import them.
                Err(ReceiptFault::SubjectRejected {
                    checked: true,
                    matches: false,
                    ..
                }) => {}
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
