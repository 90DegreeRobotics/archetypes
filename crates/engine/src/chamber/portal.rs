//! The stargate portal at the centre of the table.
//!
//! The table's `Stargate_Portal` disc carries an emissive vortex texture; spinning the
//! disc around its normal swirls that vortex, giving the living-wavefield effect where
//! the Witness places intent. The disc is authored in `table.glb`; the engine only
//! animates it.

use bevy::prelude::*;

/// Portal swirl speed (radians per second).
const PORTAL_SPIN: f32 = 0.5;

pub struct PortalPlugin;

impl Plugin for PortalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (bind_portal, spin_portal).chain());
    }
}

#[derive(Component)]
pub(super) struct StargatePortal;

fn bind_portal(mut commands: Commands, query: Query<(Entity, &Name), Without<StargatePortal>>) {
    for (entity, name) in &query {
        if name.as_str() == "Stargate_Portal" {
            commands.entity(entity).insert(StargatePortal);
        }
    }
}

fn spin_portal(
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut Transform, Option<&MeshMaterial3d<StandardMaterial>>), With<StargatePortal>>,
) {
    let pulse = 2.4 + (time.elapsed_secs() * 1.7).sin() * 0.8;
    for (mut transform, material_handle) in &mut query {
        transform.rotate_local_y(PORTAL_SPIN * time.delta_secs());
        if let Some(material) = material_handle.and_then(|handle| materials.get_mut(handle)) {
            material.emissive = LinearRgba::new(0.18 * pulse, 0.48 * pulse, pulse, 1.0);
        }
    }
}
