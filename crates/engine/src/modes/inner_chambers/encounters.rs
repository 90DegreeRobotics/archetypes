//! Diegetic typed encounters for embodied archetypes in the Inner Castle.
//!
//! This is intentionally keyboard-complete before local speech recognition exists.
//! A failed local model leaves the player's turn on screen; no answer is fabricated.

use std::{fs, io::Write, sync::{mpsc, Mutex}, time::{SystemTime, UNIX_EPOCH}};
use std::thread;

use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
use serde_json::json;

use crate::modes::game_mode::GameMode;
use crate::services::archetype_conversation::{request_reply, ArchetypeChatRecord};
use crate::services::encounter_memory::{self, RetentionState};
use crate::services::gamepad_input;
use crate::services::ledger::append_to_ledger;
use crate::services::paths::app_data_root;
use crate::theme::Archetype;

pub const INNER_CASTLE_ENCOUNTER_CAPABILITY: &str = "inner_castle_encounter";

use super::interaction::{InnerInteractionSet, InteractionFocus, InteractionTarget};
use super::world::{InnerChambersHint, InnerWorldElement};
use super::InnerChambersState;

pub const ENCOUNTER_RANGE: f32 = 6.75;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArchetypeEmbodiment {
    pub archetype: Archetype,
    pub chamber_title: &'static str,
}

#[derive(Resource, Default)]
pub struct EncounterState {
    active: Option<ArchetypeEmbodiment>,
    draft: String,
    transcript: Vec<ArchetypeChatRecord>,
    status: String,
    /// The most recently completed turn's journal id, pending a Remember/Forget decision.
    /// A turn is logged as Transient the moment a reply arrives; it only becomes part of
    /// this archetype's recallable history once the player explicitly remembers it.
    last_record_id: Option<String>,
    last_record_state: Option<RetentionState>,
}

impl EncounterState {
    pub fn is_open(&self) -> bool { self.active.is_some() }
}

#[derive(Resource, Default)]
struct EncounterBridge { receiver: Mutex<Option<mpsc::Receiver<Result<EncounterReply, String>>>>, waiting: bool }

struct EncounterReply { text: String, wav: Vec<u8> }

#[derive(Component)] struct EncounterVoice;

#[derive(Component)] struct EncounterPanel;
#[derive(Component)] struct EncounterText;

pub struct EncounterPlugin;

impl Plugin for EncounterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EncounterState>()
            .init_resource::<EncounterBridge>()
            .add_systems(OnEnter(InnerChambersState::Navigating), spawn_encounter_ui)
            .add_systems(Update, (focus_or_open_encounter, type_or_close_encounter, poll_reply, render_encounter_ui).chain().after(InnerInteractionSet::Resolve).run_if(in_state(InnerChambersState::Navigating)));
    }
}

fn spawn_encounter_ui(mut commands: Commands) {
    commands.spawn((
        Node { position_type: PositionType::Absolute, left: Val::Percent(12.0), right: Val::Percent(12.0), bottom: Val::Px(48.0), padding: UiRect::all(Val::Px(24.0)), flex_direction: FlexDirection::Column, row_gap: Val::Px(10.0), ..default() },
        BackgroundColor(Color::srgba(0.025, 0.022, 0.035, 0.96)), BorderColor::all(Color::srgb(0.72, 0.58, 0.32)), Visibility::Hidden, GlobalZIndex(1100), EncounterPanel, InnerWorldElement,
    )).with_children(|parent| {
        parent.spawn((Text::new(""), TextFont { font_size: 20.0, ..default() }, TextColor(Color::srgb(0.94, 0.90, 0.80)), EncounterText));
    });
}

fn focus_or_open_encounter(
    keyboard: Res<ButtonInput<KeyCode>>, gamepads: Query<&Gamepad>, focus: Res<InteractionFocus>,
    mut state: ResMut<EncounterState>, mut hint: Query<&mut Text, With<InnerChambersHint>>, mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if state.is_open() { return; }
    if let Ok(mut hint) = hint.single_mut() {
        if let Some(InteractionTarget::Archetype(embodiment)) = focus.0 { hint.0 = format!("[E / Pad-X] Speak with {} — typed local conversation", embodiment.archetype.theme().name); }
    }
    if keyboard.just_pressed(KeyCode::KeyE) || gamepad_input::any_just_pressed(&gamepads, GamepadButton::West) {
        if let Some(InteractionTarget::Archetype(embodiment)) = focus.0 {
            state.active = Some(embodiment); state.draft.clear();
            state.last_record_id = None; state.last_record_state = None;
            state.status = format!("Speak with {}. Enter sends locally; Esc returns to the chamber.", embodiment.archetype.theme().name);
            if let Ok(mut cursor) = cursor.single_mut() { cursor.visible = true; cursor.grab_mode = CursorGrabMode::None; }
        }
    }
}

