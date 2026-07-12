//! Title / loading screen.
//!
//! The chamber is a heavy GPU scene; on launch nothing of it should be shown until it
//! is ready. A full-screen title overlay covers everything from the first frame and is
//! only lifted once the authored council geometry has actually loaded (all seven vessels
//! bound), at which point the ritual begins — at the table for a returning Witness, or
//! at onboarding for a new one.

use bevy::prelude::*;

use super::{ritual::RitualSession, spheres::ArchetypeSphere, ChamberState};

/// Minimum time the title holds — long enough that the heavy scene textures finish
/// uploading behind it, so the reveal does not hitch.
const MIN_BOOT_SECS: f32 = 3.0;
/// The seven council vessels; when all are bound the scene is ready.
const READY_SPHERES: usize = 7;

pub struct BootPlugin;

impl Plugin for BootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_boot_ui)
            .add_systems(Update, boot_ready.run_if(in_state(ChamberState::Booting)))
            .add_systems(OnExit(ChamberState::Booting), despawn_boot_ui);
    }
}

#[derive(Component)]
struct BootUi;

fn spawn_boot_ui(mut commands: Commands) {
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
            BackgroundColor(Color::srgb(0.015, 0.016, 0.022)),
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
                TextColor(Color::srgb(0.95, 0.95, 0.96)),
            ));
            parent.spawn((
                Text::new("Council Chamber"),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::srgb(0.62, 0.66, 0.78)),
            ));
            parent.spawn((
                Text::new("Awakening the chamber..."),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.42, 0.45, 0.55)),
            ));
        });
}

fn boot_ready(
    time: Res<Time>,
    spheres: Query<&ArchetypeSphere>,
    session: Res<RitualSession>,
    mut next_state: ResMut<NextState<ChamberState>>,
) {
    if time.elapsed_secs() < MIN_BOOT_SECS || spheres.iter().count() < READY_SPHERES {
        return;
    }
    if session.has_profile() {
        next_state.set(ChamberState::IdleAtTable);
    } else {
        next_state.set(ChamberState::Onboarding);
    }
}

fn despawn_boot_ui(mut commands: Commands, query: Query<Entity, With<BootUi>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
