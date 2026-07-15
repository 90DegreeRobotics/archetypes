//! Title / loading screen.
//!
//! The chamber is a heavy GPU scene; on launch nothing of it should be shown until it
//! is ready. A full-screen title overlay covers everything from the first frame and is
//! only lifted once the authored council geometry is ready.
//! The reveal lands on a real table-based main menu; Standard Mode never starts itself.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use super::{
    camera::WitnessCamera, portal::StargatePortal, spheres::ArchetypeSphere, ChamberState,
};
use crate::modes::{
    game_mode::GameMode, standard_mecha::TriggerStandardMecha, ModeRegistry,
};

/// Minimum time the title holds — long enough that the heavy scene textures finish
/// uploading behind it, so the reveal does not hitch.
const MIN_BOOT_SECS: f32 = 8.0;
const BOOT_FADE_SECS: f32 = 3.4;
const TITLE_FADE_START_SECS: f32 = 0.8;
const TITLE_FADE_SECS: f32 = 2.6;
const SUBTITLE_FADE_START_SECS: f32 = 3.2;
const SUBTITLE_FADE_SECS: f32 = 2.8;
/// The seven council vessels; when all are bound the scene is ready.
const READY_SPHERES: usize = 7;

pub struct BootPlugin;

impl Plugin for BootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boot_ui)
            .init_resource::<BootSequence>()
            .add_systems(
                Update,
                (animate_loading_veil, boot_ready)
                    .chain()
                    .run_if(in_state(ChamberState::Booting)),
            )
            .add_systems(OnExit(ChamberState::Booting), despawn_boot_ui)
            .add_systems(OnEnter(ChamberState::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                (position_main_menu_on_portal, style_mode_buttons, activate_mode)
                    .chain()
                    .run_if(in_state(ChamberState::MainMenu)),
            )
            .add_systems(OnExit(ChamberState::MainMenu), despawn_main_menu);
    }
}

#[derive(Component)]
struct BootUi;

#[derive(Component)]
struct BootTitle;

#[derive(Component)]
struct BootSubtitle;

#[derive(Resource, Default)]
struct BootSequence {
    ready_at: Option<f32>,
}

#[derive(Component)]
pub(crate) struct MainMenuUi;

#[derive(Component)]
struct ModeButton {
    mode: GameMode,
    available: bool,
}

#[derive(Component)]
struct MainMenuPortalPanel;

fn spawn_boot_ui(mut commands: Commands, mut sequence: ResMut<BootSequence>) {
    sequence.ready_at = None;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            GlobalZIndex(1000),
            BootUi,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("ARCHETYPES"),
                TextFont {
                    font_size: 74.0,
                    ..default()
                },
                TextColor(Color::srgba(0.95, 0.95, 0.96, 0.0)),
                BootTitle,
            ));
            parent.spawn((
                Text::new("A GAME BY MICHAEL HOLT"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgba(0.78, 0.80, 0.86, 0.0)),
                BootSubtitle,
            ));
        });
}

fn timed_alpha(elapsed: f32, start: f32, duration: f32) -> f32 {
    ((elapsed - start) / duration).clamp(0.0, 1.0)
}

fn animate_loading_veil(
    time: Res<Time>,
    mut title: Query<&mut TextColor, (With<BootTitle>, Without<BootSubtitle>)>,
    mut subtitle: Query<&mut TextColor, (With<BootSubtitle>, Without<BootTitle>)>,
) {
    let elapsed = time.elapsed_secs();
    if let Ok(mut color) = title.single_mut() {
        color.0 = Color::srgba(
            0.95,
            0.95,
            0.96,
            timed_alpha(elapsed, TITLE_FADE_START_SECS, TITLE_FADE_SECS),
        );
    }
    if let Ok(mut color) = subtitle.single_mut() {
        color.0 = Color::srgba(
            0.78,
            0.80,
            0.86,
            timed_alpha(elapsed, SUBTITLE_FADE_START_SECS, SUBTITLE_FADE_SECS),
        );
    }
}

