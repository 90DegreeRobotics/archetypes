//! One deterministic decision per frame about what the player's context keys mean.
//!
//! Systems may display their own panel, but they must not independently decide what `E` or
//! `Esc` means. Three separate systems used to poll the keyboard for `E` and three for `Esc`,
//! with no arbitration: a single `Esc` could close a modal *and* exit the whole mode in the
//! same frame. Focus, actions, and modal ownership are all resolved here, before any consumer
//! runs, and consumers read the resolved values.

use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::services::gamepad_input;

use super::camera::PlayerCamera;
use super::encounters::{ArchetypeEmbodiment, EncounterState, ENCOUNTER_RANGE};
use super::manifestation::{ManifestationPhase, ManifestationState, MANIFESTATION_PEDESTAL_POS};
use super::workshop::WorkshopState;
use super::world::ArchitectWorkshopTable;
use super::InnerChambersState;

pub const ALTAR_INTERACTION_RANGE: f32 = 3.2;
pub const WORKSHOP_INTERACTION_RANGE: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionTarget {
    ManifestationAltar,
    ArchitectWorkshop,
    Archetype(ArchetypeEmbodiment),
}

impl InteractionTarget {
    /// A physical device the player is standing at outranks a conversation that merely happens
    /// to be within talking range. Both are true at once more often than not — the Architect's
    /// bench sits a few metres behind the Architect's own figure, and talking range is more than
    /// twice the device range — so ranking by distance alone made `E` flip meaning with a single
    /// step.
    fn priority(&self) -> u8 {
        match self {
            InteractionTarget::ManifestationAltar | InteractionTarget::ArchitectWorkshop => 0,
            InteractionTarget::Archetype(_) => 1,
        }
    }
}

#[derive(Resource, Default, Debug)]
pub struct InteractionFocus(pub Option<InteractionTarget>);

/// This frame's arbitrated context actions, resolved once from keyboard and every connected
/// gamepad so one physical press cannot be counted separately by three different systems.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct InnerActions {
    pub interact: bool,
    pub cancel: bool,
}

/// Snapshot of which surface owns input this frame, taken before any consumer runs.
///
/// Consumers that act on the *world* (open a conversation, use a device, leave the mode) must
/// stand down whenever any of these is true; the owning modal handles its own keys.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct InnerModalState {
    pub encounter: bool,
    pub manifestation: bool,
    pub workshop: bool,
}

