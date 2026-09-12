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
use super::manifestation::{
    altar_interaction_anchor, ManifestationPhase, ManifestationState,
};
use super::workshop::WorkshopState;
use super::world::ArchitectWorkshopTable;
use super::InnerChambersState;

pub const ALTAR_INTERACTION_RANGE: f32 = 3.2;
pub const WORKSHOP_INTERACTION_RANGE: f32 = 3.0;

/// How close the player must be to pick a placed object back up.
///
/// Measured in 3D from the eye, like every other range here, so the vertical climb has to be
/// paid for out of the same budget: an object resting on the floor sits about 2.15m below a
/// standing eye even after the anchor below lifts it. At 2.6 the vertical alone exhausted the
/// range and nothing could ever be picked up. 3.0 leaves roughly 2.1m of horizontal reach.
pub const OBJECT_PICKUP_RANGE: f32 = 3.0;

/// A placed object is reached for at its middle, not at the floor it stands on.
///
/// The same correction the altar needed: its transform sits on the ground, and reaching for the
/// ground spends the whole range going downwards.
pub const OBJECT_GRAB_HEIGHT: f32 = 0.7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InteractionTarget {
    ManifestationAltar,
    ArchitectWorkshop,
    /// An object the player placed, identified by the entity standing there.
    PlacedObject(Entity),
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
            // A thing the player is standing over and could pick up outranks even a device.
            // Objects get placed on and around the altar - that is the point of being able to
            // place them - and when one is at your feet, `E` should mean "take this", not
            // "reopen the prompt you used to make it".
            InteractionTarget::PlacedObject(_) => 0,
            InteractionTarget::ManifestationAltar | InteractionTarget::ArchitectWorkshop => 1,
            InteractionTarget::Archetype(_) => 2,
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
    pub library: bool,
    pub workshop: bool,
    pub settings: bool,
}

impl InnerModalState {
    pub fn any(&self) -> bool {
        self.encounter || self.manifestation || self.library || self.workshop || self.settings
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
    library: Option<Res<super::library::LibraryState>>,
    workshop: Option<Res<WorkshopState>>,
    settings_menu: Option<Res<super::settings_menu::SettingsMenuState>>,
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
        library: library.map(|state| state.is_open()).unwrap_or(false),
        workshop: workshop.map(|state| state.is_open()).unwrap_or(false),
        settings: settings_menu.map(|menu| menu.open).unwrap_or(false),
    };
}

fn resolve_focus(
    camera: Query<&Transform, With<PlayerCamera>>,
    embodiments: Query<(&Transform, &ArchetypeEmbodiment)>,
    workshop_table: Query<&Transform, With<ArchitectWorkshopTable>>,
    placed: Query<(Entity, &GlobalTransform), With<super::objects::PlacedObject>>,
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
    let placed_objects: Vec<(Vec3, Entity)> = placed
        .iter()
        .map(|(entity, global)| {
            (global.translation() + Vec3::Y * OBJECT_GRAB_HEIGHT, entity)
        })
        .collect();

    focus.0 = pick_target(
        camera.translation,
        altar_interaction_anchor(),
        workshop_table.iter().next().map(|transform| transform.translation),
        &embodiments,
        &placed_objects,
    );
}

