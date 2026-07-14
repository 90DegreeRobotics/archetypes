use super::{AUTHORED_COUNCIL_Y, COUNCIL_CENTER};
use crate::theme::Archetype;
use bevy::prelude::*;

pub struct SpheresPlugin;

impl Plugin for SpheresPlugin {
    fn build(&self, app: &mut App) {
        // The glass spheres are fixed to the star tetrahedron's tips. Binding only
        // records each sphere's gameplay identity and its locked world position;
        // nothing ever moves a sphere. Focus is expressed by flying the camera to
        // the selected sphere, not by relocating the sphere. The constellation as a
        // whole is lifted once to sit well above the table.
        app.add_systems(Update, (bind_authored_spheres, raise_council_constellation));
    }
}

/// Marks a constellation root that has already received the one-time runtime lift.
#[derive(Component)]
struct Raised;

/// Lift the entire authored council constellation to [`COUNCIL_CENTER`], well clear of
/// the table. The scene loads asynchronously, so this runs each frame and acts the
/// moment the nodes appear:
///
/// - `Council_Assembly` (parent of the seven vessels) is pinned to the exact target
///   height — an absolute set, so it is idempotent no matter what height the GLB baked.
/// - The seven `*_PanelSpinner` roots are authored as separate top-level nodes, so they
///   are not carried by the assembly; each gets the same lift once (marker-gated) so the
///   portraits rise with their vessels and keep their authored offset below each tip.
fn raise_council_constellation(
    mut commands: Commands,
    mut nodes: Query<(Entity, &Name, &mut Transform), Without<Raised>>,
) {
    let lift = COUNCIL_CENTER.y - AUTHORED_COUNCIL_Y;
    for (entity, name, mut transform) in &mut nodes {
        let name = name.as_str();
        if name == "Council_Assembly" {
            transform.translation.y = COUNCIL_CENTER.y;
        } else if name.ends_with("_PanelSpinner") {
            transform.translation.y += lift;
        } else {
            continue;
        }
        commands.entity(entity).insert(Raised);
    }
}

#[derive(Component)]
pub struct ArchetypeSphere {
    pub archetype: Archetype,
    /// The sphere's fixed world position at its star tip, recorded once at bind
    /// time. Consumed by the camera to aim at the selected archetype.
    pub locked_position: Vec3,
}

fn bind_authored_spheres(
    mut commands: Commands,
    query: Query<(Entity, &Name, &Transform), Without<ArchetypeSphere>>,
) {
    for (entity, name, transform) in &query {
        if let Some(archetype) = archetype_from_node_name(name.as_str()) {
            commands.entity(entity).insert(ArchetypeSphere {
                archetype,
                locked_position: transform.translation,
            });
        }
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
