use super::InnerChambersState;
use crate::modes::game_mode::GameMode;
use crate::services::ledger::append_to_ledger;
use bevy::prelude::*;
use serde_json::json;

use super::world::{InnerChambersHint, ARCHITECT_NODE_POSITIONS, ARCHITECT_NODE_RADIUS};

const EXTRACTED_TRUTH: [&str; 3] = ["Order", "Structure", "Grid"];

pub struct ExtractionPlugin;

impl Plugin for ExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            check_extraction.run_if(in_state(InnerChambersState::Navigating)),
        );
    }
}

// Logic for reading the archetype's mind.
// The player navigates to specific grid intersections to extract a "truth".
fn check_extraction(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<super::camera::PlayerCamera>>,
    mut hint: Query<&mut Text, With<InnerChambersHint>>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    let Ok(transform) = query.single() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(InnerChambersState::Exiting);
        return;
    }

    let aligned_node = nearest_truth_node(transform.translation);
    if let Ok(mut text) = hint.single_mut() {
        text.0 = if aligned_node.is_some() {
            "NODE ALIGNED\nPress E to read: Order / Structure / Grid".to_owned()
        } else {
            "Find a blue node. E reads only when aligned. Esc returns.".to_owned()
        };
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some(node_index) = aligned_node else {
        if let Ok(mut text) = hint.single_mut() {
            text.0 = "No node aligned. Move closer to a blue monolith.".to_owned();
        }
        return;
    };

    let payload = inner_chamber_truth_payload(transform.translation, node_index);
    if let Err(error) = append_to_ledger(
        GameMode::InnerChambers,
        "inner_chamber_truth_extracted",
        payload,
    ) {
        warn!("inner chamber ledger seal failed: {error}");
        if let Ok(mut text) = hint.single_mut() {
            text.0 = "Ledger seal failed. The chamber did not accept the reading.".to_owned();
        }
        return;
    }

    next_state.set(InnerChambersState::Exiting);
}

pub(crate) fn nearest_truth_node(position: Vec3) -> Option<usize> {
    ARCHITECT_NODE_POSITIONS
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (position.distance(*node) <= ARCHITECT_NODE_RADIUS)
                .then_some((index, position.distance(*node)))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(index, _)| index)
}

pub(crate) fn inner_chamber_truth_payload(location: Vec3, node_index: usize) -> serde_json::Value {
    json!({
        "archetype": "Architect",
        "node_index": node_index,
        "extracted_truth": EXTRACTED_TRUTH,
        "location": [location.x, location.y, location.z]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extraction_requires_truth_node_proximity() {
        assert_eq!(nearest_truth_node(ARCHITECT_NODE_POSITIONS[0]), Some(0));
        assert_eq!(nearest_truth_node(Vec3::ZERO), None);
    }

    #[test]
    fn truth_payload_records_node_location_and_three_words() {
        let payload = inner_chamber_truth_payload(Vec3::new(1.0, 2.0, 3.0), 4);
        assert_eq!(payload["archetype"], "Architect");
        assert_eq!(payload["node_index"], 4);
        assert_eq!(payload["extracted_truth"].as_array().unwrap().len(), 3);
        assert_eq!(payload["location"][0], 1.0);
        assert_eq!(payload["location"][1], 2.0);
        assert_eq!(payload["location"][2], 3.0);
    }
}
