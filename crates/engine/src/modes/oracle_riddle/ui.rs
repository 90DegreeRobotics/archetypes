use bevy::prelude::*;
use bevy::input::keyboard::Key;
use std::fs;
use std::path::PathBuf;

use super::{OracleSession, OracleState};
use crate::chamber::boot::spawn_main_menu;
use crate::modes::game_mode::GameMode;
use crate::modes::ModeRegistry;
use crate::services::ledger::append_to_ledger;
use serde_json::json;

pub struct OracleUiPlugin;

impl Plugin for OracleUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(OracleState::Generating), setup_generating)
            .add_systems(OnExit(OracleState::Generating), despawn_oracle_ui)
            .add_systems(OnEnter(OracleState::Guessing), setup_guessing)
            .add_systems(Update, handle_guessing.run_if(in_state(OracleState::Guessing)))
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
                Text::new("What 3 words prompted this vision?"),
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
            }
            Key::Space => {
                session.draft.push(' ');
            }
            Key::Enter => {
                let words: Vec<String> = session
                    .draft
                    .split_whitespace()
                    .map(|s| s.to_owned())
                    .collect();
                if words.len() == 3 {
                    session.guess_words = words;
                    next_state.set(OracleState::Scoring);
                } else {
                    // Let them know they need exactly 3 words.
                }
            }
            Key::Character(text) => {
                let filtered: String = text.chars().filter(|c| !c.is_control()).collect();
                session.draft.push_str(&filtered);
            }
            _ => {}
        }
    }
}

fn render_draft(session: Res<OracleSession>, mut query: Query<&mut Text, With<OracleDraftText>>) {
    if let Ok(mut text) = query.single_mut() {
        text.0 = format!("{}_", session.draft);
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
    // Seal to ledger
    let kind = if session.error_msg.is_none() {
        "oracle_round_completed"
    } else {
        "oracle_round_failed"
    };
    let payload = json!({
        "target": session.target_words,
        "guess": session.guess_words,
        "scores": session.scores,
        "error": session.error_msg,
    });
    let _ = append_to_ledger(GameMode::OracleRiddle, kind, payload);

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
                        session.target_words.get(i).unwrap_or(&"".to_owned()),
                        session.guess_words.get(i).unwrap_or(&"".to_owned()),
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
                Text::new("Press ENTER to return to menu"),
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
            commands.remove_resource::<crate::chamber::ActiveGameMode>();
            next_state.set(OracleState::Inactive);
            spawn_main_menu(commands, registry);
            return;
        }
    }
}
