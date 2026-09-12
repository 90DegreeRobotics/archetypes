//! Objects the player owns: taken from the altar, carried, placed, and duplicated.
//!
//! Operator directive, 2026-09-12: *"All objects need to be individual objects that can be
//! duplicated by the player. If a player wants to fill the room with duplicates that's fine. I
//! want the objects to be able to be placed."*
//!
//! Three things had to be true before any of that was possible, and only the third is about
//! hands:
//!
//! 1. **Each manifestation is its own file.** Bevy caches by asset path, and the altar used to
//!    overwrite one. See `services/artifacts.rs`.
//! 2. **Placements survive the session.** An append-only ledger, the third instance of the
//!    pattern `build_intent.rs` and `encounter_memory.rs` already use. Without it, decorating
//!    the castle is a sandbox demo that forgets itself on quit.
//! 3. **Carrying.** A held anchor parented to the camera — first person, no arm mesh. In first
//!    person that reads correctly: the object is in your hands because it moves with your head.
//!
//! Placement needs no physics engine, which is unusual and worth saying out loud:
//! `castle::castle_surface_y` already answers the floor height at any point in the building —
//! promenade, gallery deck, stair tread, bridge, chamber, Council floor — and
//! `clamp_inside_wall` answers whether something may be there at all. The castle describes
//! itself, so putting an object down is two queries and a transform.

use bevy::prelude::*;

use crate::services::artifacts::{
    self, ArtifactRecord, Placement,
};
use crate::services::sfx::{PlaySfx, Sfx};

use super::castle;
use super::interaction::{InnerActions, InnerModalState, InteractionFocus, InteractionTarget};
use super::world::{HintPriority, HintRequest, InnerHintSet, InnerWorldElement};
use super::InnerChambersState;

/// How far in front of the player an object is set down.
///
/// Far enough to be clear of the body, close enough that the player is putting it *there*
/// rather than throwing it. A duplicate lands beside it rather than inside it.
const PLACE_DISTANCE: f32 = 2.2;

/// Sideways step between successive duplicates.
///
/// Copies alternate sides and step further out — +0.9, -0.9, +1.8, -1.8 — so pressing `R`
/// repeatedly lays out a row rather than burying every copy in the last one.
const DUPLICATE_OFFSET: f32 = 0.9;

/// Where the nth copy since pickup goes, relative to a plain placement.
fn duplicate_offset(copy: u32) -> f32 {
    let step = (copy / 2 + 1) as f32;
    let side = if copy % 2 == 0 { 1.0 } else { -1.0 };
    DUPLICATE_OFFSET * step * side
}

/// Where a carried object sits relative to the eye: forward and down, in view.
const HELD_OFFSET: Vec3 = Vec3::new(0.28, -0.42, -0.78);

/// Carried objects are shown smaller than placed ones. At full size a 1.4m artifact held 0.78m
/// from the eye fills the whole screen and the player cannot see where they are walking.
const HELD_SCALE: f32 = 0.34;

/// How long one swing takes, in seconds.
///
/// **Motion, not impact.** The held object arcs through view with a sound and a short camera
/// kick, and nothing is struck. Striking things is the Smash Room
/// (`Game Plan_ Archetypes - The Inner Chambers Smash Room.md`), which the operator retired on
/// 2026-09-11; it needs a physics crate, rigid bodies and breakable props, none of which exist.
/// A swing that only moves is most of what a swing feels like in the hand, and it promises
/// nothing the game does not deliver.
const SWING_SECONDS: f32 = 0.38;

/// How far the camera kicks at the top of a swing, in radians.
const SWING_CAMERA_KICK: f32 = 0.055;

/// Marks an object standing in the world. `interaction.rs` resolves focus against these.
#[derive(Component, Debug, Clone)]
pub struct PlacedObject {
    pub placement: String,
    pub artifact: String,
    pub asset: String,
    pub scale: f32,
}

/// Marks the visual parented to the camera while an object is in hand.
#[derive(Component)]
pub struct CarriedVisual;

/// What the player is holding, if anything.
#[derive(Resource, Default)]
pub struct Carried {
    pub artifact: Option<ArtifactRecord>,
    /// How many copies have been dropped since this object was picked up.
    ///
    /// Used to fan successive duplicates apart. Without it every copy lands at the same offset
    /// and they stack inside one another — three presses of `R` produced one visible object.
    pub copies: u32,
}

