//! Archetype world encapsulation.
//!
//! Translates the focused archetype's design tokens (`ArchetypeTheme`) into an
//! actual rendered environment. Before this system the theme registry was dead
//! code: fully specified palettes that were never applied to anything the player
//! sees. Crossing into an archetype now inverts the *law of the world* — the
//! ceremonial dark void gives way to that archetype's void and ambient light —
//! rather than merely swapping a caption.
//!
//! The mechanism is general (driven by `CurrentFocus`), so any archetype the
//! chamber can focus is honoured; only the Architect is reachable in the current
//! vertical slice.

use bevy::prelude::*;

use super::{ChamberState, CurrentFocus, COUNCIL_CENTER};
use crate::modes::inner_chambers::InnerChambersState;
use crate::theme::Archetype;

/// Deep-space navy — the resting backdrop of the chamber. Deliberately not pure
/// black: an absolute-black void left every shiny surface with nothing to catch and
/// "broke the whole look." The starfield skybox ([`super::sky`]) renders over this,
/// so this tone only shows in any seam the skybox does not cover — matched to the
/// star map's base navy so the seam is invisible.
const CEREMONIAL_VOID: Color = Color::srgb(0.02, 0.028, 0.06);
/// Cool, dim ambient so the authored Blender lights carry the resting scene.
const CEREMONIAL_AMBIENT: Color = Color::srgb(0.60, 0.65, 0.82);
const CEREMONIAL_BRIGHTNESS: f32 = 90.0;

/// How aggressively the archetype floods its world with ambient light.
const INTERIOR_BRIGHTNESS: f32 = 1500.0;

/// Per-second lerp response for the environment transition. Kept smooth so the
/// crossing feels inevitable rather than a hard cut.
const ENV_RESPONSE: f32 = 1.6;

pub struct InteriorPlugin;

impl Plugin for InteriorPlugin {
    fn build(&self, app: &mut App) {
        // Start pure black so Booting never flashes the ceremonial navy void.
        // `drive_interior_environment` restores CEREMONIAL_VOID after the title veil.
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(GlobalAmbientLight {
                color: CEREMONIAL_AMBIENT,
                brightness: CEREMONIAL_BRIGHTNESS,
                ..default()
            })
            .add_systems(
                Update,
                (drive_interior_environment, sync_council_interior).chain(),
            );
    }
}

/// Lerp the world void and ambient light toward the focused archetype's tokens
/// while inside its world, and back to the ceremonial void otherwise.
fn drive_interior_environment(
    state: Res<State<ChamberState>>,
    inner_state: Option<Res<State<InnerChambersState>>>,
    focus: Res<CurrentFocus>,
    time: Res<Time>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    // Boot owns a pure-black veil. Never paint the ceremonial navy / chamber void
    // underneath it — that flash of the menu world before the title was a real bug.
    if *state.get() == ChamberState::Booting {
        clear.0 = Color::BLACK;
        return;
    }

    // While navigating the Inner Chambers rotunda, elevate ambient lighting
    // so the grand 76m hall, pillars, and 22m ceiling vaulting are clearly visible.
    let is_inner_chambers = inner_state
        .as_ref()
        .map(|s| *s.get() == InnerChambersState::Navigating || *s.get() == InnerChambersState::Loading)
        .unwrap_or(false);

    // While a council member holds the floor, the world takes on that archetype's
    // environment; at all other times it rests in the ceremonial void.
    let inside_world = matches!(state.get(), ChamberState::CouncilSpeaking);

    let (ambient_color, brightness) = if is_inner_chambers {
        (Color::srgb(0.78, 0.82, 0.95), 480.0)
    } else {
        match (inside_world, focus.0) {
            (true, Some(archetype)) => {
                let theme = archetype.theme();
                let light = theme.accent_secondary.unwrap_or(theme.bg_void);
                (light, INTERIOR_BRIGHTNESS)
            }
            _ => (CEREMONIAL_AMBIENT, CEREMONIAL_BRIGHTNESS),
        }
    };

    let t = (time.delta_secs() * ENV_RESPONSE).min(1.0);
    clear.0 = if is_inner_chambers {
        Color::srgb(0.08, 0.09, 0.12)
    } else {
        CEREMONIAL_VOID
    };
    ambient.color = lerp_color(ambient.color, ambient_color, t);
    ambient.brightness += (brightness - ambient.brightness) * t;
}

#[derive(Component)]
struct CouncilInteriorElement;

#[derive(Resource, Default)]
struct SpawnedInteriorFocus(Option<Archetype>);

