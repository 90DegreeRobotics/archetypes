//! Title / loading screen and the lore-chamber launcher shell.
//!
//! Default desktop launch reveals the generated lore-compliant chamber as the
//! main menu. STANDARD MODE enters the AURA council ritual (uiscene1 + table).
//! CONSCIOUSNESS is 1:1 archetype chat. Oracle, Inner Chambers, and Living Engine
//! are playable from this menu. HELP is an in-game overlay — never a CLI.

use std::{fs, path::PathBuf};

use bevy::{
    app::AppExit,
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
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<BootSequence>()
            .add_systems(PreStartup, spawn_boot_ui)
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
                (
                    style_mode_buttons,
                    style_quit_button,
                    style_help_button,
                    activate_mode,
                    activate_quit,
                    activate_help,
                    close_help_on_escape,
                )
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
struct QuitButton;

#[derive(Component)]
struct HelpButton;

#[derive(Component)]
struct HelpOverlay;

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
                    let available = entry.available;
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
                menu.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(0.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.018, 0.020, 0.028, 0.96)),
                    BorderColor::all(Color::srgba(0.32, 0.48, 0.62, 0.72)),
                    HelpButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("HELP"),
                        TextFont {
                            font_size: 17.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.78, 0.86, 0.92)),
                    ));
                });
                menu.spawn((
                    Button,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(0.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.022, 0.018, 0.018, 0.96)),
                    BorderColor::all(Color::srgba(0.54, 0.24, 0.24, 0.72)),
                    QuitButton,
                ))
                .with_children(|button| {
                    button.spawn((
                        Text::new("QUIT GAME"),
                        TextFont {
                            font_size: 17.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.72, 0.70)),
                    ));
                });
            });
            root.spawn((
                Text::new(initial_menu_banner()),
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

fn initial_menu_banner() -> String {
    let snap = crate::services::readiness::probe_readiness();
    match snap.player_hint() {
        Some(hint) => format!("{} — {hint}", snap.banner_line()),
        None => format!("{} — council chamber ready", snap.banner_line()),
    }
}

fn menu_label(mode: GameMode) -> String {
    if mode_available(mode) {
        mode.label().to_owned()
    } else {
        format!("{} - LOCKED", mode.label())
    }
}

fn mode_available(mode: GameMode) -> bool {
    GameMode::REGISTRY
        .iter()
        .find(|entry| entry.mode == mode)
        .map(|entry| entry.available)
        .unwrap_or(false)
}

fn standby_notice(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Standard => "STANDARD MODE OPENING. THE COUNCIL AWAITS.",
        GameMode::Consciousness => "CONSCIOUSNESS OPENING.",
        GameMode::OracleRiddle => "ORACLE RIDDLE OPENING.",
        GameMode::InnerChambers => "INNER CHAMBERS OPENING.",
        GameMode::LivingEngine => "LIVING ENGINE OPENING.",
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

fn style_quit_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<QuitButton>),
    >,
) {
    for (interaction, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgba(0.52, 0.07, 0.07, 0.42));
                *border = BorderColor::all(Color::srgba(1.0, 0.72, 0.68, 0.92));
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.24, 0.05, 0.05, 0.34));
                *border = BorderColor::all(Color::srgba(0.88, 0.46, 0.42, 0.84));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.022, 0.018, 0.018, 0.96));
                *border = BorderColor::all(Color::srgba(0.54, 0.24, 0.24, 0.72));
            }
        }
    }
}

fn activate_mode(
    interaction: Query<(&Interaction, &ModeButton), Changed<Interaction>>,
    help: Query<Entity, With<HelpOverlay>>,
    mut commands: Commands,
    mut notice: Query<&mut Text, With<MainMenuNotice>>,
    main_menu: Query<Entity, With<MainMenuUi>>,
) {
    if !help.is_empty() {
        return;
    }
    for (val, button) in &interaction {
        if *val == Interaction::Pressed {
            if let Ok(mut text) = notice.single_mut() {
                text.0 = standby_notice(button.mode).to_owned();
            }
            if !button.available {
                info!("{} remains locked", button.mode.label());
                continue;
            }
            for entity in &main_menu {
                commands.entity(entity).despawn();
            }
            match button.mode {
                GameMode::Standard => {
                    commands.insert_resource(crate::chamber::ritual::TriggerCouncilChamber);
                    info!("{} selected from main menu", button.mode.label());
                }
                GameMode::Consciousness => {
                    commands.insert_resource(crate::modes::standard_mecha::TriggerStandardMecha);
                    info!("{} selected from main menu", button.mode.label());
                }
                GameMode::OracleRiddle => {
                    commands.insert_resource(crate::modes::oracle_riddle::TriggerOracleRiddle);
                    info!("{} selected from main menu", button.mode.label());
                }
                GameMode::InnerChambers => {
                    commands.insert_resource(crate::modes::inner_chambers::TriggerInnerChambers);
                    info!("{} selected from main menu", button.mode.label());
                }
                GameMode::LivingEngine => {
                    commands.insert_resource(crate::modes::living_engine::TriggerLivingEngine);
                    info!("{} selected from main menu", button.mode.label());
                }
            }
        }
    }
}

