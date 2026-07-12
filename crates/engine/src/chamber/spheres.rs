use super::{ChamberState, CurrentFocus};
use crate::theme::Archetype;
use bevy::prelude::*;

pub struct SpheresPlugin;

impl Plugin for SpheresPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (bind_authored_spheres, animate_spheres).chain());
    }
}

#[derive(Component)]
pub struct ArchetypeSphere {
    pub archetype: Archetype,
    pub authored_translation: Vec3,
}

fn bind_authored_spheres(
    mut commands: Commands,
    query: Query<(Entity, &Name, &Transform), Without<ArchetypeSphere>>,
) {
    for (entity, name, transform) in &query {
        if let Some(archetype) = archetype_from_node_name(name.as_str()) {
            commands.entity(entity).insert(ArchetypeSphere {
                archetype,
                authored_translation: transform.translation,
            });
        }
    }
}

fn animate_spheres(
    time: Res<Time>,
    state: Res<State<ChamberState>>,
    current_focus: Res<CurrentFocus>,
    mut query: Query<(&mut Transform, &ArchetypeSphere)>,
) {
    for (mut transform, sphere) in &mut query {
        let is_focused = *state.get() == ChamberState::FocusArchetype
            && current_focus.0 == Some(sphere.archetype);
        let target = if is_focused {
            Vec3::ZERO
        } else {
            sphere.authored_translation
        };
        let response = if is_focused { 3.5 } else { 2.0 };

        transform.translation = transform
            .translation
            .lerp(target, (time.delta_secs() * response).min(1.0));

        let target_scale = if is_focused {
            Vec3::splat(1.5)
        } else {
            Vec3::ONE
        };
        transform.scale = transform
            .scale
            .lerp(target_scale, (time.delta_secs() * response).min(1.0));
    }
}

fn archetype_from_node_name(name: &str) -> Option<Archetype> {
    match name {
        "Architect" => Some(Archetype::Architect),
        "Sentinel" => Some(Archetype::Sentinel),
        "Jester" => Some(Archetype::Jester),
        "Mentor" => Some(Archetype::Mentor),
        "Explorer" => Some(Archetype::Explorer),
        "Oracle" => Some(Archetype::Oracle),
        "Empath" => Some(Archetype::Empath),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exported_node_names_map_to_all_seven_council_archetypes() {
        let names = [
            "Architect",
            "Sentinel",
            "Jester",
            "Mentor",
            "Explorer",
            "Oracle",
            "Empath",
        ];
        assert!(names
            .into_iter()
            .all(|name| archetype_from_node_name(name).is_some()));
        assert_eq!(archetype_from_node_name("Witness"), None);
    }
}