fn sync_council_interior(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    mut spawned: Local<SpawnedInteriorFocus>,
    existing: Query<Entity, With<CouncilInteriorElement>>,
) {
    let desired = matches!(state.get(), ChamberState::CouncilSpeaking)
        .then_some(focus.0)
        .flatten();
    if spawned.0 == desired && (desired.is_none() || !existing.is_empty()) {
        return;
    }
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    spawned.0 = desired;
    let Some(archetype) = desired else {
        return;
    };
    spawn_speaking_interior(&mut commands, &mut meshes, &mut materials, archetype);
}

fn spawn_speaking_interior(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    archetype: Archetype,
) {
    let theme = archetype.theme();
    let mat = materials.add(StandardMaterial {
        base_color: theme.accent_primary.with_alpha(0.55),
        emissive: LinearRgba::from(theme.accent_primary) * theme.glow_intensity,
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    let center = COUNCIL_CENTER;
    match archetype {
        Archetype::Architect => {
            let line = meshes.add(Cuboid::new(18.0, 0.06, 0.06));
            for i in -3..=3 {
                let offset = i as f32 * 1.6;
                commands.spawn((
                    Mesh3d(line.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(center + Vec3::new(0.0, -1.2, offset)),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Sentinel => {
            let wall = meshes.add(Cuboid::new(10.0, 8.0, 0.12));
            for yaw in [0.4_f32, -0.4] {
                commands.spawn((
                    Mesh3d(wall.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(center + Vec3::new(0.0, 0.5, -3.5))
                        .with_rotation(Quat::from_rotation_y(yaw)),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Mentor => {
            for radius in [2.4, 3.8, 5.2] {
                commands.spawn((
                    Mesh3d(meshes.add(Cuboid::new(radius * 2.0, 0.1, 0.1))),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(center + Vec3::Y * -0.8),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Explorer => {
            let shard = meshes.add(Cuboid::new(0.35, 0.35, 2.8));
            for i in 0..6 {
                let t = i as f32 / 5.0;
                commands.spawn((
                    Mesh3d(shard.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(
                        center + Vec3::new((t - 0.5) * 7.0, -0.4, (t - 0.5) * 5.0),
                    ),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Oracle => {
            commands.spawn((
                Mesh3d(meshes.add(Cuboid::new(9.0, 7.5, 0.08))),
                MeshMaterial3d(mat),
                Transform::from_translation(center + Vec3::new(0.0, 0.8, -5.0)),
                CouncilInteriorElement,
            ));
        }
        Archetype::Empath => {
            let orb = meshes.add(Sphere::new(0.45));
            for i in 0..7 {
                let a = i as f32 * std::f32::consts::TAU / 7.0;
                commands.spawn((
                    Mesh3d(orb.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(
                        center + Vec3::new(a.sin() * 3.4, -0.2, a.cos() * 3.4),
                    ),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Jester => {
            let cube = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
            for i in 0..5 {
                commands.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(
                        center + Vec3::new((i as f32 - 2.0) * 1.6, -0.6 + (i % 2) as f32 * 0.8, 2.0),
                    )
                    .with_rotation(Quat::from_euler(EulerRot::XYZ, 0.5, 0.6 * i as f32, 0.2)),
                    CouncilInteriorElement,
                ));
            }
        }
        Archetype::Codex | Archetype::Viren => {}
    }
}

/// Frame-rate independent color lerp in linear space.
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let a = from.to_linear();
    let b = to.to_linear();
    Color::LinearRgba(LinearRgba {
        red: a.red + (b.red - a.red) * t,
        green: a.green + (b.green - a.green) * t,
        blue: a.blue + (b.blue - a.blue) * t,
        alpha: a.alpha + (b.alpha - a.alpha) * t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_color_reaches_endpoints() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        let start = lerp_color(black, white, 0.0).to_linear();
        assert!(start.red < 0.001);

        let end = lerp_color(black, white, 1.0).to_linear();
        assert!(end.red > 0.999);
    }

    #[test]
    fn the_render_backdrop_is_deep_space_not_pure_black() {
        // The resting backdrop is intentionally dark but never pure black: an
        // absolute-black void gave reflective surfaces nothing to catch. It must
        // stay dark enough to sit behind the starfield without competing with it.
        let void = CEREMONIAL_VOID.to_linear();
        assert!(void.red > 0.0 || void.green > 0.0 || void.blue > 0.0);
        assert!(void.red < 0.1 && void.green < 0.1 && void.blue < 0.1);
    }
}