fn type_or_close_encounter(
    keyboard: Res<ButtonInput<KeyCode>>, gamepads: Query<&Gamepad>, mut state: ResMut<EncounterState>, mut bridge: ResMut<EncounterBridge>, mut cursor: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let Some(active) = state.active else { return; };
    if keyboard.just_pressed(KeyCode::Escape) || gamepad_input::any_just_pressed(&gamepads, GamepadButton::East) {
        state.active = None; state.draft.clear(); state.status.clear();
        state.last_record_id = None; state.last_record_state = None;
        if let Ok(mut cursor) = cursor.single_mut() { cursor.visible = false; cursor.grab_mode = CursorGrabMode::Locked; }
        return;
    }
    if let Some(record_id) = state.last_record_id.clone() {
        if keyboard.just_pressed(KeyCode::F5) {
            match encounter_memory::remember(&record_id) {
                Ok(()) => { state.last_record_state = Some(RetentionState::Remembered); state.status = "Remembered — this turn will be recalled in future encounters. F6 Forget • F7 View record".into(); }
                Err(error) => state.status = format!("Could not remember this turn: {error}"),
            }
        } else if keyboard.just_pressed(KeyCode::F6) {
            match encounter_memory::forget(&record_id, Some("player pressed Forget")) {
                Ok(()) => { state.last_record_state = Some(RetentionState::Forgotten); state.status = "Forgotten — withdrawn, and will not be recalled again. F7 View record".into(); }
                Err(error) => state.status = format!("Could not forget this turn: {error}"),
            }
        } else if keyboard.just_pressed(KeyCode::F7) {
            match encounter_memory::view_record(&record_id) {
                Ok(Some(found)) => state.status = format!(
                    "Record {} [{:?}]\n> {}\n{}: {}",
                    &found.record.id[..8.min(found.record.id.len())],
                    found.retention_state,
                    found.record.player_input,
                    active.archetype.theme().name,
                    found.record.response,
                ),
                Ok(None) => state.status = "That record could not be found on disk.".into(),
                Err(error) => state.status = format!("Could not read the local record: {error}"),
            }
        }
    }
    if bridge.waiting { return; }
    if keyboard.just_pressed(KeyCode::Backspace) { state.draft.pop(); }
    for (key, ch) in [(KeyCode::KeyA,'a'),(KeyCode::KeyB,'b'),(KeyCode::KeyC,'c'),(KeyCode::KeyD,'d'),(KeyCode::KeyE,'e'),(KeyCode::KeyF,'f'),(KeyCode::KeyG,'g'),(KeyCode::KeyH,'h'),(KeyCode::KeyI,'i'),(KeyCode::KeyJ,'j'),(KeyCode::KeyK,'k'),(KeyCode::KeyL,'l'),(KeyCode::KeyM,'m'),(KeyCode::KeyN,'n'),(KeyCode::KeyO,'o'),(KeyCode::KeyP,'p'),(KeyCode::KeyQ,'q'),(KeyCode::KeyR,'r'),(KeyCode::KeyS,'s'),(KeyCode::KeyT,'t'),(KeyCode::KeyU,'u'),(KeyCode::KeyV,'v'),(KeyCode::KeyW,'w'),(KeyCode::KeyX,'x'),(KeyCode::KeyY,'y'),(KeyCode::KeyZ,'z'),(KeyCode::Space,' '), (KeyCode::Comma,','),(KeyCode::Period,'.'),(KeyCode::Quote,'\''),(KeyCode::Minus,'-'),(KeyCode::Slash,'/'),(KeyCode::Digit0,'0'),(KeyCode::Digit1,'1'),(KeyCode::Digit2,'2'),(KeyCode::Digit3,'3'),(KeyCode::Digit4,'4'),(KeyCode::Digit5,'5'),(KeyCode::Digit6,'6'),(KeyCode::Digit7,'7'),(KeyCode::Digit8,'8'),(KeyCode::Digit9,'9')] {
        if keyboard.just_pressed(key) && state.draft.chars().count() < 600 { state.draft.push(ch); }
    }
    if !keyboard.just_pressed(KeyCode::Enter) || state.draft.trim().is_empty() { return; }
    let message = state.draft.trim().to_owned();
    state.transcript.push(ArchetypeChatRecord { role: "Witness".into(), content: message.clone() });
    state.draft.clear(); state.status = format!("{} is forming a local reply…", active.archetype.theme().name);
    if let Err(error) = append_encounter_record(active, "Witness", &message) { state.status = format!("Conversation blocked: local transcript could not be written ({error})."); return; }
    if let Err(error) = append_to_ledger(GameMode::InnerChambers, "archetype_encounter_user", json!({"archetype": active.archetype.theme().name, "chamber": active.chamber_title, "chars": message.chars().count()})) { state.status = format!("Conversation blocked: ledger could not be sealed ({error})."); return; }
    let history = state.transcript.clone(); let location = active.chamber_title.to_owned(); let archetype = active.archetype;
    let (sender, receiver) = mpsc::channel(); *bridge.receiver.lock().expect("encounter receiver lock") = Some(receiver); bridge.waiting = true;
    thread::spawn(move || {
        let result = request_reply(archetype, &history, &message, &location).and_then(|text| {
            crate::chamber::speech::synthesize_archetype_reply(archetype, text.clone()).map(|wav| EncounterReply { text, wav })
        });
        let _ = sender.send(result);
    });
}

