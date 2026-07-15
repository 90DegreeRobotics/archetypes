//! Title / loading screen and the current lore-chamber launcher shell.
//!
//! The old table/chamber/menu visuals are parked behind an explicit legacy gate,
//! while default desktop launch reveals the generated lore-compliant chamber.
//! The main menu stays in standby until the rebuilt gameplay path is approved.

use std::{fs, path::PathBuf};

use bevy::{
    prelude::*,
    render::view::window::screenshot::{save_to_disk, Screenshot},
};

use super::ChamberState;
use crate::modes::{game_mode::GameMode, ModeRegistry};

/// Minimum time the title holds — long enough that the heavy scene textures finish
/// uploading behind it, so the reveal does not hitch.
const MIN_BOOT_SECS: f32 = 8.0;
const BOOT_FADE_SECS: f32 = 3.4;
const TITLE_FADE_START_SECS: f32 = 0.8;
const TITLE_FADE_SECS: f32 = 2.6;
const SUBTITLE_FADE_START_SECS: f32 = 3.2;
const SUBTITLE_FADE_SECS: f32 = 2.8;

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
                (style_mode_buttons, activate_mode)
                    .chain()
                    .run_if(in_state(ChamberState::MainMenu)),
            )
            .add_systems(OnExit(ChamberState::MainMenu), despawn_main_menu);

        if let Some(run) = BlankShellCaptureRun::from_env() {
            app.insert_resource(run)
                .add_systems(Update, run_blank_shell_capture);
        }
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
struct MainMenuNotice;

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
    mut sequence: ResMut<BootSequence>,
    mut overlay: Query<&mut BackgroundColor, With<BootUi>>,
    mut title: Query<&mut TextColor, (With<BootTitle>, Without<BootSubtitle>)>,
    mut subtitle: Query<&mut TextColor, (With<BootSubtitle>, Without<BootTitle>)>,
    mut next_state: ResMut<NextState<ChamberState>>,
) {
    let elapsed = time.elapsed_secs();
    if elapsed < MIN_BOOT_SECS {
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
                padding: UiRect::axes(Val::Px(64.0), Val::Px(50.0)),
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
                    left: Val::Px(64.0),
                    top: Val::Px(56.0),
                    width: Val::Percent(36.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.92, 0.88, 0.76, 0.58)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(64.0),
                    top: Val::Px(56.0),
                    width: Val::Px(1.0),
                    height: Val::Percent(72.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.92, 0.88, 0.76, 0.42)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(64.0),
                    bottom: Val::Px(56.0),
                    width: Val::Percent(30.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.42, 0.74, 0.86, 0.44)),
            ));
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(92.0),
                    top: Val::Px(88.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|brand| {
                brand.spawn((
                    Text::new("ARCHETYPES"),
                    TextFont {
                        font_size: 42.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.94, 0.93, 0.88)),
                ));
                brand.spawn((
                    Text::new("MAIN MENU"),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.73, 0.83)),
                ));
            });
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(92.0),
                    top: Val::Percent(37.0),
                    width: Val::Px(430.0),
                    max_width: Val::Percent(72.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Stretch,
                    row_gap: Val::Px(12.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|menu| {
                for entry in registry.registrations().iter().copied() {
                    let available = false;
                    let text_color = if available {
                        Color::WHITE
                    } else {
                        Color::srgb(0.54, 0.58, 0.60)
                    };
                    menu.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(48.0),
                            justify_content: JustifyContent::FlexStart,
                            align_items: AlignItems::Center,
                            padding: UiRect::axes(Val::Px(18.0), Val::Px(0.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.018, 0.020, 0.022, 0.96)),
                        BorderColor::all(Color::srgba(0.22, 0.24, 0.25, 0.92)),
                        ModeButton {
                            mode: entry.mode,
                            available,
                        },
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Text::new(menu_label(entry.mode)),
                            TextFont {
                                font_size: 17.0,
                                ..default()
                            },
                            TextColor(text_color),
                        ));
                    });
                }
            });
            root.spawn((
                Text::new("COUNCIL CHAMBER ONLINE"),
                TextFont {
                    font_size: 15.0,
                    ..default()
                },
                TextColor(Color::srgb(0.70, 0.72, 0.70)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(92.0),
                    bottom: Val::Px(72.0),
                    max_width: Val::Percent(72.0),
                    ..default()
                },
                MainMenuNotice,
            ));
        });
}

fn menu_label(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Standard => "STANDARD MODE - STANDBY",
        GameMode::OracleRiddle => "ORACLE RIDDLE - STANDBY",
        GameMode::InnerChambers => "INNER CHAMBERS - LOCKED",
        GameMode::LivingEngine => "LIVING ENGINE - LOCKED",
    }
}

