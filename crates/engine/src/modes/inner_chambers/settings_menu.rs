//! The in-game settings menu.
//!
//! `GAMEPAD_AND_SETTINGS_GUIDE.md` specified this in full and it was never built.
//! `services/settings.rs` has persisted sensitivity, deadzone and the four volumes to
//! `%LOCALAPPDATA%\NeuroCognica\Archetypes\config\settings.json` for a while — atomically,
//! clamped on load and on save — but nothing in the game displayed any of it, so the only way
//! to change a setting was to hand-edit JSON. The only reference to `GameSettings` outside its
//! own file was one read in `camera.rs`.
//!
//! **Esc opens this menu.** The HUD has been telling the player "Esc/B: Menu" while Esc
//! actually ejected them from the castle; making the label true is most of the point. Leaving
//! is now an explicit row in the menu rather than a keystroke that could be hit by accident.
//!
//! Input goes through the existing `InnerActions` / `InnerModalState` arbiter rather than
//! reading keys directly. Three systems racing to own one hint line, and `Esc` closing a modal
//! *and* exiting the mode in the same frame, were both real shipped bugs that arbiter exists to
//! prevent; a menu that read its own keys would re-open the second one immediately.

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::services::gamepad_input;
use crate::services::settings::GameSettings;
use crate::services::sfx::{PlaySfx, Sfx};

use super::interaction::{InnerActions, InnerInteractionSet, InnerModalState};
use super::InnerChambersState;

/// One adjustable or actionable line in the menu.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SettingsRow {
    MouseSensitivity,
    GamepadLookSensitivity,
    GamepadDeadzone,
    VolumeMaster,
    VolumeMusic,
    VolumeSfx,
    VolumeVoice,
    ResetDefaults,
    Resume,
    LeaveCastle,
}

pub const SETTINGS_ROWS: [SettingsRow; 10] = [
    SettingsRow::MouseSensitivity,
    SettingsRow::GamepadLookSensitivity,
    SettingsRow::GamepadDeadzone,
    SettingsRow::VolumeMaster,
    SettingsRow::VolumeMusic,
    SettingsRow::VolumeSfx,
    SettingsRow::VolumeVoice,
    SettingsRow::ResetDefaults,
    SettingsRow::Resume,
    SettingsRow::LeaveCastle,
];

impl SettingsRow {
    pub fn label(self) -> &'static str {
        match self {
            SettingsRow::MouseSensitivity => "Mouse look sensitivity",
            SettingsRow::GamepadLookSensitivity => "Gamepad look sensitivity",
            SettingsRow::GamepadDeadzone => "Gamepad stick deadzone",
            SettingsRow::VolumeMaster => "Master volume",
            SettingsRow::VolumeMusic => "Ambient music volume",
            SettingsRow::VolumeSfx => "Sound effects volume",
            SettingsRow::VolumeVoice => "Council voices volume",
            SettingsRow::ResetDefaults => "Reset everything to defaults",
            SettingsRow::Resume => "Resume",
            SettingsRow::LeaveCastle => "Leave the Inner Castle",
        }
    }

    /// Rows that hold a value, versus rows that do something when confirmed.
    pub fn is_slider(self) -> bool {
        !matches!(
            self,
            SettingsRow::ResetDefaults | SettingsRow::Resume | SettingsRow::LeaveCastle
        )
    }

    /// The range each slider moves through, matching `GameSettings::clamped` exactly. If these
    /// disagreed, a slider could be pushed to a value the persistence layer silently clamps
    /// back on the next load, and the setting would appear to forget itself.
    pub fn range(self) -> Option<(f32, f32)> {
        Some(match self {
            SettingsRow::MouseSensitivity => (0.0002, 0.006),
            SettingsRow::GamepadLookSensitivity => (0.2, 8.0),
            SettingsRow::GamepadDeadzone => (0.0, 0.6),
            SettingsRow::VolumeMaster
            | SettingsRow::VolumeMusic
            | SettingsRow::VolumeSfx
            | SettingsRow::VolumeVoice => (0.0, 1.0),
            _ => return None,
        })
    }

    pub fn read(self, settings: &GameSettings) -> Option<f32> {
        Some(match self {
            SettingsRow::MouseSensitivity => settings.mouse_sensitivity,
            SettingsRow::GamepadLookSensitivity => settings.gamepad_look_sensitivity,
            SettingsRow::GamepadDeadzone => settings.gamepad_deadzone,
            SettingsRow::VolumeMaster => settings.volume_master,
            SettingsRow::VolumeMusic => settings.volume_music,
            SettingsRow::VolumeSfx => settings.volume_sfx,
            SettingsRow::VolumeVoice => settings.volume_voice,
            _ => return None,
        })
    }

    pub fn write(self, settings: &mut GameSettings, value: f32) {
        let Some((low, high)) = self.range() else {
            return;
        };
        let value = value.clamp(low, high);
        match self {
            SettingsRow::MouseSensitivity => settings.mouse_sensitivity = value,
            SettingsRow::GamepadLookSensitivity => settings.gamepad_look_sensitivity = value,
            SettingsRow::GamepadDeadzone => settings.gamepad_deadzone = value,
            SettingsRow::VolumeMaster => settings.volume_master = value,
            SettingsRow::VolumeMusic => settings.volume_music = value,
            SettingsRow::VolumeSfx => settings.volume_sfx = value,
            SettingsRow::VolumeVoice => settings.volume_voice = value,
            _ => {}
        }
    }

    /// A slider's position as 0..1, for drawing the bar.
    pub fn fraction(self, settings: &GameSettings) -> Option<f32> {
        let value = self.read(settings)?;
        let (low, high) = self.range()?;
        Some(((value - low) / (high - low)).clamp(0.0, 1.0))
    }

    /// What the player sees. Volumes read as percentages because that is how people think
    /// about volume; the raw mouse sensitivity is a number like 0.0013 that means nothing, so
    /// it is shown on the same 0-100 scale as everything else.
    pub fn display(self, settings: &GameSettings) -> String {
        match self.fraction(settings) {
            Some(fraction) => format!("{:>3}%", (fraction * 100.0).round() as i32),
            None => String::new(),
        }
    }
}