impl InnerModalState {
    pub fn any(&self) -> bool {
        self.encounter || self.manifestation || self.workshop
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InnerInteractionSet {
    Resolve,
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InteractionFocus>()
            .init_resource::<InnerActions>()
            .init_resource::<InnerModalState>()
            .configure_sets(
                Update,
                InnerInteractionSet::Resolve.run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(
                Update,
                (resolve_actions, resolve_modal_state, resolve_focus)
                    .in_set(InnerInteractionSet::Resolve),
            );
    }
}

fn resolve_actions(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut actions: ResMut<InnerActions>,
) {
    actions.interact = keyboard.just_pressed(KeyCode::KeyE)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::West);
    actions.cancel = keyboard.just_pressed(KeyCode::Escape)
        || gamepad_input::any_just_pressed(&gamepads, GamepadButton::East);
}

fn resolve_modal_state(
    encounter: Option<Res<EncounterState>>,
    manifestation: Option<Res<ManifestationState>>,
    workshop: Option<Res<WorkshopState>>,
    mut modal: ResMut<InnerModalState>,
) {
    // `Manifesting` counts as owning input even though it is not a typing surface: `Esc` during
    // a render must cancel that job, not eject the player out of the castle.
    let manifestation = manifestation
        .map(|state| {
            matches!(
                state.phase,
                ManifestationPhase::Prompting | ManifestationPhase::Manifesting
            )
        })
        .unwrap_or(false);
    *modal = InnerModalState {
        encounter: encounter.map(|state| state.is_open()).unwrap_or(false),
        manifestation,
        workshop: workshop.map(|state| state.is_open()).unwrap_or(false),
    };
}

fn resolve_focus(
    camera: Query<&Transform, With<PlayerCamera>>,
    embodiments: Query<(&Transform, &ArchetypeEmbodiment)>,
    workshop_table: Query<&Transform, With<ArchitectWorkshopTable>>,
    mut focus: ResMut<InteractionFocus>,
) {
    let Ok(camera) = camera.single() else {
        focus.0 = None;
        return;
    };
    let embodiments: Vec<(Vec3, ArchetypeEmbodiment)> = embodiments
        .iter()
        .map(|(transform, embodiment)| (transform.translation, *embodiment))
        .collect();
    focus.0 = pick_target(
        camera.translation,
        MANIFESTATION_PEDESTAL_POS,
        workshop_table.iter().next().map(|transform| transform.translation),
        &embodiments,
    );
}

/// Pure target choice, so the rule can be tested at the real coordinates the castle uses.
pub(super) fn pick_target(
    player: Vec3,
    altar: Vec3,
    workshop_table: Option<Vec3>,
    embodiments: &[(Vec3, ArchetypeEmbodiment)],
) -> Option<InteractionTarget> {
    let mut candidates: Vec<(u8, f32, InteractionTarget)> = Vec::new();

    let altar_distance = player.distance(altar);
    if altar_distance <= ALTAR_INTERACTION_RANGE {
        let target = InteractionTarget::ManifestationAltar;
        candidates.push((target.priority(), altar_distance, target));
    }

    if let Some(table) = workshop_table {
        let distance = player.distance(table);
        if distance <= WORKSHOP_INTERACTION_RANGE {
            let target = InteractionTarget::ArchitectWorkshop;
            candidates.push((target.priority(), distance, target));
        }
    }

    for (position, embodiment) in embodiments {
        let distance = player.distance(*position);
        if distance <= ENCOUNTER_RANGE {
            let target = InteractionTarget::Archetype(*embodiment);
            candidates.push((target.priority(), distance, target));
        }
    }

    candidates
        .into_iter()
        .min_by(|left, right| left.0.cmp(&right.0).then(left.1.total_cmp(&right.1)))
        .map(|(_, _, target)| target)
}

/// One place that owns the cursor when a modal opens or closes, instead of each surface
/// setting both fields itself and drifting.
pub(super) fn set_modal_cursor(
    cursor: &mut Query<&mut CursorOptions, With<PrimaryWindow>>,
    free: bool,
) {
    if let Ok(mut cursor) = cursor.single_mut() {
        cursor.visible = free;
        cursor.grab_mode = if free {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Archetype;

    const ARCHITECT: ArchetypeEmbodiment = ArchetypeEmbodiment {
        archetype: Archetype::Architect,
        chamber_title: "LuminousBlueprint",
    };

    // Real castle coordinates: the Architect room centre is (0, 0, -62) with its radial
    // pointing away from the hub, so its figure stands at z = -67 and the workshop bench's
    // interaction anchor at z = -69.9, y = 1.55. Standing eye height on the room floor is
    // y = 3.25.
    const FIGURE: Vec3 = Vec3::new(0.0, 0.42, -67.0);
    const BENCH_ANCHOR: Vec3 = Vec3::new(0.0, 1.55, -69.9);
    const FAR_ALTAR: Vec3 = Vec3::new(0.0, 0.30, 3.4);

    #[test]
    fn altar_has_a_tighter_range_than_an_embodied_conversation() {
        assert!(ALTAR_INTERACTION_RANGE < ENCOUNTER_RANGE);
        assert!(WORKSHOP_INTERACTION_RANGE < ENCOUNTER_RANGE);
    }

    #[test]
    fn standing_at_the_bench_targets_the_workshop_even_though_the_architect_is_also_in_range() {
        let player = Vec3::new(0.0, 3.25, -69.0);
        // Both really are in range; this is the ambiguity the priority rule exists to settle.
        assert!(player.distance(BENCH_ANCHOR) <= WORKSHOP_INTERACTION_RANGE);
        assert!(player.distance(FIGURE) <= ENCOUNTER_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)]),
            Some(InteractionTarget::ArchitectWorkshop)
        );
    }

    #[test]
    fn the_bench_still_wins_when_the_architect_is_the_closer_of_the_two() {
        // Just behind the figure and within reach of the bench. Ranking by distance alone
        // would flip `E` to a conversation here, one step away from flipping back.
        let player = Vec3::new(0.0, 3.25, -67.5);
        assert!(player.distance(FIGURE) < player.distance(BENCH_ANCHOR));
        assert!(player.distance(BENCH_ANCHOR) <= WORKSHOP_INTERACTION_RANGE);
        assert!(player.distance(FIGURE) <= ENCOUNTER_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)]),
            Some(InteractionTarget::ArchitectWorkshop)
        );
    }

    #[test]
    fn stepping_back_to_the_room_centre_returns_focus_to_the_architect() {
        let player = Vec3::new(0.0, 3.25, -62.0);
        assert!(player.distance(BENCH_ANCHOR) > WORKSHOP_INTERACTION_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)]),
            Some(InteractionTarget::Archetype(ARCHITECT))
        );
    }

    #[test]
    fn nothing_in_range_yields_no_target() {
        let player = Vec3::new(0.0, 3.25, -30.0);
        assert_eq!(pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)]), None);
    }

    #[test]
    fn the_altar_also_outranks_a_closer_conversation() {
        let player = Vec3::new(0.0, 3.25, 3.4);
        let bystander = Vec3::new(0.0, 1.0, 3.2);
        assert!(player.distance(FAR_ALTAR) <= ALTAR_INTERACTION_RANGE);
        assert!(player.distance(bystander) < player.distance(FAR_ALTAR));
        assert_eq!(
            pick_target(player, FAR_ALTAR, None, &[(bystander, ARCHITECT)]),
            Some(InteractionTarget::ManifestationAltar)
        );
    }

    #[test]
    fn a_modal_snapshot_reports_ownership_for_any_open_surface() {
        assert!(!InnerModalState::default().any());
        assert!(InnerModalState { workshop: true, ..Default::default() }.any());
        assert!(InnerModalState { manifestation: true, ..Default::default() }.any());
        assert!(InnerModalState { encounter: true, ..Default::default() }.any());
    }
}
