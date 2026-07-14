use bevy::prelude::*;
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{OracleSession, OracleState};
use crate::modes::difficulty::Difficulty;
use crate::services::chronos::{request_chronos_artifact, ArtifactOutcome};
use crate::services::llm::embed;

pub struct OracleScoringPlugin;

impl Plugin for OracleScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(OracleState::Generating), start_generation)
            .add_systems(
                Update,
                poll_generation.run_if(in_state(OracleState::Generating)),
            )
            .add_systems(OnEnter(OracleState::Scoring), start_scoring)
            .add_systems(Update, poll_scoring.run_if(in_state(OracleState::Scoring)));
    }
}

#[derive(Resource)]
struct AsyncGeneration {
    receiver: Mutex<mpsc::Receiver<ArtifactOutcome>>,
}

#[derive(Resource)]
pub struct AsyncScoring {
    receiver: Mutex<mpsc::Receiver<Result<Vec<f32>, String>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OraclePrompt {
    pub difficulty: Difficulty,
    pub words: [&'static str; 3],
}

impl OraclePrompt {
    fn words_vec(self) -> Vec<String> {
        self.words.iter().map(|word| (*word).to_owned()).collect()
    }

    fn image_prompt(self) -> String {
        format!(
            "A readable symbolic image evoking exactly three prompt anchors: {}, {}, {}. No text, no letters, no captions.",
            self.words[0], self.words[1], self.words[2]
        )
    }
}

const ORACLE_PROMPTS: [OraclePrompt; 8] = [
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Dog", "Red", "Running"],
    },
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Moon", "Blue", "Water"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Memory", "Silver", "Door"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Hope", "Broken", "Bridge"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Signal", "Ash", "Garden"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Witness", "Glass", "Thunder"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Entropy", "Memory", "Bloom"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Silence", "Hunger", "Star"],
    },
];

fn start_generation(mut commands: Commands) {
    let oracle_prompt = prompt_for_round(round_seed());
    let session = OracleSession {
        difficulty: oracle_prompt.difficulty,
        target_words: oracle_prompt.words_vec(),
        ..default()
    };
    commands.insert_resource(session);

    let prompt = oracle_prompt.image_prompt();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(request_chronos_artifact(&prompt));
    });
    commands.insert_resource(AsyncGeneration {
        receiver: Mutex::new(rx),
    });
}

fn poll_generation(
    mut commands: Commands,
    async_gen: Option<Res<AsyncGeneration>>,
    mut session: ResMut<OracleSession>,
    mut next_state: ResMut<NextState<OracleState>>,
) {
    if let Some(gen) = async_gen {
        if let Ok(outcome) = gen.receiver.lock().unwrap().try_recv() {
            commands.remove_resource::<AsyncGeneration>();
            session.outcome = Some(outcome.clone());
            if outcome.status == "complete" {
                if let Some(path) = outcome.png_path {
                    session.image_path = Some(path);
                    next_state.set(OracleState::Guessing);
                } else {
                    session.error_msg = Some("Completed without a valid image path".to_owned());
                    next_state.set(OracleState::Result);
                }
            } else {
                session.error_msg = Some(outcome.detail.clone());
                next_state.set(OracleState::Result);
            }
        }
    }
}

fn start_scoring(mut commands: Commands, session: Res<OracleSession>) {
    let target = session.target_words.clone();
    let guess = session.guess_words.clone();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(score_guess(&target, &guess));
    });
    commands.insert_resource(AsyncScoring {
        receiver: Mutex::new(rx),
    });
}

fn poll_scoring(
    mut commands: Commands,
    async_score: Option<Res<AsyncScoring>>,
    mut session: ResMut<OracleSession>,
    mut next_state: ResMut<NextState<OracleState>>,
) {
    if let Some(score) = async_score {
        if let Ok(result) = score.receiver.lock().unwrap().try_recv() {
            commands.remove_resource::<AsyncScoring>();
            match result {
                Ok(scores) => {
                    session.scores = scores;
                }
                Err(e) => {
                    session.error_msg = Some(format!("Scoring failed: {}", e));
                }
            }
            next_state.set(OracleState::Result);
        }
    }
}

pub fn score_guess(target: &[String], guess: &[String]) -> Result<Vec<f32>, String> {
    if target.len() != guess.len() {
        return Err("Word count mismatch".into());
    }
    let mut target_embeddings = Vec::with_capacity(target.len());
    let mut guess_embeddings = Vec::with_capacity(guess.len());
    for (t, g) in target.iter().zip(guess.iter()) {
        target_embeddings.push(embed(t)?);
        guess_embeddings.push(embed(g)?);
    }
    score_vectors(&target_embeddings, &guess_embeddings)
}

pub fn score_vectors(target: &[Vec<f32>], guess: &[Vec<f32>]) -> Result<Vec<f32>, String> {
    if target.len() != guess.len() {
        return Err("Word count mismatch".into());
    }
    target
        .iter()
        .zip(guess.iter())
        .map(|(target_vector, guess_vector)| {
            if target_vector.len() != guess_vector.len() {
                Err("Embedding dimension mismatch".to_owned())
            } else {
                Ok(cosine_similarity(target_vector, guess_vector))
            }
        })
        .collect()
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

pub fn prompt_for_round(seed: u64) -> OraclePrompt {
    ORACLE_PROMPTS[(seed as usize) % ORACLE_PROMPTS.len()]
}

fn round_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cosine_similarity_handles_exact_orthogonal_and_opposed_vectors() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);

        let d = vec![-1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &d), -1.0);
    }

    #[test]
    fn prompt_selection_has_three_words_and_all_difficulty_tiers() {
        let mut difficulties = Vec::new();
        for index in 0..ORACLE_PROMPTS.len() {
            let prompt = prompt_for_round(index as u64);
            assert_eq!(prompt.words.len(), 3);
            difficulties.push(prompt.difficulty);
        }
        assert!(difficulties.contains(&Difficulty::Literal));
        assert!(difficulties.contains(&Difficulty::Metaphorical));
        assert!(difficulties.contains(&Difficulty::Obscured));
        assert!(difficulties.contains(&Difficulty::Abyssal));
        assert!(ORACLE_PROMPTS
            .iter()
            .any(|prompt| prompt.words == ["Entropy", "Memory", "Bloom"]));
    }

    #[test]
    fn scoring_rejects_word_count_mismatch() {
        let target = vec!["Dog".to_owned(), "Red".to_owned()];
        let guess = vec!["Dog".to_owned()];
        assert!(score_guess(&target, &guess).is_err());
    }

    #[test]
    fn vector_scoring_proves_exact_and_partial_matches_without_ollama() {
        let target = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0],
            vec![1.0, 1.0, 0.0],
        ];
        let exact = score_vectors(&target, &target).unwrap();
        assert!(exact.iter().all(|score| (score - 1.0).abs() < 0.0001));

        let partial = score_vectors(
            &target,
            &[
                vec![1.0, 0.0, 0.0],
                vec![1.0, 0.0, 0.0],
                vec![1.0, 1.0, 0.0],
            ],
        )
        .unwrap();
        assert!((partial[0] - 1.0).abs() < 0.0001);
        assert_eq!(partial[1], 0.0);
        assert!((partial[2] - 1.0).abs() < 0.0001);
    }
}