/// Pure target choice, so the rule can be tested at the real coordinates the castle uses.
pub(super) fn pick_target(
    player: Vec3,
    altar: Vec3,
    workshop_table: Option<Vec3>,
    embodiments: &[(Vec3, ArchetypeEmbodiment)],
    placed_objects: &[(Vec3, Entity)],
) -> Option<InteractionTarget> {
    let mut candidates: Vec<(u8, f32, InteractionTarget)> = Vec::new();

    for (position, entity) in placed_objects {
        let distance = player.distance(*position);
        if distance <= OBJECT_PICKUP_RANGE {
            let target = InteractionTarget::PlacedObject(*entity);
            candidates.push((target.priority(), distance, target));
        }
    }

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

    // Real castle coordinates. The Architect room centre sits at (0, 0, -74) with its radial
    // pointing away from the hub, so its figure stands 7m behind the centre at z = -81 and the
    // bench's interaction anchor at z = -85.3, y = 1.55. Standing eye height is y = 3.25.
    const FIGURE: Vec3 = Vec3::new(0.0, 0.42, -81.0);
    const BENCH_ANCHOR: Vec3 = Vec3::new(0.0, 1.55, -85.3);
    const FAR_ALTAR: Vec3 = Vec3::new(0.0, 0.30, 3.4);

    #[test]
    fn altar_has_a_tighter_range_than_an_embodied_conversation() {
        assert!(ALTAR_INTERACTION_RANGE < ENCOUNTER_RANGE);
        assert!(WORKSHOP_INTERACTION_RANGE < ENCOUNTER_RANGE);
    }

    #[test]
    fn standing_at_the_bench_targets_the_workshop_even_though_the_architect_is_also_in_range() {
        let player = Vec3::new(0.0, 3.25, -84.4);
        // Both really are in range; this is the ambiguity the priority rule exists to settle.
        assert!(player.distance(BENCH_ANCHOR) <= WORKSHOP_INTERACTION_RANGE);
        assert!(player.distance(FIGURE) <= ENCOUNTER_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)], &[]),
            Some(InteractionTarget::ArchitectWorkshop)
        );
    }

    #[test]
    fn a_device_outranks_a_conversation_that_is_strictly_closer() {
        // Ranking by distance alone would flip `E` between a device and a conversation from
        // one step to the next. At the Architect bench the device happens to be nearer as
        // well, so the rule is pinned here directly rather than relying on that coincidence.
        let player = Vec3::new(0.0, 3.25, -84.4);
        let crowding_figure = Vec3::new(0.0, 3.0, -84.6);
        assert!(player.distance(crowding_figure) < player.distance(BENCH_ANCHOR));
        assert!(player.distance(crowding_figure) <= ENCOUNTER_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(crowding_figure, ARCHITECT)], &[]),
            Some(InteractionTarget::ArchitectWorkshop)
        );
    }

    #[test]
    fn stepping_back_from_the_bench_returns_focus_to_the_architect() {
        let player = Vec3::new(0.0, 3.25, -77.0);
        assert!(player.distance(BENCH_ANCHOR) > WORKSHOP_INTERACTION_RANGE);
        assert!(player.distance(FIGURE) <= ENCOUNTER_RANGE);
        assert_eq!(
            pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)], &[]),
            Some(InteractionTarget::Archetype(ARCHITECT))
        );
    }

    #[test]
    fn nothing_in_range_yields_no_target() {
        let player = Vec3::new(0.0, 3.25, -60.0);
        assert_eq!(pick_target(player, FAR_ALTAR, Some(BENCH_ANCHOR), &[(FIGURE, ARCHITECT)], &[]), None);
    }

    #[test]
    fn the_altar_also_outranks_a_closer_conversation() {
        let player = Vec3::new(0.0, 3.25, 3.4);
        let bystander = Vec3::new(0.0, 1.0, 3.2);
        assert!(player.distance(FAR_ALTAR) <= ALTAR_INTERACTION_RANGE);
        assert!(player.distance(bystander) < player.distance(FAR_ALTAR));
        assert_eq!(
            pick_target(player, FAR_ALTAR, None, &[(bystander, ARCHITECT)], &[]),
            Some(InteractionTarget::ManifestationAltar)
        );
    }

    /// An object standing at the player's feet outranks the altar it is standing on. Placing
    /// things on and around the altar is the point of being able to place things; if `E` still
    /// meant "open the prompt" there, the object could never be picked back up.
    #[test]
    fn an_object_underfoot_outranks_the_altar_it_stands_on() {
        let player = Vec3::new(0.0, 3.25, 1.2);
        let altar = Vec3::new(0.0, 2.02, 0.0);
        let object = Vec3::new(0.0, 0.9, 1.1);
        let entity = Entity::from_raw_u32(7).expect("valid test entity");

        assert!(player.distance(altar) <= ALTAR_INTERACTION_RANGE);
        assert!(player.distance(object) <= OBJECT_PICKUP_RANGE);
        assert_eq!(
            pick_target(player, altar, None, &[], &[(object, entity)]),
            Some(InteractionTarget::PlacedObject(entity))
        );
    }

    /// A player standing in front of an object on the floor must actually be able to reach it.
    /// Ranges here are 3D from a 3.25m eye, so a floor-level target spends most of the budget
    /// going downwards - which is exactly why 2.6m reached nothing at all.
    #[test]
    fn a_player_standing_at_an_object_can_reach_it() {
        let floor = 0.4_f32;
        let eye = Vec3::new(0.0, floor + 2.85, 1.5);
        let object = Vec3::new(0.0, floor, 0.0) + Vec3::Y * OBJECT_GRAB_HEIGHT;
        let entity = Entity::from_raw_u32(9).expect("valid test entity");

        assert!(
            eye.distance(object) <= OBJECT_PICKUP_RANGE,
            "standing 1.5m away the eye is {:.2}m from the object, beyond the {OBJECT_PICKUP_RANGE}m reach",
            eye.distance(object)
        );
        assert_eq!(
            pick_target(eye, Vec3::new(0.0, 99.0, 99.0), None, &[], &[(object, entity)]),
            Some(InteractionTarget::PlacedObject(entity))
        );
    }

    /// Step away from the object and the altar comes back, rather than `E` going dead.
    #[test]
    fn stepping_off_an_object_returns_focus_to_the_altar() {
        let player = Vec3::new(0.0, 3.25, 2.4);
        let altar = Vec3::new(0.0, 2.02, 0.0);
        let object = Vec3::new(0.0, 0.9, -1.6);
        let entity = Entity::from_raw_u32(8).expect("valid test entity");

        assert!(player.distance(object) > OBJECT_PICKUP_RANGE);
        assert_eq!(
            pick_target(player, altar, None, &[], &[(object, entity)]),
            Some(InteractionTarget::ManifestationAltar)
        );
    }

    #[test]
    fn a_modal_snapshot_reports_ownership_for_any_open_surface() {
        assert!(!InnerModalState::default().any());
        assert!(InnerModalState { workshop: true, ..Default::default() }.any());
        assert!(InnerModalState { manifestation: true, ..Default::default() }.any());
        assert!(InnerModalState { library: true, ..Default::default() }.any());
        assert!(InnerModalState { encounter: true, ..Default::default() }.any());
    }
}
