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
    receiver: Mutex<mpsc::Receiver<Result<ScoredGuess, String>>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScoredGuess {
    pub scores: Vec<f32>,
    pub matched_guess_words: Vec<String>,
    pub total_score: f32,
    pub reward_points: u32,
    pub reward_label: &'static str,
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
            "Create a clear visual guessing-game image with exactly three obvious clues. Hidden answer words: {}, {}, {}. Make the first word the main subject, make the second word visually obvious as color/material/condition, and make the third word visible as action, setting, or time of day. Literal, readable, not symbolic. No text, no letters, no captions.",
            self.words[0], self.words[1], self.words[2]
        )
    }
}

const ORACLE_PROMPTS: [OraclePrompt; 16] = [
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Dog", "Red", "Running"],
    },
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Moon", "Blue", "Water"],
    },
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Boat", "Blue", "Sunset"],
    },
    OraclePrompt {
        difficulty: Difficulty::Literal,
        words: ["Tree", "Green", "Lightning"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Cat", "Gold", "Window"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Castle", "Snow", "Night"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Clock", "Broken", "Forest"],
    },
    OraclePrompt {
        difficulty: Difficulty::Metaphorical,
        words: ["Bird", "White", "Cage"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Robot", "Rusty", "Garden"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Mask", "Glass", "Rain"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Key", "Silver", "Ocean"],
    },
    OraclePrompt {
        difficulty: Difficulty::Obscured,
        words: ["Lantern", "Orange", "Fog"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Dragon", "Silver", "Storm"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Skull", "Flower", "Stars"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Tower", "Black", "Fire"],
    },
    OraclePrompt {
        difficulty: Difficulty::Abyssal,
        words: ["Eye", "Crystal", "Desert"],
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
                    session.scores = scores.scores;
                    session.matched_guess_words = scores.matched_guess_words;
                    session.total_score = scores.total_score;
                    session.reward_points = scores.reward_points;
                    session.reward_label = scores.reward_label.to_owned();
                }
                Err(e) => {
                    session.error_msg = Some(format!("Scoring failed: {}", e));
                }
            }
            next_state.set(OracleState::Result);
        }
    }
}

pub fn score_guess(target: &[String], guess: &[String]) -> Result<ScoredGuess, String> {
    if target.len() != guess.len() {
        return Err("Word count mismatch".into());
    }
    let lexical = score_word_matrix(target, guess, None, None);
    let lexical_result = scored_result_from_matrix(&lexical, guess);
    if lexical_result.total_score >= 0.80 {
        return Ok(lexical_result);
    }
    let matrix = match embedding_matrix(target, guess) {
        Ok((target_embeddings, guess_embeddings)) => score_word_matrix(
            target,
            guess,
            Some(&target_embeddings),
            Some(&guess_embeddings),
        ),
        Err(_) => lexical,
    };
    Ok(scored_result_from_matrix(&matrix, guess))
}

fn scored_result_from_matrix(matrix: &[Vec<f32>], guess: &[String]) -> ScoredGuess {
    let (scores, matched_guess_words) = best_assignment_scores(&matrix, guess);
    let total_score = average_score(&scores);
    let (reward_points, reward_label) = reward_for_score(total_score);
    ScoredGuess {
        scores,
        matched_guess_words,
        total_score,
        reward_points,
        reward_label,
    }
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

fn embedding_matrix(
    target: &[String],
    guess: &[String],
) -> Result<(Vec<Vec<f32>>, Vec<Vec<f32>>), String> {
    let mut target_embeddings = Vec::with_capacity(target.len());
    let mut guess_embeddings = Vec::with_capacity(guess.len());
    for word in target {
        target_embeddings.push(embed(word)?);
    }
    for word in guess {
        guess_embeddings.push(embed(word)?);
    }
    Ok((target_embeddings, guess_embeddings))
}

fn score_word_matrix(
    target: &[String],
    guess: &[String],
    target_embeddings: Option<&[Vec<f32>]>,
    guess_embeddings: Option<&[Vec<f32>]>,
) -> Vec<Vec<f32>> {
    target
        .iter()
        .enumerate()
        .map(|(target_index, target_word)| {
            guess
                .iter()
                .enumerate()
                .map(|(guess_index, guess_word)| {
                    let lexical = lexical_word_score(target_word, guess_word);
                    let semantic = target_embeddings
                        .and_then(|target_vectors| target_vectors.get(target_index))
                        .zip(
                            guess_embeddings
                                .and_then(|guess_vectors| guess_vectors.get(guess_index)),
                        )
                        .map(|(a, b)| cosine_similarity(a, b).clamp(0.0, 1.0))
                        .unwrap_or(0.0);
                    lexical.max(semantic)
                })
                .collect()
        })
        .collect()
}

fn best_assignment_scores(matrix: &[Vec<f32>], guess: &[String]) -> (Vec<f32>, Vec<String>) {
    let count = matrix.len();
    let mut best_order = Vec::new();
    let mut best_score = f32::MIN;
    let mut current = Vec::new();
    let mut used = vec![false; count];
    search_assignment(
        matrix,
        0,
        &mut used,
        &mut current,
        0.0,
        &mut best_score,
        &mut best_order,
    );
    let scores = best_order
        .iter()
        .enumerate()
        .map(|(target_index, guess_index)| matrix[target_index][*guess_index])
        .collect();
    let matched_guess_words = best_order
        .iter()
        .map(|guess_index| guess.get(*guess_index).cloned().unwrap_or_default())
        .collect();
    (scores, matched_guess_words)
}

fn search_assignment(
    matrix: &[Vec<f32>],
    target_index: usize,
    used: &mut [bool],
    current: &mut Vec<usize>,
    score: f32,
    best_score: &mut f32,
    best_order: &mut Vec<usize>,
) {
    if target_index == matrix.len() {
        if score > *best_score {
            *best_score = score;
            *best_order = current.clone();
        }
        return;
    }
    for guess_index in 0..matrix.len() {
        if used[guess_index] {
            continue;
        }
        used[guess_index] = true;
        current.push(guess_index);
        search_assignment(
            matrix,
            target_index + 1,
            used,
            current,
            score + matrix[target_index][guess_index],
            best_score,
            best_order,
        );
        current.pop();
        used[guess_index] = false;
    }
}

fn average_score(scores: &[f32]) -> f32 {
    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f32>() / scores.len() as f32
    }
}