/// One press of left/right moves a slider this much of its range. Ten steps end to end is
/// coarse enough to cross quickly and fine enough to settle on a value.
const STEP: f32 = 0.10;

/// Characters in a slider bar. Short enough that the bar and its percentage fit on one
/// line in the row's fixed width; at 20 they wrapped and the number fell onto its own line.
const BAR_SEGMENTS: usize = 16;

#[derive(Resource, Default)]
pub struct SettingsMenuState {
    pub open: bool,
    pub row: usize,
    /// Set by the `Leave the Inner Castle` row. `extraction.rs` watches this instead of
    /// watching `Esc`, so the key that opens the menu can never also eject the player.
    pub leave_requested: bool,
}

impl SettingsMenuState {
    pub fn selected(&self) -> SettingsRow {
        SETTINGS_ROWS[self.row.min(SETTINGS_ROWS.len() - 1)]
    }
}

#[derive(Component)]
struct SettingsMenuUi;

#[derive(Component)]
struct SettingsMenuRowText(usize);

#[derive(Component)]
struct SettingsMenuRowBar(usize);

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SettingsMenuState>()
            .add_systems(OnEnter(InnerChambersState::Loading), spawn_settings_menu)
            .add_systems(
                Update,
                (toggle_settings_menu, drive_settings_menu, render_settings_menu)
                    .chain()
                    .after(InnerInteractionSet::Resolve)
                    .run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), teardown_settings_menu);
    }
}

fn spawn_settings_menu(mut commands: Commands) {
    commands.insert_resource(SettingsMenuState::default());

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(28.0),
                top: Val::Percent(14.0),
                width: Val::Px(820.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(26.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(Color::srgb(0.85, 0.70, 0.30)),
            BackgroundColor(Color::srgba(0.03, 0.04, 0.07, 0.97)),
            GlobalZIndex(970),
            SettingsMenuUi,
            Visibility::Hidden,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SETTINGS"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.35)),
            ));
            parent.spawn((
                Node {
                    margin: UiRect::bottom(Val::Px(14.0)),
                    ..default()
                },
                Text::new("Changes apply immediately and are saved to settings.json."),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.60, 0.70, 0.78)),
            ));

            for (index, _) in SETTINGS_ROWS.iter().enumerate() {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            margin: UiRect::vertical(Val::Px(3.0)),
                            ..default()
                        },
                        Visibility::Inherited,
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Node {
                                width: Val::Px(400.0),
                                ..default()
                            },
                            Text::new(""),
                            TextFont {
                                font_size: 17.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.85, 0.88, 0.92)),
                            SettingsMenuRowText(index),
                        ));
                        row.spawn((
                            Node {
                                // Fixed and wide enough for the bar plus its
                                // percentage, so the value never wraps onto its
                                // own line.
                                width: Val::Px(250.0),
                                ..default()
                            },
                            Text::new(""),
                            TextFont {
                                font_size: 17.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.45, 0.80, 1.0)),
                            SettingsMenuRowBar(index),
                        ));
                    });
            }

            parent.spawn((
                Node {
                    margin: UiRect::top(Val::Px(16.0)),
                    ..default()
                },
                Text::new(
                    "[W/S or Up/Down or D-Pad] Select    [A/D or Left/Right or D-Pad] Adjust    \
                     [Enter or A] Confirm    [Esc or B] Close",
                ),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.60, 0.65, 0.70)),
            ));
        });
}

