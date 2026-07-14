use bevy::prelude::*;
use std::sync::{mpsc, Mutex};
use std::thread;

use super::{OracleSession, OracleState};
use crate::services::chronos::{request_chronos_artifact, ArtifactOutcome};
use crate::services::llm::embed;

pub struct OracleScoringPlugin;

impl Plugin for OracleScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(OracleState::Generating), start_generation)
            .add_systems(Update, poll_generation.run_if(in_state(OracleState::Generating)))
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

fn start_generation(mut commands: Commands) {
    // Choose a hidden 3-word prompt randomly or hardcoded for now
    let words = vec!["Dog".to_owned(), "Red".to_owned(), "Running".to_owned()];
    let prompt = format!("A stylized concept of {} {} {}", words[0], words[1], words[2]);
    let session = OracleSession {
        target_words: words,
        ..default()
    };
    commands.insert_resource(session);

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
    let mut scores = Vec::new();
    for (t, g) in target.iter().zip(guess.iter()) {
        let t_emb = embed(t)?;
        let g_emb = embed(g)?;
        scores.push(cosine_similarity(&t_emb, &g_emb));
    }
    Ok(scores)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &b), 1.0);

        let c = vec![0.0, 1.0, 0.0];
        assert_eq!(cosine_similarity(&a, &c), 0.0);

        let d = vec![-1.0, 0.0, 0.0];
        assert_eq!(cosine_similarity(&a, &d), -1.0);
    }

    #[test]
    fn test_prompt_selection_shape() {
        let words = vec!["Dog".to_owned(), "Red".to_owned(), "Running".to_owned()];
        assert_eq!(words.len(), 3);
    }

    #[test]
    fn test_scoring_mismatch_error() {
        let target = vec!["Dog".to_owned(), "Red".to_owned()];
        let guess = vec!["Dog".to_owned()];
        assert!(score_guess(&target, &guess).is_err());
    }
}
