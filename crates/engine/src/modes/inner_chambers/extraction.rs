use super::catalog::{all_nodes, council_chambers, NODE_RADIUS};
use super::seed::persist_extracted_truth;
use super::world::InnerChambersHint;
use super::InnerChambersState;
use super::encounters::EncounterState;
use crate::modes::game_mode::GameMode;
use crate::services::ledger::append_to_ledger;
use bevy::prelude::*;
use serde_json::json;

pub struct ExtractionPlugin;

impl Plugin for ExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            check_extraction.run_if(in_state(InnerChambersState::Navigating)),
        );
    }
}

fn check_extraction(
    keyboard: Res<ButtonInput<KeyCode>>,
    query: Query<(&Transform, &super::camera::CameraController), With<super::camera::PlayerCamera>>,
    mut hint: Query<&mut Text, With<InnerChambersHint>>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
    encounter_state: Res<EncounterState>,
) {
    if encounter_state.is_open() {
        return;
    }
    let Ok((transform, controller)) = query.single() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Escape) {
        next_state.set(InnerChambersState::Exiting);
        return;
    }

    let aligned = nearest_truth_node(transform.translation);
    if let Ok(mut text) = hint.single_mut() {
        text.0 = if let Some((chamber_index, node_index, words)) = aligned {
            let spec = council_chambers()[chamber_index];
            format!(
                "{}\nNODE {} ALIGNED\nPress E to read: {} / {} / {}",
                spec.title,
                node_index + 1,
                words[0],
                words[1],
                words[2]
            )
        } else {
            controller.locomotion_hud_text()
        };
    }

    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some((chamber_index, node_index, words)) = aligned else {
        if let Ok(mut text) = hint.single_mut() {
            text.0 = "No node aligned. Enter a chamber and stand with a luminous node.".to_owned();
        }
        return;
    };

    let spec = council_chambers()[chamber_index];
    let payload = inner_chamber_truth_payload(
        transform.translation,
        chamber_index,
        node_index,
        spec.archetype.theme().name,
        words,
    );
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
    if let Err(error) = persist_extracted_truth(words, node_index, spec.archetype.theme().name) {
        warn!("inner chamber truth persist failed: {error}");
    }

    next_state.set(InnerChambersState::Exiting);
}

pub(crate) fn nearest_truth_node(position: Vec3) -> Option<(usize, usize, [&'static str; 3])> {
    all_nodes()
        .into_iter()
        .filter_map(|(chamber_index, node_index, node, words)| {
            (position.distance(node) <= NODE_RADIUS)
                .then_some((chamber_index, node_index, words, position.distance(node)))
        })
        .min_by(|a, b| a.3.total_cmp(&b.3))
        .map(|(chamber_index, node_index, words, _)| (chamber_index, node_index, words))
}

pub(crate) fn inner_chamber_truth_payload(
    location: Vec3,
    chamber_index: usize,
    node_index: usize,
    archetype: &str,
    words: [&str; 3],
) -> serde_json::Value {
    json!({
        "archetype": archetype,
        "chamber_index": chamber_index,
        "node_index": node_index,
        "extracted_truth": words,
        "location": [location.x, location.y, location.z]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modes::inner_chambers::catalog::node_positions;

    #[test]
    fn extraction_requires_truth_node_proximity() {
        let spec = council_chambers()[0];
        let node = node_positions(&spec)[0];
        let hit = nearest_truth_node(node).expect("architect node");
        assert_eq!(hit.0, 0);
        assert_eq!(hit.2[0], "Order");
        assert_eq!(nearest_truth_node(Vec3::ZERO), None);
    }

    #[test]
    fn each_chamber_has_distinct_concrete_triples() {
        let mut seen = std::collections::BTreeSet::new();
        for spec in council_chambers() {
            for words in spec.truths {
                seen.insert(words);
            }
        }
        assert_eq!(seen.len(), 14);
    }

    #[test]
    fn truth_payload_records_which_mind_was_read() {
        let payload = inner_chamber_truth_payload(
            Vec3::new(1.0, 2.0, 3.0),
            4,
            1,
            "Noctis Veil",
            ["Pattern", "Veil", "Bell"],
        );
        assert_eq!(payload["archetype"], "Noctis Veil");
        assert_eq!(payload["chamber_index"], 4);
        assert_eq!(payload["extracted_truth"].as_array().unwrap().len(), 3);
    }
}