fn standby_notice(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Standard => "STANDARD MODE AWAITS THE NEW WORLD.",
        GameMode::OracleRiddle => "ORACLE RIDDLE IS STANDING BY OFF THIS PATH.",
        GameMode::InnerChambers => "INNER CHAMBERS REMAIN LOCKED.",
        GameMode::LivingEngine => "LIVING ENGINE REMAINS LOCKED.",
    }
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
            match *interaction {
                Interaction::Pressed => {
                    *background = BackgroundColor(Color::srgba(0.07, 0.075, 0.072, 0.98));
                    *border = BorderColor::all(Color::srgba(0.76, 0.70, 0.54, 0.75));
                }
                Interaction::Hovered => {
                    *background = BackgroundColor(Color::srgba(0.045, 0.050, 0.052, 0.98));
                    *border = BorderColor::all(Color::srgba(0.42, 0.74, 0.86, 0.70));
                }
                Interaction::None => {
                    *background = BackgroundColor(Color::srgba(0.018, 0.020, 0.022, 0.96));
                    *border = BorderColor::all(Color::srgba(0.22, 0.24, 0.25, 0.92));
                }
            }
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
    interaction: Query<(&Interaction, &ModeButton), Changed<Interaction>>,
    mut notice: Query<&mut Text, With<MainMenuNotice>>,
) {
    for (val, button) in &interaction {
        if *val == Interaction::Pressed {
            if let Ok(mut text) = notice.single_mut() {
                text.0 = standby_notice(button.mode).to_owned();
            }
            if button.available {
                info!(
                    "{} is registered but the blank-slate shell is holding mode entry in standby",
                    button.mode.label()
                );
            } else {
                info!(
                    "{} remains in standby on the blank-slate shell",
                    button.mode.label()
                );
            }
        }
    }
}

fn despawn_main_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[derive(Resource)]
struct BlankShellCaptureRun {
    dir: PathBuf,
    menu_stem: &'static str,
    title_shot: bool,
    subtitle_shot: bool,
    menu_seen: Option<f32>,
    menu_shot: bool,
    exit_at: Option<f32>,
}

impl BlankShellCaptureRun {
    fn from_env() -> Option<Self> {
        let lore_capture = std::env::var_os("ARCHETYPES_LORE_CAPTURE").is_some();
        if !lore_capture && std::env::var_os("ARCHETYPES_BLANK_CAPTURE").is_none() {
            return None;
        }
        let dir = std::env::var_os("ARCHETYPES_CAPTURE_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if lore_capture {
                    PathBuf::from("artifacts/visual-proof/lore-chamber-runtime")
                } else {
                    PathBuf::from("artifacts/visual-proof/blank-slate-shell")
                }
            });
        let _ = fs::create_dir_all(&dir);
        Some(Self {
            dir,
            menu_stem: if lore_capture {
                "02_lore_main_menu"
            } else {
                "02_blank_main_menu"
            },
            title_shot: false,
            subtitle_shot: false,
            menu_seen: None,
            menu_shot: false,
            exit_at: None,
        })
    }

    fn shot(&self, commands: &mut Commands, stem: &str) {
        let path = self.dir.join(format!("{stem}.png"));
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn run_blank_shell_capture(
    time: Res<Time>,
    chamber_state: Res<State<ChamberState>>,
    mut capture: ResMut<BlankShellCaptureRun>,
    mut commands: Commands,
) {
    let now = time.elapsed_secs();
    if !capture.title_shot && now >= 2.6 {
        capture.shot(&mut commands, "00_title_arch");
        capture.title_shot = true;
    }
    if !capture.subtitle_shot && now >= 5.4 {
        capture.shot(&mut commands, "01_title_subtitle");
        capture.subtitle_shot = true;
    }

    if let Some(exit_at) = capture.exit_at {
        if now >= exit_at {
            std::process::exit(0);
        }
        return;
    }

    if *chamber_state.get() == ChamberState::MainMenu {
        let seen = *capture.menu_seen.get_or_insert(now);
        if !capture.menu_shot && now - seen >= 1.0 {
            capture.shot(&mut commands, capture.menu_stem);
            capture.menu_shot = true;
            capture.exit_at = Some(now + 1.5);
        }
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

    #[test]
    fn blank_menu_labels_keep_modes_in_standby() {
        assert_eq!(menu_label(GameMode::Standard), "STANDARD MODE - STANDBY");
        assert_eq!(
            menu_label(GameMode::OracleRiddle),
            "ORACLE RIDDLE - STANDBY"
        );
        assert_eq!(
            menu_label(GameMode::InnerChambers),
            "INNER CHAMBERS - LOCKED"
        );
        assert_eq!(menu_label(GameMode::LivingEngine), "LIVING ENGINE - LOCKED");
    }
}
