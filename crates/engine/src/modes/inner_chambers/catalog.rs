use bevy::prelude::*;

use crate::theme::Archetype;

pub const NODE_RADIUS: f32 = 3.25;
pub const HUB_RADIUS: f32 = 22.0;

#[derive(Clone, Copy)]
pub struct ChamberSpec {
    pub archetype: Archetype,
    pub title: &'static str,
    pub origin: Vec3,
    pub truths: [[&'static str; 3]; 2],
}

pub fn council_chambers() -> [ChamberSpec; 7] {
    [
        chamber(0, Archetype::Architect, "LUMINOUS BLUEPRINT", [
            ["Order", "Structure", "Grid"],
            ["Load", "Span", "Joint"],
        ]),
        chamber(1, Archetype::Sentinel, "NULL AEGIS", [
            ["Threshold", "Oath", "Wall"],
            ["Watch", "Permit", "Deny"],
        ]),
        chamber(2, Archetype::Mentor, "ANCIENT RESONANCE", [
            ["Patience", "Bowl", "Hum"],
            ["Lesson", "Echo", "Root"],
        ]),
        chamber(3, Archetype::Explorer, "FRONTIER FLARE", [
            ["Trail", "Horizon", "Spark"],
            ["Map", "Risk", "Dawn"],
        ]),
        chamber(4, Archetype::Oracle, "NOCTIS VEIL", [
            ["Pattern", "Veil", "Bell"],
            ["Omen", "Mirror", "Night"],
        ]),
        chamber(5, Archetype::Empath, "LUMA RESONANCE", [
            ["Breath", "Heart", "Rose"],
            ["Hold", "Warmth", "Tide"],
        ]),
        chamber(6, Archetype::Jester, "JESTER", [
            ["Crack", "Joke", "Door"],
            ["Glitch", "Flip", "Key"],
        ]),
    ]
}

fn chamber(
    index: usize,
    archetype: Archetype,
    title: &'static str,
    truths: [[&'static str; 3]; 2],
) -> ChamberSpec {
    ChamberSpec {
        archetype,
        title,
        origin: room_origin(index),
        truths,
    }
}

pub fn room_origin(index: usize) -> Vec3 {
    let angle = index as f32 * std::f32::consts::TAU / 7.0;
    Vec3::new(angle.sin() * HUB_RADIUS, 0.0, angle.cos() * HUB_RADIUS)
}

pub fn node_positions(spec: &ChamberSpec) -> [Vec3; 2] {
    [
        spec.origin + Vec3::new(-3.5, 2.0, -2.0),
        spec.origin + Vec3::new(3.5, 2.0, 2.0),
    ]
}

pub fn all_nodes() -> Vec<(usize, usize, Vec3, [&'static str; 3])> {
    council_chambers()
        .iter()
        .enumerate()
        .flat_map(|(chamber_index, spec)| {
            node_positions(spec)
                .into_iter()
                .enumerate()
                .map(move |(node_index, position)| {
                    (chamber_index, node_index, position, spec.truths[node_index])
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_chambers_have_fourteen_distinct_triples() {
        let chambers = council_chambers();
        assert_eq!(chambers.len(), 7);
        let mut seen = std::collections::BTreeSet::new();
        for spec in chambers {
            for words in spec.truths {
                seen.insert(words);
            }
        }
        assert_eq!(seen.len(), 14);
    }

    #[test]
    fn rooms_sit_on_a_heptagon_around_the_hub() {
        let origins: Vec<Vec3> = council_chambers().iter().map(|spec| spec.origin).collect();
        for origin in &origins {
            assert!((origin.length() - HUB_RADIUS).abs() < 0.01);
        }
        assert!(origins.iter().all(|origin| origin.y.abs() < 0.01));
    }
}
