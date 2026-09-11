//! One deterministic focus decision for Inner Castle context actions.
//!
//! Systems may display their own contextual panel, but they must not independently
//! decide what `E` means. The closest eligible embodied archetype or manifestation
//! altar owns the action for that frame.

use bevy::prelude::*;

use super::camera::PlayerCamera;
use super::encounters::{ArchetypeEmbodiment, ENCOUNTER_RANGE};
use super::manifestation::MANIFESTATION_PEDESTAL_POS;
use super::InnerChambersState;

pub const ALTAR_INTERACTION_RANGE: f32 = 3.2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionTarget {
    ManifestationAltar,
    Archetype(ArchetypeEmbodiment),
}

#[derive(Resource, Default, Debug)]
pub struct InteractionFocus(pub Option<InteractionTarget>);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InnerInteractionSet { Resolve }

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionFocus>()
            .configure_sets(Update, InnerInteractionSet::Resolve.run_if(in_state(InnerChambersState::Navigating)))
            .add_systems(Update, resolve_focus.in_set(InnerInteractionSet::Resolve));
    }
}

fn resolve_focus(
    camera: Query<&Transform, With<PlayerCamera>>,
    embodiments: Query<(&Transform, &ArchetypeEmbodiment)>,
    mut focus: ResMut<InteractionFocus>,
) {
    let Ok(camera) = camera.single() else { focus.0 = None; return; };
    let mut candidates = Vec::new();
    let altar_distance = camera.translation.distance(MANIFESTATION_PEDESTAL_POS);
    if altar_distance <= ALTAR_INTERACTION_RANGE { candidates.push((altar_distance, 1_u8, InteractionTarget::ManifestationAltar)); }
    for (transform, embodiment) in &embodiments {
        let distance = camera.translation.distance(transform.translation);
        if distance <= ENCOUNTER_RANGE { candidates.push((distance, 0_u8, InteractionTarget::Archetype(*embodiment))); }
    }
    focus.0 = candidates.into_iter().min_by(|left, right| left.0.total_cmp(&right.0).then(left.1.cmp(&right.1))).map(|(_, _, target)| target);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn altar_has_a_tighter_range_than_an_embodied_conversation() {
        assert!(ALTAR_INTERACTION_RANGE < ENCOUNTER_RANGE);
    }
}