impl Carried {
    pub fn is_carrying(&self) -> bool {
        self.artifact.is_some()
    }
}

/// A swing in progress. `None` between swings.
#[derive(Resource, Default)]
pub struct Swing {
    pub elapsed: Option<f32>,
}

impl Swing {
    /// 0 at the start of the swing, 1 at the end.
    pub fn progress(&self) -> Option<f32> {
        self.elapsed.map(|elapsed| (elapsed / SWING_SECONDS).clamp(0.0, 1.0))
    }

    /// A single arc: up and through, then back. Sine rather than a linear ramp, so the object
    /// accelerates into the swing and settles out of it.
    pub fn arc(progress: f32) -> f32 {
        (progress * std::f32::consts::PI).sin()
    }
}

/// Counter for minting placement ids within a session, combined with a timestamp so ids stay
/// unique across sessions too.
#[derive(Resource, Default)]
pub struct PlacementCounter(pub u64);

pub struct ObjectsPlugin;

impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Carried>()
            .init_resource::<Swing>()
            .init_resource::<PlacementCounter>()
            .add_systems(OnEnter(InnerChambersState::Loading), spawn_standing_placements)
            .add_systems(
                Update,
                (
                    take_object,
                    take_from_altar,
                    place_or_duplicate,
                    hold_carried_object,
                    swing_held_object,
                )
                    .chain()
                    .after(super::interaction::InnerInteractionSet::Resolve)
                    .run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(
                Update,
                object_hint
                    .in_set(InnerHintSet::Request)
                    .run_if(in_state(InnerChambersState::Navigating)),
            )
            .add_systems(OnEnter(InnerChambersState::Exiting), drop_carried_on_exit);
    }
}

fn spawn_one_placement(
    commands: &mut Commands,
    asset_server: &AssetServer,
    placement: &Placement,
) {
    commands.spawn((
        SceneRoot(asset_server.load(format!("{}#Scene0", placement.asset))),
        Transform::from_xyz(
            placement.position[0],
            placement.position[1],
            placement.position[2],
        )
        .with_rotation(Quat::from_rotation_y(placement.yaw))
        .with_scale(Vec3::splat(placement.scale)),
        PlacedObject {
            placement: placement.placement.clone(),
            artifact: placement.artifact.clone(),
            asset: placement.asset.clone(),
            scale: placement.scale,
        },
        InnerWorldElement,
        Name::new(format!("PlacedObject_{}", placement.placement)),
    ));
}

/// Everything the player has put down in previous sessions, back where they left it.
fn spawn_standing_placements(mut commands: Commands, asset_server: Res<AssetServer>) {
    let standing = artifacts::load_placements();
    for placement in &standing {
        spawn_one_placement(&mut commands, &asset_server, placement);
    }
    if !standing.is_empty() {
        info!(
            "objects: restored {} placed object(s) from the ledger",
            standing.len()
        );
    }
}

fn mint_placement_id(counter: &mut PlacementCounter) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    counter.0 += 1;
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{seconds}_{}", counter.0)
}

/// Where the player is looking, on the floor, clamped to somewhere they could stand.
fn ground_target(camera: &Transform, offset: f32) -> Vec3 {
    let forward = camera.forward();
    let flat = Vec2::new(forward.x, forward.z);
    let heading = if flat.length_squared() > 1e-4 {
        flat.normalize()
    } else {
        // Looking straight up or straight down: fall back to the camera's own facing rather
        // than dividing by a zero-length vector and placing the object at the player's feet.
        Vec2::new(0.0, -1.0)
    };
    let right = Vec2::new(heading.y, -heading.x);

    let eye = Vec2::new(camera.translation.x, camera.translation.z);
    let wanted = eye + heading * PLACE_DISTANCE + right * offset;

    // The same collision the player obeys. An object cannot be put somewhere the player could
    // not have walked to, which keeps them out of masonry and out of the abyss.
    let ground = castle::clamp_inside_wall(castle::resolve_room_walls(wanted));
    let surface = castle::castle_surface_y(ground, camera.translation.y - 2.85);
    Vec3::new(ground.x, surface, ground.y)
}

