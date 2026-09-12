//! The player's reachable catalogue of manifested creations.
//!
//! Artifact rows are durable evidence, but a JSONL file the player cannot reach is not a
//! collection. This modal turns that durable library into a world action: select an earlier
//! creation and summon the exact retained asset into the player's hand. It never regenerates,
//! overwrites, or copies an artifact file.

use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::services::artifacts::{self, ArtifactRecord};
use crate::services::sfx::{PlaySfx, Sfx};

use super::interaction::{set_modal_cursor, InnerInteractionSet, InnerModalState};
use super::objects::Carried;
use super::world::{HintPriority, HintRequest, InnerHintSet, InnerWorldElement};
use super::InnerChambersState;

#[derive(Resource, Default)]
pub struct LibraryState {
    open: bool,
    records: Vec<ArtifactRecord>,
    selected: usize,
    status: String,
}

impl LibraryState {
    pub fn is_open(&self) -> bool {
        self.open
    }

    fn selected(&self) -> Option<&ArtifactRecord> {
        self.records.get(self.selected)
    }

    fn reload(&mut self) {
        self.records = artifacts::load_library();
        // Newest creation first. The artifact id is retained as the stable identity even when
        // two prompts happen to use identical words.
        self.records.reverse();
        self.selected = self.selected.min(self.records.len().saturating_sub(1));
    }
}

#[derive(Component)]
struct LibraryPanel;

#[derive(Component)]
struct LibraryText;

pub struct LibraryPlugin;

impl Plugin for LibraryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LibraryState>()
            .add_systems(OnEnter(InnerChambersState::Navigating), spawn_library_ui)
            .add_systems(OnEnter(InnerChambersState::Exiting), close_library_on_exit)
            .add_systems(
                Update,
                (toggle_library, drive_library, render_library_ui)
                    .chain()
                    .in_set(InnerHintSet::Request)
                    .after(InnerInteractionSet::Resolve)
                    .run_if(in_state(InnerChambersState::Navigating)),
            );
    }
}

fn spawn_library_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(15.0),
                right: Val::Percent(15.0),
                top: Val::Px(92.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.035, 0.065, 0.97)),
            BorderColor::all(Color::srgb(0.38, 0.76, 0.96)),
            Visibility::Hidden,
            GlobalZIndex(1120),
            LibraryPanel,
            InnerWorldElement,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont { font_size: 20.0, ..default() },
                TextColor(Color::srgb(0.88, 0.94, 1.0)),
                LibraryText,
            ));
        });
}

fn close_library_on_exit(mut state: ResMut<LibraryState>) {
    *state = LibraryState::default();
}

fn toggle_library(
    keyboard: Res<ButtonInput<KeyCode>>,
    modal: Res<InnerModalState>,
    mut state: ResMut<LibraryState>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyL) || state.open {
        return;
    }
    if modal.any() {
        return;
    }
    state.open = true;
    state.status.clear();
    state.reload();
    set_modal_cursor(&mut cursor, true);
}

fn drive_library(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<LibraryState>,
    mut carried: ResMut<Carried>,
    mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
    mut hint: ResMut<HintRequest>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if !state.open {
        return;
    }
    hint.request(HintPriority::Modal, "Creation Library — choose something you made");
    if keyboard.just_pressed(KeyCode::Escape) {
        state.open = false;
        set_modal_cursor(&mut cursor, false);
        return;
    }
    if !state.records.is_empty() {
        if keyboard.just_pressed(KeyCode::ArrowDown) {
            state.selected = (state.selected + 1) % state.records.len();
            sfx.write(PlaySfx::new(Sfx::MenuMove));
        }
        if keyboard.just_pressed(KeyCode::ArrowUp) {
            state.selected = (state.selected + state.records.len() - 1) % state.records.len();
            sfx.write(PlaySfx::new(Sfx::MenuMove));
        }
    }
    if keyboard.just_pressed(KeyCode::KeyL) {
        state.reload();
        state.status = "Library refreshed from your local artifact record.".to_owned();
        return;
    }
    if !keyboard.just_pressed(KeyCode::Enter) {
        return;
    }
    if carried.is_carrying() {
        state.status = "Your hands are full. Set the held object down before summoning another.".to_owned();
        return;
    }
    let Some(record) = state.selected().cloned() else {
        state.status = "Nothing has been manifested yet. Create something at the altar first.".to_owned();
        return;
    };
    carried.artifact = Some(record.clone());
    carried.copies = 0;
    state.open = false;
    set_modal_cursor(&mut cursor, false);
    sfx.write(PlaySfx::new(Sfx::PickUp));
    info!("library: summoned retained artifact {}", record.id);
}

fn render_library_ui(
    state: Res<LibraryState>,
    mut panel: Query<&mut Visibility, With<LibraryPanel>>,
    mut text: Query<&mut Text, With<LibraryText>>,
) {
    let Ok(mut panel) = panel.single_mut() else { return; };
    *panel = if state.open { Visibility::Visible } else { Visibility::Hidden };
    if !state.open { return; }
    let Ok(mut text) = text.single_mut() else { return; };
    let body = library_body(&state);
    if text.0 != body { text.0 = body; }
}

fn library_body(state: &LibraryState) -> String {
    let mut body = String::from("CREATION LIBRARY — objects you manifested and kept locally\n\n");
    if state.records.is_empty() {
        body.push_str("No creations are recorded yet. Manifest an object at the altar; it will stay here for later use.");
    } else {
        for (index, record) in state.records.iter().enumerate() {
            let marker = if index == state.selected { ">" } else { " " };
            let prompt = if record.prompt.trim().is_empty() { "(prompt unavailable)" } else { &record.prompt };
            body.push_str(&format!("{marker} {prompt}\n    id: {}\n", record.id));
        }
    }
    body.push_str("\n[Up/Down] Choose    [Enter] Summon to hand    [L] Refresh    [Esc] Return");
    if !state.status.is_empty() { body.push_str(&format!("\n\n{}", state.status)); }
    body
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_record_is_selectable_and_its_identity_is_visible() {
        let state = LibraryState {
            open: true,
            records: vec![ArtifactRecord { id: "kept-1".to_owned(), asset: "manifested/kept-1.glb".to_owned(), prompt: "a brass astrolabe".to_owned(), created: "1".to_owned(), ..Default::default() }],
            selected: 0,
            status: String::new(),
        };
        let body = library_body(&state);
        assert!(body.contains("> a brass astrolabe"));
        assert!(body.contains("id: kept-1"));
        assert_eq!(state.selected().map(|record| record.asset.as_str()), Some("manifested/kept-1.glb"));
    }

    #[test]
    fn an_empty_library_explains_how_to_create_the_first_object() {
        assert!(library_body(&LibraryState::default()).contains("Manifest an object at the altar"));
    }
}