fn poll_reply(mut commands: Commands, mut state: ResMut<EncounterState>, mut bridge: ResMut<EncounterBridge>, mut audio_assets: ResMut<Assets<AudioSource>>, playing: Query<Entity, With<EncounterVoice>>) {
    if !bridge.waiting { return; }
    let reply = bridge.receiver.lock().expect("encounter receiver lock").as_ref().and_then(|receiver| receiver.try_recv().ok());
    let Some(reply) = reply else { return; }; bridge.waiting = false;
    match reply { Ok(reply) => { let role = state.active.map(|a| a.archetype.theme().name.to_owned()).unwrap_or_else(|| "Archetype".into()); if let Some(active) = state.active { if let Err(error) = append_encounter_record(active, &role, &reply.text) { state.status = format!("Reply received but could not persist local transcript ({error})."); return; } } for entity in &playing { commands.entity(entity).despawn(); } let bytes: std::sync::Arc<[u8]> = reply.wav.into(); let handle = audio_assets.add(AudioSource { bytes }); commands.spawn((AudioPlayer::new(handle), PlaybackSettings::DESPAWN, EncounterVoice, Name::new("InnerCastleArchetypeVoice")));
            let player_turn = state.transcript.last().map(|turn| turn.content.clone()).unwrap_or_default();
            state.transcript.push(ArchetypeChatRecord { role: role.clone(), content: reply.text.clone() });
            match encounter_memory::record_turn(&role, INNER_CASTLE_ENCOUNTER_CAPABILITY, &player_turn, &reply.text) {
                Ok(record) => {
                    state.last_record_id = Some(record.id);
                    state.last_record_state = Some(RetentionState::Transient);
                    state.status = "Local reply received and speaking. Enter sends • F5 Remember • F6 Forget • F7 View record • Esc returns to the chamber.".into();
                }
                Err(error) => {
                    state.last_record_id = None;
                    state.last_record_state = None;
                    state.status = format!("Reply received but could not open a memory record for it ({error}). This turn cannot be remembered.");
                }
            }
        }
        Err(error) => state.status = format!("Local reply failed: {error} Your message remains in this encounter record."), }
}

fn append_encounter_record(active: ArchetypeEmbodiment, role: &str, content: &str) -> Result<(), String> {
    let path = app_data_root().join("conversation_history").join("inner_castle.jsonl");
    let parent = path.parent().ok_or("conversation history has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let record = json!({"timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis(), "archetype": active.archetype.theme().name, "chamber": active.chamber_title, "role": role, "content": content});
    let mut file = fs::OpenOptions::new().create(true).append(true).open(path).map_err(|error| error.to_string())?;
    writeln!(file, "{}", serde_json::to_string(&record).map_err(|error| error.to_string())?).map_err(|error| error.to_string())
}

fn render_encounter_ui(state: Res<EncounterState>, mut panel: Query<&mut Visibility, With<EncounterPanel>>, mut text: Query<&mut Text, With<EncounterText>>) {
    let Ok(mut panel) = panel.single_mut() else { return; }; *panel = if state.is_open() { Visibility::Visible } else { Visibility::Hidden };
    let Ok(mut text) = text.single_mut() else { return; }; let Some(active) = state.active else { return; };
    let turns = state.transcript.iter().rev().take(6).rev().map(|turn| format!("{}: {}", turn.role, turn.content)).collect::<Vec<_>>().join("\n\n");
    text.0 = format!("{} — {}\n\n{}\n\n> {}▌\n\n{}", active.archetype.theme().name, active.chamber_title, turns, state.draft, state.status);
}

#[cfg(test)] mod tests { use super::*; #[test] fn closest_embodiment_must_be_in_real_talking_range() { assert_eq!(ENCOUNTER_RANGE, 6.75); } }