fn boot_ready(
    time: Res<Time>,
    spheres: Query<&ArchetypeSphere>,
    portal: Query<&Name>,
    mut sequence: ResMut<BootSequence>,
    mut overlay: Query<&mut BackgroundColor, With<BootUi>>,
    mut title: Query<&mut TextColor, (With<BootTitle>, Without<BootSubtitle>)>,
    mut subtitle: Query<&mut TextColor, (With<BootSubtitle>, Without<BootTitle>)>,
    mut next_state: ResMut<NextState<ChamberState>>,
) {
    let elapsed = time.elapsed_secs();
    let chamber_ready = spheres.iter().count() >= READY_SPHERES
        && portal.iter().any(|name| name.as_str() == "Stargate_Portal");
    if elapsed < MIN_BOOT_SECS || !chamber_ready {
        return;
    }
    let ready_at = *sequence.ready_at.get_or_insert(elapsed);
    let fade = ((elapsed - ready_at) / BOOT_FADE_SECS).clamp(0.0, 1.0);
    if let Ok(mut background) = overlay.single_mut() {
        background.0 = Color::srgba(0.0, 0.0, 0.0, 1.0 - fade);
    }
    if let Ok(mut color) = title.single_mut() {
        color.0 = color.0.with_alpha(1.0 - fade);
    }
    if let Ok(mut color) = subtitle.single_mut() {
        color.0 = color.0.with_alpha(1.0 - fade);
    }
    if fade >= 1.0 {
        next_state.set(ChamberState::MainMenu);
    }
}

fn despawn_boot_ui(mut commands: Commands, query: Query<Entity, With<BootUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub(crate) fn spawn_main_menu(mut commands: Commands, registry: Res<ModeRegistry>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::NONE),
            GlobalZIndex(900),
            MainMenuUi,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Percent(50.0),
                    top: Val::Percent(56.0),
                    width: Val::Px(470.0),
                    min_height: Val::Px(270.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(18.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.01, 0.025, 0.045, 0.42)),
                BorderColor::all(Color::srgba(0.24, 0.78, 1.0, 0.46)),
                MainMenuPortalPanel,
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("ARCHETYPES"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.90, 0.96, 1.0)),
                ));
                for entry in registry.registrations().iter().copied() {
                    let text_color = if entry.available {
                        Color::WHITE
                    } else {
                        Color::srgb(0.42, 0.50, 0.56)
                    };
                    let border_color = if entry.available {
                        Color::srgba(0.30, 0.84, 1.0, 0.78)
                    } else {
                        Color::srgba(0.24, 0.32, 0.38, 0.50)
                    };
                    panel
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(330.0),
                                min_height: Val::Px(42.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.00, 0.08, 0.15, 0.52)),
                            BorderColor::all(border_color),
                            ModeButton {
                                mode: entry.mode,
                                available: entry.available,
                            },
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(menu_label(entry.mode, entry.available)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(text_color),
                            ));
                        });
                }
            });
        });
}

fn menu_label(mode: GameMode, available: bool) -> String {
    match (mode, available) {
        (GameMode::Standard, true) => "ENTER COUNCIL".to_owned(),
        (GameMode::OracleRiddle, true) => "ORACLE RIDDLE".to_owned(),
        (locked, false) => format!("{} - SEALED", locked.label()),
        (other, true) => other.label().to_owned(),
    }
}

fn position_main_menu_on_portal(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<WitnessCamera>>,
    portal: Query<&GlobalTransform, With<StargatePortal>>,
    mut panel: Query<&mut Node, With<MainMenuPortalPanel>>,
) {
    let (Ok(window), Ok((camera, camera_transform)), Ok(portal), Ok(mut node)) = (
        windows.single(),
        camera.single(),
        portal.single(),
        panel.single_mut(),
    ) else {
        return;
    };
    let Ok(viewport) = camera.world_to_viewport(camera_transform, portal.translation()) else {
        return;
    };
    let width = 470.0;
    let height = 270.0;
    node.left = Val::Px((viewport.x - width / 2.0).clamp(18.0, window.width() - width - 18.0));
    node.top = Val::Px((viewport.y - height * 0.35).clamp(18.0, window.height() - height - 18.0));
}

