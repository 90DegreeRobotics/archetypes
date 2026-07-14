use bevy::input::keyboard::Key;
use bevy::prelude::*;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use super::{OracleSession, OracleState};
use crate::chamber::boot::spawn_main_menu;
use crate::modes::game_mode::GameMode;
use crate::modes::ModeRegistry;
use crate::services::ledger::append_to_ledger;

pub struct OracleUiPlugin;

impl Plugin for OracleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(OracleState::Generating), setup_generating)
            .add_systems(OnExit(OracleState::Generating), despawn_oracle_ui)
            .add_systems(OnEnter(OracleState::Guessing), setup_guessing)
            .add_systems(
                Update,
                handle_guessing.run_if(in_state(OracleState::Guessing)),
            )
            .add_systems(Update, render_draft.run_if(in_state(OracleState::Guessing)))
            .add_systems(OnExit(OracleState::Guessing), despawn_oracle_ui)
            .add_systems(OnEnter(OracleState::Scoring), setup_scoring)
            .add_systems(OnExit(OracleState::Scoring), despawn_oracle_ui)
            .add_systems(OnEnter(OracleState::Result), setup_result)
            .add_systems(Update, handle_result.run_if(in_state(OracleState::Result)))
            .add_systems(OnExit(OracleState::Result), despawn_oracle_ui);
    }
}

#[derive(Component)]
struct OracleUiNode;

#[derive(Component)]
struct OracleDraftText;

#[derive(Component)]
struct OracleFeedbackText;

fn despawn_oracle_ui(mut commands: Commands, query: Query<Entity, With<OracleUiNode>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn setup_generating(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.9)),
            GlobalZIndex(900),
            OracleUiNode,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("The Oracle is Meditating...\nGenerating vision..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn oracle_asset_dir() -> PathBuf {
    let assets = if cfg!(debug_assertions) {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets")
    } else {
        PathBuf::from("assets")
    };
    assets.join("oracle_riddle")
}

fn setup_guessing(
    mut commands: Commands,
    session: Res<OracleSession>,
    asset_server: Res<AssetServer>,
) {
    let mut image_node = None;
    if let Some(path) = &session.image_path {
        let dir = oracle_asset_dir();
        let _ = fs::create_dir_all(&dir);
        let file_name = "oracle_current.png";
        if fs::copy(path, dir.join(file_name)).is_ok() {
            image_node = Some(asset_server.load(format!("oracle_riddle/{}", file_name)));
        }
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.9)),
            GlobalZIndex(900),
            OracleUiNode,
        ))
        .with_children(|p| {
            if let Some(img) = image_node {
                p.spawn((
                    ImageNode::new(img),
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(400.0),
                        ..default()
                    },
                ));
            } else {
                p.spawn((
                    Text::new("Failed to load image"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ));
            }
            p.spawn((
                Text::new(format!(
                    "What 3 words prompted this vision?\nDifficulty: {:?}",
                    session.difficulty
                )),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.9)),
            ));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                OracleDraftText,
            ));
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.62, 0.36)),
                OracleFeedbackText,
            ));
        });
}

fn handle_guessing(
    keyboard: Res<ButtonInput<Key>>,
    mut session: ResMut<OracleSession>,
    mut next_state: ResMut<NextState<OracleState>>,
) {
    for key in keyboard.get_just_pressed() {
        match key {
            Key::Backspace => {
                let _ = session.draft.pop();
                session.feedback_msg = None;
            }
            Key::Space => {
                session.draft.push(' ');
                session.feedback_msg = None;
            }
            Key::Enter => {
                let words: Vec<String> = session
                    .draft
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect();
                if words.len() == 3 {
                    session.guess_words = words;
                    session.feedback_msg = None;
                    next_state.set(OracleState::Scoring);
                } else {
                    session.feedback_msg = Some(format!(
                        "Enter exactly 3 words. Current count: {}.",
                        words.len()
                    ));
                }
            }
            Key::Character(text) => {
                let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                session.draft.push_str(&filtered);
                session.feedback_msg = None;
            }
            _ => {}
        }
    }
}