/// `E` on an object standing in the world takes it into the hand.
fn take_object(
    mut commands: Commands,
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    focus: Res<InteractionFocus>,
    mut carried: ResMut<Carried>,
    placed: Query<&PlacedObject>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if modal.any() || !actions.interact {
        return;
    }
    let Some(InteractionTarget::PlacedObject(entity)) = focus.0 else {
        return;
    };
    if carried.is_carrying() {
        // One thing at a time. Swapping silently would lose whatever was in hand.
        return;
    }
    let Ok(object) = placed.get(entity) else {
        return;
    };

    // The ledger records the withdrawal rather than forgetting the placement ever happened.
    if let Err(error) = artifacts::withdraw_placement(&object.placement) {
        warn!("objects: picked up {} but could not record it: {error}", object.placement);
    }
    // Prefer the library row: a placement records an id and an asset path, but the prompt this
    // object came from and the provenance of its run live on the row. Falling back to the
    // placement keeps an object whose row was lost still pickup-able -- it is the player's
    // either way, it just cannot say where it came from.
    carried.artifact = Some(artifacts::find_artifact(&object.artifact).unwrap_or(ArtifactRecord {
        id: object.artifact.clone(),
        asset: object.asset.clone(),
        ..Default::default()
    }));
    carried.copies = 0;
    commands.entity(entity).despawn();
    sfx.write(PlaySfx::new(Sfx::PickUp));
}

/// `E` at the altar takes whatever is standing on the cushion into the hand.
///
/// This lives here rather than in `manifestation.rs` for two reasons: the altar's own input
/// handler is already at Bevy's system-parameter ceiling, and "take a thing into your hand" is
/// this module's job wherever the thing happens to be standing.
///
/// `handle_manifestation_input` returns early while an artifact is on the cushion, so one press
/// of `E` cannot both take the object and reopen the prompt.
fn take_from_altar(
    mut commands: Commands,
    actions: Res<InnerActions>,
    modal: Res<InnerModalState>,
    focus: Res<InteractionFocus>,
    mut carried: ResMut<Carried>,
    mut state: ResMut<super::manifestation::ManifestationState>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if modal.any() || !actions.interact || carried.is_carrying() {
        return;
    }
    if focus.0 != Some(InteractionTarget::ManifestationAltar) {
        return;
    }
    let (Some(entity), Some(id)) = (state.active_artifact, state.active_artifact_id.clone()) else {
        return;
    };

    commands.entity(entity).despawn();
    state.active_artifact = None;
    state.active_artifact_id = None;
    carried.artifact = Some(artifacts::find_artifact(&id).unwrap_or(ArtifactRecord {
        asset: artifacts::asset_path_for(&id),
        id,
        prompt: state.active_prompt.clone(),
        ..Default::default()
    }));
    carried.copies = 0;
    sfx.write(PlaySfx::new(Sfx::PickUp));
}

/// `F` sets the carried object down. `R` drops a copy and keeps carrying.
fn place_or_duplicate(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    modal: Res<InnerModalState>,
    asset_server: Res<AssetServer>,
    mut carried: ResMut<Carried>,
    mut counter: ResMut<PlacementCounter>,
    camera: Query<&Transform, With<super::camera::PlayerCamera>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if modal.any() {
        return;
    }
    let place = keyboard.just_pressed(KeyCode::KeyF)
        || crate::services::gamepad_input::any_just_pressed(&gamepads, GamepadButton::North);
    let duplicate = keyboard.just_pressed(KeyCode::KeyR)
        || crate::services::gamepad_input::any_just_pressed(&gamepads, GamepadButton::RightTrigger);
    if !place && !duplicate {
        return;
    }
    let Some(artifact) = carried.artifact.clone() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };

    // A duplicate lands beside where a plain placement would, and each successive copy steps
    // further out on alternating sides.
    let offset = if duplicate {
        duplicate_offset(carried.copies)
    } else {
        0.0
    };
    let position = ground_target(camera, offset);
    let placement = Placement {
        placement: mint_placement_id(&mut counter),
        artifact: artifact.id.clone(),
        asset: artifact.asset.clone(),
        position: [position.x, position.y, position.z],
        // Faces back at the player, which is how someone sets a thing down to look at it.
        yaw: camera.rotation.to_euler(EulerRot::YXZ).0 + std::f32::consts::PI,
        scale: 1.0,
    };

    if let Err(error) = artifacts::record_placement(
        &placement.placement,
        &placement.artifact,
        &placement.asset,
        placement.position,
        placement.yaw,
        placement.scale,
    ) {
        // Reported, not swallowed, and the object still goes down: a ledger that cannot be
        // written is a problem for the next session, not a reason to refuse this one. Same
        // honesty rule as `WriteReceipt` in `services/build_intent.rs`.
        warn!(
            "objects: placed {} but the ledger write failed ({error}); it will not survive a \
             restart",
            placement.placement
        );
    }
    spawn_one_placement(&mut commands, &asset_server, &placement);

    if duplicate {
        carried.copies += 1;
        sfx.write(PlaySfx::new(Sfx::Duplicate));
    } else {
        // A plain placement empties the hand; a duplicate keeps it full, which is what makes
        // filling a room with copies a matter of pressing one key repeatedly.
        carried.artifact = None;
        sfx.write(PlaySfx::new(Sfx::Place));
    }
}