pub fn reward_for_score(score: f32) -> (u32, &'static str) {
    if score >= 0.95 {
        (100, "Perfect Read")
    } else if score >= 0.80 {
        (60, "Clear Read")
    } else if score >= 0.60 {
        (25, "Partial Read")
    } else {
        (5, "Faint Echo")
    }
}

fn lexical_word_score(target: &str, guess: &str) -> f32 {
    let target = normalize_word(target);
    let guess = normalize_word(guess);
    if target.is_empty() || guess.is_empty() {
        return 0.0;
    }
    if target == guess {
        return 1.0;
    }
    if aliases_for(&target).iter().any(|alias| *alias == guess) {
        return 0.92;
    }
    if soft_stem(&target) == soft_stem(&guess) {
        return 0.84;
    }
    0.0
}

fn normalize_word(word: &str) -> String {
    word.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn soft_stem(word: &str) -> String {
    for suffix in ["ing", "ed", "es", "s"] {
        if word.len() > suffix.len() + 2 && word.ends_with(suffix) {
            return word[..word.len() - suffix.len()].to_owned();
        }
    }
    word.to_owned()
}

fn aliases_for(target: &str) -> &'static [&'static str] {
    match target {
        "boat" => &["ship", "vessel", "sailboat"],
        "dog" => &["hound", "puppy"],
        "running" => &["run", "runs", "sprinting", "sprint"],
        "moon" => &["lunar"],
        "water" => &["ocean", "sea", "lake", "river"],
        "sunset" => &["dusk", "sunrise", "evening"],
        "tree" => &["forest"],
        "lightning" => &["thunderbolt", "storm"],
        "cat" => &["kitten"],
        "gold" => &["golden", "yellow"],
        "window" => &["glass"],
        "castle" => &["fortress", "tower"],
        "snow" => &["ice", "winter"],
        "night" => &["dark", "moonlight"],
        "clock" => &["watch", "timepiece"],
        "broken" => &["cracked", "shattered", "ruined"],
        "forest" => &["woods"],
        "bird" => &["raven", "crow"],
        "white" => &["pale"],
        "cage" => &["bars", "prison"],
        "robot" => &["android", "machine"],
        "rusty" => &["rust", "corroded"],
        "garden" => &["flowers", "yard"],
        "mask" => &["face"],
        "glass" => &["crystal", "transparent"],
        "rain" => &["storm", "raining"],
        "key" => &["keys"],
        "silver" => &["metal", "metallic", "gray", "grey"],
        "ocean" => &["sea", "water"],
        "lantern" => &["lamp", "light"],
        "orange" => &["amber"],
        "fog" => &["mist", "haze"],
        "dragon" => &["wyvern"],
        "storm" => &["thunder", "lightning"],
        "skull" => &["bones", "bone"],
        "flower" => &["bloom", "blossom"],
        "stars" => &["star", "starlight", "sky"],
        "tower" => &["castle", "spire"],
        "black" => &["dark"],
        "fire" => &["flame", "burning"],
        "eye" => &["eyes"],
        "crystal" => &["glass", "gem"],
        "desert" => &["sand", "dunes"],
        _ => &[],
    }
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
            .any(|prompt| prompt.words == ["Boat", "Blue", "Sunset"]));
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

    #[test]
    fn guess_scoring_accepts_aliases_and_any_order_without_ollama() {
        let target = vec!["Boat".to_owned(), "Blue".to_owned(), "Sunset".to_owned()];
        let guess = vec!["dusk".to_owned(), "ship".to_owned(), "blue".to_owned()];

        let result = score_guess(&target, &guess).unwrap();

        assert_eq!(result.matched_guess_words, vec!["ship", "blue", "dusk"]);
        assert!(result.total_score >= 0.94);
        assert_eq!(result.reward_label, "Clear Read");
    }

    #[test]
    fn reward_tiers_give_clear_player_feedback() {
        assert_eq!(reward_for_score(0.97), (100, "Perfect Read"));
        assert_eq!(reward_for_score(0.82), (60, "Clear Read"));
        assert_eq!(reward_for_score(0.64), (25, "Partial Read"));
        assert_eq!(reward_for_score(0.20), (5, "Faint Echo"));
    }

    #[test]
    fn prompt_pool_avoids_unreadable_abstract_targets() {
        let banned = [
            "Signal", "Witness", "Entropy", "Silence", "Hunger", "Memory",
        ];
        for prompt in ORACLE_PROMPTS {
            for word in prompt.words {
                assert!(
                    !banned.contains(&word),
                    "prompt word {word} is too abstract for a fair image-guessing game"
                );
            }
        }
    }
}