fn render_draft(
    session: Res<OracleSession>,
    mut draft: Query<&mut Text, (With<OracleDraftText>, Without<OracleFeedbackText>)>,
    mut feedback: Query<&mut Text, (With<OracleFeedbackText>, Without<OracleDraftText>)>,
) {
    if let Ok(mut text) = draft.single_mut() {
        text.0 = format!("{}_", session.draft);
    }
    if let Ok(mut text) = feedback.single_mut() {
        text.0 = session.feedback_msg.clone().unwrap_or_default();
    }
}

fn setup_scoring(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.9)),
            GlobalZIndex(900),
            OracleUiNode,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("Scoring Embeddings..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn setup_result(mut commands: Commands, session: Res<OracleSession>) {
    let kind = oracle_round_kind(&session);
    let payload = oracle_round_payload(&session);
    if let Err(error) = append_to_ledger(GameMode::OracleRiddle, kind, payload) {
        warn!("oracle ledger seal failed: {error}");
    }

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.03, 0.9)),
            GlobalZIndex(900),
            OracleUiNode,
        ))
        .with_children(|p| {
            if let Some(err) = &session.error_msg {
                p.spawn((
                    Text::new(format!("Oracle Failed:\n{}", err)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ));
            } else {
                p.spawn((
                    Text::new("Oracle Riddle Result"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                let mut results_text = String::new();
                for i in 0..3 {
                    results_text.push_str(&format!(
                        "Target: '{}' vs Guess: '{}' - Score: {:.2}\n",
                        word_at(&session.target_words, i),
                        word_at(&session.guess_words, i),
                        session.scores.get(i).unwrap_or(&0.0)
                    ));
                }
                p.spawn((
                    Text::new(results_text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.8, 1.0)),
                ));
            }
            p.spawn((
                Text::new("ENTER starts another riddle. Type M to return to menu."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn handle_result(
    mut commands: Commands,
    keyboard: Res<ButtonInput<Key>>,
    mut next_state: ResMut<NextState<OracleState>>,
    registry: Res<ModeRegistry>,
) {
    for key in keyboard.get_just_pressed() {
        if matches!(key, Key::Enter) {
            next_state.set(OracleState::Generating);
            return;
        }
        if matches!(key, Key::Character(text) if text.eq_ignore_ascii_case("m")) {
            commands.remove_resource::<crate::chamber::ActiveGameMode>();
            next_state.set(OracleState::Inactive);
            spawn_main_menu(commands, registry);
            return;
        }
    }
}

pub fn oracle_round_kind(session: &OracleSession) -> &'static str {
    if session.error_msg.is_none() {
        "oracle_round_completed"
    } else {
        "oracle_round_failed"
    }
}

pub fn oracle_round_payload(session: &OracleSession) -> serde_json::Value {
    json!({
        "difficulty": format!("{:?}", session.difficulty),
        "target": session.target_words.clone(),
        "guess": session.guess_words.clone(),
        "scores": session.scores.clone(),
        "error": session.error_msg.clone(),
    })
}

fn word_at(words: &[String], index: usize) -> &str {
    words.get(index).map(String::as_str).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::difficulty::Difficulty;

    #[test]
    fn ledger_payload_shape_records_difficulty_words_scores_and_error() {
        let session = OracleSession {
            difficulty: Difficulty::Abyssal,
            target_words: vec!["Entropy".into(), "Memory".into(), "Bloom".into()],
            guess_words: vec!["Chaos".into(), "Recall".into(), "Flower".into()],
            scores: vec![0.72, 0.66, 0.81],
            ..default()
        };

        let payload = oracle_round_payload(&session);
        assert_eq!(oracle_round_kind(&session), "oracle_round_completed");
        assert_eq!(payload["difficulty"], "Abyssal");
        assert_eq!(payload["target"].as_array().unwrap().len(), 3);
        assert_eq!(payload["guess"].as_array().unwrap().len(), 3);
        assert_eq!(payload["scores"].as_array().unwrap().len(), 3);
        assert!(payload["error"].is_null());
    }

    #[test]
    fn failed_round_payload_uses_failure_kind() {
        let session = OracleSession {
            difficulty: Difficulty::Literal,
            target_words: vec!["Dog".into(), "Red".into(), "Running".into()],
            error_msg: Some("Chronos unavailable".into()),
            ..default()
        };

        let payload = oracle_round_payload(&session);
        assert_eq!(oracle_round_kind(&session), "oracle_round_failed");
        assert_eq!(payload["error"], "Chronos unavailable");
    }
}
