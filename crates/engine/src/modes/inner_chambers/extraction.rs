use super::catalog::{all_nodes, council_chambers, NODE_RADIUS};
use super::interaction::{InnerActions, InnerInteractionSet, InnerModalState};
use super::seed::persist_extracted_truth;
use super::world::{HintPriority, HintRequest, InnerHintSet};
use super::InnerChambersState;
use crate::modes::game_mode::GameMode;
use crate::services::ledger::append_to_ledger;
use bevy::prelude::*;
use serde_json::json;

/// The truth-node overlay is parked, not deleted.
///
/// `catalog.rs` places its fourteen nodes on the retired heptagon (seven rooms around a 22m
/// hub), while the live Seed-of-Life castle puts six rooms at 62m. Measured against the world
/// as built, six of the fourteen hang over the abyss, and the reachable ones sit beside
/// whichever bridge happens to pass nearby — the Architect's own pair lands ~0.8m from the
/// *Empath* bridge, so "LUMINOUS BLUEPRINT / NODE 1 ALIGNED" reads out in the wrong room
/// entirely. Re-anchoring the nodes to the real rooms is its own unit of work; until then the
/// overlay stays off rather than competing for `E` and the hint line with the systems that are
/// correctly placed. `persist_extracted_truth` and the Oracle Riddle seeding it feeds are
/// untouched.
const TRUTH_NODES_ENABLED: bool = false;

pub struct ExtractionPlugin;

impl Plugin for ExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (leave_inner_chambers, check_extraction)
                .after(InnerInteractionSet::Resolve)
                .in_set(InnerHintSet::Request)
                .run_if(in_state(InnerChambersState::Navigating)),
        );
    }
}

/// The only binding that leaves the castle. It stands down whenever a modal owns input, so
/// cancelling a conversation, a manifestation, or a plan edit can never also eject the player
/// out of the mode in the same frame.
fn leave_inner_chambers(
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    if modal.any() || !actions.cancel {
        return;
    }
    next_state.set(InnerChambersState::Exiting);
}

fn check_extraction(
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    query: Query<&Transform, With<super::camera::PlayerCamera>>,
    mut hint: ResMut<HintRequest>,
    mut next_state: ResMut<NextState<InnerChambersState>>,
) {
    if !TRUTH_NODES_ENABLED || modal.any() {
        return;
    }
    let Ok(transform) = query.single() else {
        return;
    };

    let Some((chamber_index, node_index, words)) = nearest_truth_node(transform.translation) else {
        return;
    };
    let spec = council_chambers()[chamber_index];
    hint.request(
        HintPriority::TruthNode,
        format!(
            "{}\nNODE {} ALIGNED\nPress E to read: {} / {} / {}",
            spec.title,
            node_index + 1,
            words[0],
            words[1],
            words[2]
        ),
    );

    if !actions.interact {
        return;
    }

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
        hint.request(
            HintPriority::Modal,
            "Ledger seal failed. The chamber did not accept the reading.",
        );
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
