use super::{ChamberState, CurrentFocus};
use crate::theme::Archetype;
use bevy::prelude::*;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_portal)
            .add_systems(Update, handle_portal_input);
    }
}

fn setup_portal(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The Table
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(1.5, 0.2))),
        MeshMaterial3d(materials.add(Color::srgb(0.1, 0.1, 0.15))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    // The Portal
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.0, 0.2, 0.8), // deep blue portal
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.11, 0.0),
    ));
}

fn handle_portal_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<State<ChamberState>>,
    mut next_state: ResMut<NextState<ChamberState>>,
    mut current_focus: ResMut<CurrentFocus>,
) {
    if keyboard.just_pressed(KeyCode::Space) {
        match state.get() {
            ChamberState::IdleAtTable => {
                println!("Prompt submitted. Witness invokes the council.");
                next_state.set(ChamberState::Deliberating);
            }
            ChamberState::Deliberating => {
                current_focus.0 = Some(Archetype::Architect);
                println!("The Architect takes the focus lane.");
                next_state.set(ChamberState::FocusArchetype);
            }
            ChamberState::FocusArchetype => {
                current_focus.0 = None;
                println!("Returning to table.");
                next_state.set(ChamberState::IdleAtTable);
            }
        }
    }
}