fn activate_quit(
    interaction: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    help: Query<Entity, With<HelpOverlay>>,
    mut notice: Query<&mut Text, With<MainMenuNotice>>,
    mut app_exit: MessageWriter<AppExit>,
) {
    if !help.is_empty() {
        return;
    }
    for val in &interaction {
        if *val == Interaction::Pressed {
            if let Ok(mut text) = notice.single_mut() {
                text.0 = "CLOSING ARCHETYPES.".to_owned();
            }
            info!("Quit Game selected from main menu");
            app_exit.write(AppExit::Success);
        }
    }
}

fn style_help_button(
    mut interactions: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<HelpButton>),
    >,
) {
    for (interaction, mut background, mut border) in &mut interactions {
        match *interaction {
            Interaction::Pressed => {
                *background = BackgroundColor(Color::srgba(0.16, 0.32, 0.48, 0.42));
                *border = BorderColor::all(Color::srgba(0.84, 0.92, 1.0, 0.92));
            }
            Interaction::Hovered => {
                *background = BackgroundColor(Color::srgba(0.08, 0.16, 0.26, 0.34));
                *border = BorderColor::all(Color::srgba(0.62, 0.78, 0.92, 0.84));
            }
            Interaction::None => {
                *background = BackgroundColor(Color::srgba(0.018, 0.020, 0.028, 0.96));
                *border = BorderColor::all(Color::srgba(0.32, 0.48, 0.62, 0.72));
            }
        }
    }
}

const HELP_BODY: &str = "ARCHETYPES — THE WITNESS MANUAL\n\n\
You are the Witness: the sovereign eighth seat. The seven archetypes advise. They never replace you.\n\n\
MODES\n\
• STANDARD MODE — Offer language to the AURA council. Three voices confer. A verdict collapses. Chronos paints the authorized image.\n\
• CONSCIOUSNESS — Sit with one archetype. Direct counsel, memory, and a painted reply.\n\
• ORACLE RIDDLE — A hidden three-word vision. Reconstruct the prompt. Insight is the reward.\n\
• INNER CHAMBERS — Walk seven minds around a hub. Align with a node and press E to extract a truth. That truth can seed the next Oracle round.\n\
• LIVING ENGINE — Tune three orbital resonances. Keep the Aura from starving or overloading. Viren is entropy; counter-frequency cures it.\n\n\
ESC returns to this menu from every mode.\n\n\
SERVICES\n\
The Desktop launcher starts Ollama, Chronos Director, and ComfyUI if they are installed and down. Chronos remains a sibling product; Archetypes does not download it.\n\
Council voices (Kokoro / sherpa-onnx) install beside the game. The launcher repairs them if they are missing.\n\n\
LOGS\n\
%LOCALAPPDATA%\\NeuroCognica\\Archetypes\\logs\\last-failure.txt\n\
%LOCALAPPDATA%\\NeuroCognica\\Archetypes\\logs\\last-engine.log\n\
If the chamber will not open, that failure file is the truth.\n\n\
UNINSTALL\n\
Start Menu → Uninstall Archetypes, or run the uninstall shortcut the installer created. Your Witness profile can be kept.\n\n\
Esc closes this help.";

fn activate_help(
    interaction: Query<&Interaction, (Changed<Interaction>, With<HelpButton>)>,
    existing: Query<Entity, With<HelpOverlay>>,
    mut commands: Commands,
) {
    if !existing.is_empty() {
        return;
    }
    for val in &interaction {
        if *val == Interaction::Pressed {
            spawn_help_overlay(&mut commands);
        }
    }
}

fn spawn_help_overlay(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(64.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.82)),
            GlobalZIndex(980),
            HelpOverlay,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    width: Val::Px(820.0),
                    max_width: Val::Percent(92.0),
                    max_height: Val::Percent(88.0),
                    padding: UiRect::all(Val::Px(28.0)),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.045, 0.055, 0.96)),
                BorderColor::all(Color::srgba(0.78, 0.84, 0.90, 0.35)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(HELP_BODY),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.88, 0.90, 0.92)),
                ));
            });
        });
}

fn close_help_on_escape(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    overlay: Query<Entity, With<HelpOverlay>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
}

fn despawn_main_menu(
    mut commands: Commands,
    query: Query<Entity, With<MainMenuUi>>,
    help: Query<Entity, With<HelpOverlay>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    for entity in &help {
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
    fn menu_labels_expose_all_five_playable_modes() {
        assert_eq!(menu_label(GameMode::Standard), "STANDARD MODE");
        assert_eq!(menu_label(GameMode::Consciousness), "CONSCIOUSNESS");
        assert_eq!(menu_label(GameMode::OracleRiddle), "ORACLE RIDDLE");
        assert_eq!(menu_label(GameMode::InnerChambers), "INNER CHAMBERS");
        assert_eq!(menu_label(GameMode::LivingEngine), "LIVING ENGINE");
        assert!(GameMode::REGISTRY.iter().all(|entry| entry.available));
    }

    #[test]
    fn help_manual_names_every_mode_and_the_log_path() {
        assert!(HELP_BODY.contains("STANDARD MODE"));
        assert!(HELP_BODY.contains("CONSCIOUSNESS"));
        assert!(HELP_BODY.contains("ORACLE RIDDLE"));
        assert!(HELP_BODY.contains("INNER CHAMBERS"));
        assert!(HELP_BODY.contains("LIVING ENGINE"));
        assert!(HELP_BODY.contains("last-failure.txt"));
        assert!(HELP_BODY.contains("Ollama"));
        assert!(HELP_BODY.contains("Chronos"));
    }
}