fn style_mode_buttons(
    mut interactions: Query<
        (
            &Interaction,
            &ModeButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, button, mut background, mut border) in &mut interactions {
        if !button.available {
            *background = BackgroundColor(Color::srgba(0.02, 0.03, 0.04, 0.42));
            *border = BorderColor::all(Color::srgba(0.24, 0.32, 0.38, 0.50));
            continue;
        }
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgba(0.30, 0.84, 1.0, 0.34));
                *border = BorderColor::all(Color::WHITE);
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.16, 0.54, 0.86, 0.30));
                *border = BorderColor::all(Color::srgba(0.64, 0.92, 1.0, 0.96));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.00, 0.08, 0.15, 0.52));
                *border = BorderColor::all(Color::srgba(0.30, 0.84, 1.0, 0.78));
            }
        }
    }
}

fn activate_mode(
    mut commands: Commands,
    interaction: Query<(&Interaction, &ModeButton), Changed<Interaction>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    oracle_state: Option<Res<State<crate::modes::oracle_riddle::OracleState>>>,
    standard_state: Option<Res<State<crate::modes::standard_mecha::StandardMechaState>>>,
    main_menu_entities: Query<Entity, With<MainMenuUi>>,
) {
    if let Some(state) = oracle_state {
        if *state.get() != crate::modes::oracle_riddle::OracleState::Inactive {
            return;
        }
    }
    if let Some(state) = standard_state {
        if *state.get() != crate::modes::standard_mecha::StandardMechaState::Inactive {
            return;
        }
    }
    let mut selected_mode = None;
    for (val, button) in &interaction {
        if *val == Interaction::Pressed {
            if button.available {
                selected_mode = Some(button.mode);
            } else {
                info!(
                    "{} is registered for a future lane but is not playable yet",
                    button.mode.label()
                );
            }
        }
    }

    // For now, Enter selects Standard
    if selected_mode.is_none() && keyboard.just_pressed(KeyCode::Enter) {
        selected_mode = Some(GameMode::Standard);
    }

    if let Some(mode) = selected_mode {
        commands.insert_resource(crate::chamber::ActiveGameMode(mode));
        match mode {
            GameMode::Standard => {
                for entity in &main_menu_entities {
                    commands.entity(entity).despawn();
                }
                commands.insert_resource(TriggerStandardMecha);
            }
            GameMode::OracleRiddle => {
                for entity in &main_menu_entities {
                    commands.entity(entity).despawn();
                }
                commands.insert_resource(crate::modes::oracle_riddle::TriggerOracleRiddle);
            }
            GameMode::InnerChambers => {
                for entity in &main_menu_entities {
                    commands.entity(entity).despawn();
                }
                commands.insert_resource(crate::modes::inner_chambers::TriggerInnerChambers);
            }
            GameMode::LivingEngine => {
                info!("Living Engine remains locked");
            }
        }
    }
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn title_and_subtitle_enter_in_sequence() {
        assert_eq!(
            timed_alpha(0.0, TITLE_FADE_START_SECS, TITLE_FADE_SECS),
            0.0
        );
        assert!(timed_alpha(2.0, TITLE_FADE_START_SECS, TITLE_FADE_SECS) > 0.0);
        assert_eq!(
            timed_alpha(2.0, SUBTITLE_FADE_START_SECS, SUBTITLE_FADE_SECS),
            0.0
        );
        assert_eq!(
            timed_alpha(
                SUBTITLE_FADE_START_SECS + SUBTITLE_FADE_SECS,
                SUBTITLE_FADE_START_SECS,
                SUBTITLE_FADE_SECS
            ),
            1.0
        );
    }

    #[test]
    fn boot_hold_outlasts_the_text_fades() {
        assert!(MIN_BOOT_SECS > TITLE_FADE_START_SECS + TITLE_FADE_SECS);
        assert!(MIN_BOOT_SECS > SUBTITLE_FADE_START_SECS + SUBTITLE_FADE_SECS);
        assert!(BOOT_FADE_SECS >= 3.0);
    }
}