/// Keeps the carried object in view, parented to the camera.
fn hold_carried_object(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    carried: Res<Carried>,
    camera: Query<Entity, With<super::camera::PlayerCamera>>,
    held: Query<Entity, With<CarriedVisual>>,
) {
    let wanted = carried.artifact.as_ref().map(|record| record.asset.clone());
    let showing = !held.is_empty();

    match (wanted, showing) {
        (Some(_), true) => {
            // Nothing to do: the swing system owns the held transform while a swing runs, and
            // `swing_held_object` puts it back at rest when one is not.
        }
        (Some(asset), false) => {
            let Ok(camera) = camera.single() else {
                return;
            };
            commands.spawn((
                SceneRoot(asset_server.load(format!("{asset}#Scene0"))),
                Transform::from_translation(HELD_OFFSET).with_scale(Vec3::splat(HELD_SCALE)),
                CarriedVisual,
                ChildOf(camera),
                Name::new("CarriedArtifact"),
            ));
        }
        (None, true) => {
            for entity in &held {
                commands.entity(entity).despawn();
            }
        }
        _ => {}
    }
}

/// Swings whatever is in hand: the object arcs through view, a sound plays, and the camera
/// kicks a little. Nothing is struck.
fn swing_held_object(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    modal: Res<InnerModalState>,
    carried: Res<Carried>,
    mut swing: ResMut<Swing>,
    mut held: Query<&mut Transform, With<CarriedVisual>>,
    mut camera: Query<
        &mut super::camera::CameraController,
        With<super::camera::PlayerCamera>,
    >,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if !carried.is_carrying() {
        // Dropping mid-swing leaves nothing to animate; clear the state rather than letting it
        // run on and apply to the next thing picked up.
        swing.elapsed = None;
        return;
    }

    let started = !modal.any()
        && swing.elapsed.is_none()
        && (mouse.just_pressed(MouseButton::Left)
            || crate::services::gamepad_input::any_just_pressed(
                &gamepads,
                GamepadButton::LeftTrigger,
            ));
    if started {
        swing.elapsed = Some(0.0);
        sfx.write(PlaySfx::new(Sfx::Swing));
        if let Ok(mut controller) = camera.single_mut() {
            controller.pitch = (controller.pitch + SWING_CAMERA_KICK).clamp(-1.54, 1.54);
        }
    }

    let Some(elapsed) = swing.elapsed else {
        // At rest: hold the object at its carry pose. Without this it would stay wherever the
        // last arc left it.
        for mut transform in &mut held {
            transform.translation = HELD_OFFSET;
            transform.rotation = Quat::IDENTITY;
        }
        return;
    };
    let elapsed = elapsed + time.delta_secs();
    if elapsed >= SWING_SECONDS {
        swing.elapsed = None;
    } else {
        swing.elapsed = Some(elapsed);
    }

    let progress = (elapsed / SWING_SECONDS).clamp(0.0, 1.0);
    let arc = Swing::arc(progress);
    for mut transform in &mut held {
        // Forward, up and rotating: the object leads with its top, which is what a swing looks
        // like from behind the hands.
        transform.translation = HELD_OFFSET
            + Vec3::new(-0.10 * arc, 0.30 * arc, -0.42 * arc);
        transform.rotation = Quat::from_axis_angle(Vec3::X, -1.5 * arc);
    }
}