/// Opens and closes the menu. The only place `cancel` is consumed for this purpose, so the
/// key cannot both open the menu and be read by something else in the same frame.
fn toggle_settings_menu(
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    gamepads: Query<&Gamepad>,
    mut menu: ResMut<SettingsMenuState>,
    mut cursor_options: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let start_pressed = gamepad_input::any_just_pressed(&gamepads, GamepadButton::Start);
    let wants_toggle = actions.cancel || start_pressed;
    if !wants_toggle {
        return;
    }
    // Another surface owns input: a conversation, a prompt, the plan workshop. Its own cancel
    // handling runs; this menu stays out of the way.
    if !menu.open && (modal.encounter || modal.manifestation || modal.library || modal.workshop) {
        return;
    }

    menu.open = !menu.open;

    // Hand the cursor back while the menu is up, and take it again on close. Without this the
    // player is reading a menu they cannot point at, with the mouse still driving the camera.
    if let Ok(mut cursor) = cursor_options.single_mut() {
        if menu.open {
            cursor.visible = true;
            cursor.grab_mode = CursorGrabMode::None;
        } else {
            cursor.visible = false;
            cursor.grab_mode = CursorGrabMode::Locked;
        }
    }
}

/// What confirming a row actually does.
///
/// Extracted from `drive_settings_menu` so it can be reached by a test. Inline in the system it
/// could not be, and the test that claimed to cover Reset built a modified `GameSettings`,
/// overwrote it with `GameSettings::default()`, and asserted that equalled the default -- a
/// tautology that could never fail, reading as coverage of a path it never touched. The compiler
/// had been pointing at it the whole time as an unused assignment.
fn confirm_row(row: SettingsRow, settings: &mut GameSettings, menu: &mut SettingsMenuState) {
    match row {
        SettingsRow::ResetDefaults => *settings = GameSettings::default(),
        SettingsRow::Resume => menu.open = false,
        SettingsRow::LeaveCastle => {
            menu.leave_requested = true;
            menu.open = false;
        }
        // Confirming a slider does nothing rather than jumping it somewhere: the player is
        // already adjusting it with left/right and a surprise jump loses their setting.
        _ => {}
    }
}

fn drive_settings_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut menu: ResMut<SettingsMenuState>,
    mut settings: ResMut<GameSettings>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if !menu.open {
        return;
    }

    let up = keyboard.just_pressed(KeyCode::ArrowUp)
        || keyboard.just_pressed(KeyCode::KeyW)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::DPadUp);
    let down = keyboard.just_pressed(KeyCode::ArrowDown)
        || keyboard.just_pressed(KeyCode::KeyS)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::DPadDown);
    let left = keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::DPadLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::DPadRight);
    let confirm = keyboard.just_pressed(KeyCode::Enter)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::South);

    if up {
        menu.row = (menu.row + SETTINGS_ROWS.len() - 1) % SETTINGS_ROWS.len();
        sfx.write(PlaySfx::new(Sfx::MenuMove));
    }
    if down {
        menu.row = (menu.row + 1) % SETTINGS_ROWS.len();
        sfx.write(PlaySfx::new(Sfx::MenuMove));
    }

    let row = menu.selected();

    if row.is_slider() {
        if let (Some(current), Some((low, high))) = (row.read(&settings), row.range()) {
            let step = (high - low) * STEP;
            if left {
                row.write(&mut settings, current - step);
            }
            if right {
                row.write(&mut settings, current + step);
            }
            if left || right {
                // Played at the *new* level, so moving the SFX slider is audible feedback on
                // the thing being moved. That is the one slider that can demonstrate itself.
                sfx.write(PlaySfx::new(Sfx::MenuAdjust));
            }
        }
    }

    if confirm {
        sfx.write(PlaySfx::new(Sfx::MenuConfirm));
        confirm_row(row, &mut settings, &mut menu);
    }
}