/// Tells the player what the keys do, through the shared hint arbiter rather than by writing
/// the hint line directly — three systems racing to own that line was a real shipped bug.
fn object_hint(
    carried: Res<Carried>,
    focus: Res<InteractionFocus>,
    modal: Res<InnerModalState>,
    manifestation: Option<Res<super::manifestation::ManifestationState>>,
    mut hint: ResMut<HintRequest>,
) {
    if modal.any() {
        return;
    }
    let altar_holds_something = manifestation
        .map(|state| state.active_artifact.is_some())
        .unwrap_or(false);
    if carried.is_carrying() {
        hint.request(
            HintPriority::Device,
            "[F] Set it down    [R] Duplicate    [LMB] Swing".to_string(),
        );
    } else if matches!(focus.0, Some(InteractionTarget::PlacedObject(_))) {
        hint.request(HintPriority::Device, "[E] Pick it up".to_string());
    } else if altar_holds_something && focus.0 == Some(InteractionTarget::ManifestationAltar) {
        hint.request(HintPriority::Device, "[E] Take it from the altar".to_string());
    }
}

/// Leaving the castle with something in hand puts it down where the player stood, rather than
/// quietly destroying it. What the player made is theirs.
fn drop_carried_on_exit(
    mut carried: ResMut<Carried>,
    mut counter: ResMut<PlacementCounter>,
    camera: Query<&Transform, With<super::camera::PlayerCamera>>,
) {
    let Some(artifact) = carried.artifact.take() else {
        return;
    };
    let Ok(camera) = camera.single() else {
        return;
    };
    let position = ground_target(camera, 0.0);
    if let Err(error) = artifacts::record_placement(
        &mint_placement_id(&mut counter),
        &artifact.id,
        &artifact.asset,
        [position.x, position.y, position.z],
        0.0,
        1.0,
    ) {
        warn!("objects: could not set down a carried object on exit: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn looking(from: Vec3, yaw: f32) -> Transform {
        Transform::from_translation(from).with_rotation(Quat::from_axis_angle(Vec3::Y, yaw))
    }

    /// An object is put down in front of the player, on the floor, not at their feet and not
    /// floating.
    #[test]
    fn an_object_lands_in_front_of_the_player_on_the_floor() {
        let camera = looking(Vec3::new(0.0, castle::GROUND_Y + 2.85, 10.0), 0.0);
        let target = ground_target(&camera, 0.0);

        let travelled = Vec2::new(target.x, target.z).distance(Vec2::new(0.0, 10.0));
        assert!(
            (travelled - PLACE_DISTANCE).abs() < 0.05,
            "landed {travelled:.2}m away, not {PLACE_DISTANCE}m"
        );
        assert!(
            (target.y - castle::GROUND_Y).abs() < 0.01,
            "landed at y={:.2}, not on the Council floor at {}",
            target.y,
            castle::GROUND_Y
        );
    }

    /// Copies must not land inside the original, or inside each other. Pressing `R` three
    /// times used to produce one visible object with two more hidden inside it.
    #[test]
    fn every_copy_lands_clear_of_the_original_and_of_the_other_copies() {
        let camera = looking(Vec3::new(0.0, castle::GROUND_Y + 2.85, 10.0), 0.0);

        let mut spots = vec![ground_target(&camera, 0.0)];
        for copy in 0..6 {
            spots.push(ground_target(&camera, duplicate_offset(copy)));
        }

        for (left, first) in spots.iter().enumerate() {
            for second in spots.iter().skip(left + 1) {
                let apart = Vec2::new(first.x, first.z).distance(Vec2::new(second.x, second.z));
                assert!(
                    apart > 0.5,
                    "two placements landed {apart:.2}m apart, effectively inside each other"
                );
            }
        }
    }

    /// Copies alternate sides so a row grows outwards from where the player is looking, rather
    /// than marching off in one direction.
    #[test]
    fn copies_alternate_sides_and_step_outwards() {
        assert!(duplicate_offset(0) > 0.0);
        assert!(duplicate_offset(1) < 0.0);
        assert!(duplicate_offset(2) > duplicate_offset(0));
        assert!(duplicate_offset(3) < duplicate_offset(1));
        for copy in 0..12 {
            assert!(duplicate_offset(copy).abs() >= DUPLICATE_OFFSET);
        }
    }

    /// The fan resets when a new object is taken up, so a fresh object starts beside the player
    /// rather than wherever the last one's row had reached.
    #[test]
    fn picking_something_up_resets_the_duplicate_fan() {
        let mut carried = Carried::default();
        carried.copies = 5;
        carried.artifact = Some(ArtifactRecord {
            id: "a".into(),
            asset: artifacts::asset_path_for("a"),
            prompt: String::new(),
            created: String::new(),
            ..Default::default()
        });
        carried.copies = 0;
        assert_eq!(carried.copies, 0);
    }

    /// An object may not be put anywhere the player could not have walked. Placing into the
    /// masonry or over the abyss would be a way to lose something you made.
    #[test]
    fn an_object_cannot_be_placed_through_a_wall() {
        // Standing at the wall face at a blind bay, looking straight at it.
        let bearing = castle::arcade_bay_bearing(1);
        let radial = Vec3::new(bearing.cos(), 0.0, bearing.sin());
        let at_wall = radial * (castle::castle_inner_face() - 1.0)
            + Vec3::Y * (castle::museum_chamber_floor_y() + 2.85);
        let yaw = (-radial.x).atan2(-radial.z);
        let target = ground_target(&looking(at_wall, yaw), 0.0);

        let flat = Vec2::new(target.x, target.z);
        assert!(
            flat.length() <= castle::castle_inner_face(),
            "an object was placed at {:.2}m, inside the wall at {:.2}m",
            flat.length(),
            castle::castle_inner_face()
        );
    }

    /// Looking straight down must not divide by a zero-length heading.
    #[test]
    fn looking_straight_down_still_places_somewhere_sane() {
        let mut camera = looking(Vec3::new(0.0, castle::GROUND_Y + 2.85, 0.0), 0.0);
        camera.rotation = Quat::from_axis_angle(Vec3::X, -std::f32::consts::FRAC_PI_2);
        let target = ground_target(&camera, 0.0);
        assert!(target.is_finite());
        assert!(target.y.is_finite());
    }

    /// Carrying is one object at a time, and the hand empties on a plain placement.
    #[test]
    fn the_hand_holds_one_thing() {
        let mut carried = Carried::default();
        assert!(!carried.is_carrying());
        carried.artifact = Some(ArtifactRecord {
            id: "a".into(),
            asset: artifacts::asset_path_for("a"),
            prompt: String::new(),
            created: String::new(),
            ..Default::default()
        });
        assert!(carried.is_carrying());
        carried.artifact = None;
        assert!(!carried.is_carrying());
    }

    /// Held objects are shown small enough to see past. A 1.4m artifact at full size 0.78m from
    /// the eye fills the screen.
    #[test]
    fn a_held_object_does_not_fill_the_screen() {
        const AUTHORED_MAX_DIMENSION: f32 = 1.4;
        let held = AUTHORED_MAX_DIMENSION * HELD_SCALE;
        assert!(held < 0.6, "a held object is {held:.2}m across");
        assert!(held > 0.25, "a held object is only {held:.2}m across and reads as a speck");
        assert!(HELD_OFFSET.z < 0.0, "the held object must be in front of the eye");
        assert!(HELD_OFFSET.y < 0.0, "the held object must be below the centre of view");
    }

    /// A swing is one arc: it starts and ends at rest, and peaks in the middle. A linear ramp
    /// would snap back to the carry pose at the end.
    #[test]
    fn a_swing_is_one_arc_that_starts_and_ends_at_rest() {
        assert!(Swing::arc(0.0).abs() < 1e-6);
        assert!(Swing::arc(1.0).abs() < 1e-6);
        assert!((Swing::arc(0.5) - 1.0).abs() < 1e-6);
        for step in 0..=10 {
            let progress = step as f32 / 10.0;
            let arc = Swing::arc(progress);
            // `sin(PI)` in f32 is about -8.7e-8, so the endpoints need a float epsilon rather
            // than an exact range.
            assert!(
                (-1e-6..=1.0 + 1e-6).contains(&arc),
                "arc left its range at {progress}: {arc}"
            );
        }
    }

    #[test]
    fn a_swing_reports_progress_only_while_it_runs() {
        let mut swing = Swing::default();
        assert_eq!(swing.progress(), None);
        swing.elapsed = Some(SWING_SECONDS * 0.5);
        assert!((swing.progress().unwrap() - 0.5).abs() < 1e-6);
        // Past the end it saturates rather than running on.
        swing.elapsed = Some(SWING_SECONDS * 4.0);
        assert_eq!(swing.progress(), Some(1.0));
    }

    /// Placement ids must be unique, or two objects share a row and one silently replaces the
    /// other in the ledger.
    #[test]
    fn placement_ids_do_not_collide_within_a_session() {
        let mut counter = PlacementCounter::default();
        let ids: std::collections::HashSet<String> =
            (0..64).map(|_| mint_placement_id(&mut counter)).collect();
        assert_eq!(ids.len(), 64);
    }
}