fn render_settings_menu(
    menu: Res<SettingsMenuState>,
    settings: Res<GameSettings>,
    mut panel: Query<&mut Visibility, With<SettingsMenuUi>>,
    mut labels: Query<(&SettingsMenuRowText, &mut Text, &mut TextColor), Without<SettingsMenuRowBar>>,
    mut bars: Query<(&SettingsMenuRowBar, &mut Text), Without<SettingsMenuRowText>>,
) {
    for mut visibility in &mut panel {
        *visibility = if menu.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if !menu.open {
        return;
    }

    for (index, mut text, mut colour) in &mut labels {
        let row = SETTINGS_ROWS[index.0];
        let selected = index.0 == menu.row;
        let marker = if selected { "> " } else { "  " };
        text.0 = format!("{marker}{}", row.label());
        colour.0 = if selected {
            Color::srgb(1.0, 0.88, 0.42)
        } else if row.is_slider() {
            Color::srgb(0.85, 0.88, 0.92)
        } else {
            Color::srgb(0.70, 0.82, 0.90)
        };
    }

    for (index, mut text) in &mut bars {
        let row = SETTINGS_ROWS[index.0];
        text.0 = match row.fraction(&settings) {
            Some(fraction) => {
                // A drawn bar rather than a bare number: the number alone gives no sense of
                // where in the range the value sits.
                let filled = (fraction * BAR_SEGMENTS as f32).round() as usize;
                format!(
                    "[{}{}] {}",
                    "#".repeat(filled),
                    "-".repeat(BAR_SEGMENTS - filled),
                    row.display(&settings)
                )
            }
            None => String::new(),
        };
    }
}

fn teardown_settings_menu(mut commands: Commands, ui: Query<Entity, With<SettingsMenuUi>>) {
    for entity in &ui {
        commands.entity(entity).despawn();
    }
    commands.insert_resource(SettingsMenuState::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every slider's range must match `GameSettings::clamped` exactly. If the menu allowed a
    /// value the persistence layer clamps, the setting would appear to forget itself on the
    /// next launch, which is worse than not having the slider.
    #[test]
    fn slider_ranges_match_what_the_settings_file_will_actually_keep() {
        for row in SETTINGS_ROWS {
            let Some((low, high)) = row.range() else {
                continue;
            };
            let mut settings = GameSettings::default();
            row.write(&mut settings, high);
            let kept = settings.clone().clamped();
            assert_eq!(
                row.read(&kept),
                Some(high),
                "{} loses its maximum on save",
                row.label()
            );

            let mut settings = GameSettings::default();
            row.write(&mut settings, low);
            let kept = settings.clone().clamped();
            assert_eq!(
                row.read(&kept),
                Some(low),
                "{} loses its minimum on save",
                row.label()
            );
        }
    }

    #[test]
    fn ten_steps_cross_a_slider_end_to_end() {
        let row = SettingsRow::VolumeMusic;
        let (low, high) = row.range().unwrap();
        let mut settings = GameSettings::default();
        row.write(&mut settings, low);
        for _ in 0..10 {
            let current = row.read(&settings).unwrap();
            row.write(&mut settings, current + (high - low) * STEP);
        }
        assert_eq!(row.read(&settings), Some(high));
    }

    #[test]
    fn action_rows_hold_no_value_and_slider_rows_all_do() {
        for row in SETTINGS_ROWS {
            let settings = GameSettings::default();
            assert_eq!(
                row.is_slider(),
                row.read(&settings).is_some(),
                "{} disagrees about whether it holds a value",
                row.label()
            );
            assert_eq!(row.is_slider(), row.range().is_some(), "{}", row.label());
        }
    }

    #[test]
    fn selection_wraps_in_both_directions() {
        let mut menu = SettingsMenuState::default();
        assert_eq!(menu.selected(), SettingsRow::MouseSensitivity);
        menu.row = SETTINGS_ROWS.len() - 1;
        assert_eq!(menu.selected(), SettingsRow::LeaveCastle);
        menu.row = (menu.row + 1) % SETTINGS_ROWS.len();
        assert_eq!(menu.selected(), SettingsRow::MouseSensitivity);
        menu.row = (menu.row + SETTINGS_ROWS.len() - 1) % SETTINGS_ROWS.len();
        assert_eq!(menu.selected(), SettingsRow::LeaveCastle);
    }

    /// Leaving must be a deliberate choice inside the menu, not the key that opens it. The HUD
    /// has been advertising "Esc/B: Menu" the whole time Esc actually ejected the player.
    #[test]
    fn leaving_the_castle_is_a_row_in_the_menu() {
        assert!(SETTINGS_ROWS.contains(&SettingsRow::LeaveCastle));
        assert!(!SettingsRow::LeaveCastle.is_slider());
    }

    /// The bundled font has no glyphs for block characters or arrows; they draw as empty
    /// boxes, which is visible in every HUD screenshot this project has taken. Every string
    /// this menu renders must stay in ASCII.
    #[test]
    fn every_rendered_string_is_ascii() {
        let settings = GameSettings::default();
        for row in SETTINGS_ROWS {
            assert!(row.label().is_ascii(), "{}", row.label());
            assert!(row.display(&settings).is_ascii(), "{}", row.label());
            if let Some(fraction) = row.fraction(&settings) {
                let filled = (fraction * BAR_SEGMENTS as f32).round() as usize;
                let bar = format!(
                    "[{}{}] {}",
                    "#".repeat(filled),
                    "-".repeat(BAR_SEGMENTS - filled),
                    row.display(&settings)
                );
                assert!(bar.is_ascii(), "{bar}");
            }
        }
    }

    #[test]
    fn a_full_bar_and_its_percentage_stay_on_one_line() {
        // 16 segments plus brackets, a space and "100%" is 23 characters; at font size 17
        // that sits inside the 250px the bar node reserves, so the value never wraps.
        let rendered = format!("[{}] 100%", "#".repeat(BAR_SEGMENTS));
        assert!(rendered.len() <= 24, "bar renders {} chars", rendered.len());
    }

    /// All four volume sliders now reach a real audio path: music, Council voices, and - since
    /// `services/sfx.rs` - sound effects. No row needs a qualifier, and none may carry one
    /// again without a bus behind it.
    #[test]
    fn no_volume_slider_has_to_apologise_for_itself() {
        for row in SETTINGS_ROWS {
            assert!(
                !row.label().contains("yet"),
                "{} still says it does not work",
                row.label()
            );
        }
    }

    /// Drives the real confirm path, so a Reset that stopped resetting would fail this.
    #[test]
    fn resetting_restores_every_default() {
        let mut settings = GameSettings {
            volume_music: 0.1,
            mouse_sensitivity: 0.005,
            ..GameSettings::default()
        };
        assert_ne!(settings, GameSettings::default(), "the fixture must start off-default");

        let mut menu = SettingsMenuState::default();
        confirm_row(SettingsRow::ResetDefaults, &mut settings, &mut menu);

        assert_eq!(settings, GameSettings::default());
        assert!(menu.open == SettingsMenuState::default().open, "Reset must not close the menu");
    }

    /// Leaving is the one row with a consequence outside this menu, and `extraction.rs` consumes
    /// the flag. A Leave that stopped requesting it would strand the player in the castle.
    #[test]
    fn leaving_requests_extraction_and_closes_the_menu() {
        let mut settings = GameSettings::default();
        let mut menu = SettingsMenuState { open: true, ..Default::default() };
        confirm_row(SettingsRow::LeaveCastle, &mut settings, &mut menu);
        assert!(menu.leave_requested);
        assert!(!menu.open);
        assert_eq!(settings, GameSettings::default(), "leaving must not disturb settings");
    }

    /// Confirming a slider must do nothing: the player is adjusting it with left/right, and a
    /// jump on Enter would lose the value they just set.
    #[test]
    fn confirming_a_slider_leaves_it_exactly_where_the_player_put_it() {
        let mut settings = GameSettings { volume_music: 0.37, ..GameSettings::default() };
        let mut menu = SettingsMenuState { open: true, ..Default::default() };
        let slider = SETTINGS_ROWS
            .iter()
            .copied()
            .find(|row| row.is_slider())
            .expect("the menu has at least one slider");

        confirm_row(slider, &mut settings, &mut menu);
        assert_eq!(settings.volume_music, 0.37);
        assert!(menu.open, "confirming a slider must not close the menu");
    }
}
