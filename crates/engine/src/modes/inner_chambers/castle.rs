//! The castle's dimensional truth, in one place.
//!
//! Every number the world builder spawns from and every number the player collides against
//! comes from here, so the wall you can touch is the wall that was drawn and the stair you can
//! climb is the stair that was modelled.
//!
//! # Why the previous ascent had to be replaced
//!
//! The old perimeter stair ran 1,120 treads over seven full circuits of an ~84m radius: about
//! 3,705m of travel to gain 21.08m. That is a **1.9cm riser on a 3.31m tread — a 0.33° grade**.
//! No stair is 0.33°; that is a runway with slabs on it, and it read as one.
//!
//! # The geometry now
//!
//! Stairs are dimensioned from the same rules a real building uses. With a 12.0m floor-to-floor
//! storey split into 72 risers:
//!
//! * riser `12.0 / 72 = 166.7mm` — inside the 180mm limit typical of public stairs
//! * tread (going) `300mm`
//! * Blondel comfort rule `2R + T = 2(166.7) + 300 = 633mm` — inside the 600-660mm band
//! * pitch `atan(166.7 / 300) = 29.1°` — a monumental stair (the Spanish Steps sit near 27°)
//!
//! A storey therefore needs `72 x 0.30 = 21.6m` of going, plus two 3.0m landings, so `27.6m`
//! of run. Wrapped onto the inner wall at a 108m radius that is `27.6 / 108 = 0.2556 rad`, or
//! **14.64° of the circle per storey**.
//!
//! That last number is the whole reason the old stair was impossible: a flight that climbs a
//! real storey only consumes about a fifteenth of the circumference, so "one full circuit per
//! level" and "real stairs" cannot both be true at this radius. Real buildings resolve it the
//! way this one now does — the *stairs* climb and the *walkways* do the circling. Each storey
//! advances 60° around the ring: 14.64° of stair, then 45.36° of gallery to reach the next
//! flight. Seven storeys make 420°, so the ascent is a little over one full circuit of the
//! building and about 792m of travel, roughly two minutes of running to reach the top gallery.

use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_6, PI, TAU};

// ---------------------------------------------------------------------------------------
// Shell
// ---------------------------------------------------------------------------------------

/// Centreline radius of the enclosing wall.
pub const CASTLE_RADIUS: f32 = 120.0;
/// A 96m wall carries real load; it is built with real mass rather than a 2.4m screen.
pub const CASTLE_WALL_THICKNESS: f32 = 12.0;
pub const CASTLE_WALL_HEIGHT: f32 = 96.0;
/// Vault springs from the wall head and crowns 30m higher, so the interior clear height is
/// 126m — a little under the Pantheon's height-to-width proportion at this diameter.
pub const VAULT_RISE: f32 = 30.0;

pub const COUNCIL_RADIUS: f32 = 24.0;
pub const OUTER_ROOM_RADIUS: f32 = 26.0;
pub const OUTER_ROOM_DISTANCE: f32 = 74.0;
pub const BRIDGE_LENGTH: f32 = 26.0;
pub const BRIDGE_HALF_WIDTH: f32 = 3.6;

/// Room shell, matched by both the mesh and the collision.
pub const ROOM_WALL_RADIUS: f32 = OUTER_ROOM_RADIUS - 1.3;
pub const ROOM_WALL_HALF_THICKNESS: f32 = 0.8;
pub const ROOM_DOOR_HALF_ARC: f32 = 0.24;
pub const ROOM_WALL_HEIGHT: f32 = 26.0;

/// Radial offsets of a room's authored contents, measured outward from its centre.
pub const EMBODIMENT_RADIAL_OFFSET: f32 = 7.0;
pub const WORKSHOP_TABLE_RADIAL_OFFSET: f32 = 12.0;

pub const GROUND_Y: f32 = 0.4;
pub const PROMENADE_Y: f32 = 0.5;
pub const BRIDGE_Y: f32 = 0.31;
pub const ABYSS_Y: f32 = -19.8;

/// The six room bearings, shared by the world builder and by collision.
pub const ROOM_ANGLES: [f32; 6] = [
    -FRAC_PI_2,
    -FRAC_PI_6,
    FRAC_PI_6,
    FRAC_PI_2,
    5.0 * FRAC_PI_6,
    7.0 * FRAC_PI_6,
];

// ---------------------------------------------------------------------------------------
// Galleries and stairs
// ---------------------------------------------------------------------------------------

pub const GALLERY_LEVELS: usize = 7;
/// Floor-to-floor. A monumental storey, not a domestic one.
pub const GALLERY_RISE: f32 = 12.0;
pub const FIRST_GALLERY_Y: f32 = 12.0;
pub const GALLERY_INNER_RADIUS: f32 = 102.0;
pub const GALLERY_OUTER_RADIUS: f32 = 114.0;

pub const STAIR_CENTRE_RADIUS: f32 = 108.0;
pub const STAIR_WIDTH: f32 = 8.0;
pub const STAIR_TREAD: f32 = 0.30;
pub const STAIR_RISERS_PER_LEVEL: usize = 72;
/// Two equal flights with a landing between them, then a landing on arrival.
pub const STAIR_FLIGHT_RISERS: usize = 36;
pub const STAIR_LANDING_LENGTH: f32 = 3.0;
/// Each storey's flight begins 60° around the ring from the one below it.
pub const LEVEL_ANGULAR_ADVANCE: f32 = TAU / 6.0;
/// The first flight rises out of the promenade on the Architect's bearing.
pub const STAIR_BASE_BEARING: f32 = -FRAC_PI_2;

/// How much of the circle a player may stand within before falling: the walkable annulus.
pub const PROMENADE_INNER_RADIUS: f32 = OUTER_ROOM_DISTANCE + OUTER_ROOM_RADIUS;

pub fn castle_inner_face() -> f32 {
    CASTLE_RADIUS - CASTLE_WALL_THICKNESS * 0.5
}

// ---------------------------------------------------------------------------------------
// Blender kit placement
// ---------------------------------------------------------------------------------------

/// Bays of arcade around one gallery storey.
///
/// `scripts/author_arcade_bay.py` models the module against this same count and wall radius,
/// and fails its own export check if the cornice run stops matching the chord — so the asset
/// and this placement layer cannot drift apart silently.
pub const ARCADE_BAYS_PER_LEVEL: usize = 72;

/// Chord length of one bay on the inscribed polygon at the wall face. This is the module's
/// repeat distance, and the width the Blender cornice is modelled to.
pub fn arcade_bay_width() -> f32 {
    2.0 * GALLERY_OUTER_RADIUS * (PI / ARCADE_BAYS_PER_LEVEL as f32).sin()
}

pub fn arcade_bay_bearing(index: usize) -> f32 {
    index as f32 * TAU / ARCADE_BAYS_PER_LEVEL as f32
}

/// Every arch in the building: one per bay, on every gallery storey.
///
/// This is the inventory a museum build has to work against — how many walkable chambers the
/// wall could ever hold, and therefore how many exhibits could ever be hung.
pub fn total_arches() -> usize {
    ARCADE_BAYS_PER_LEVEL * GALLERY_LEVELS
}

/// Yaw that seats a wall kit module: its local +X along the wall tangent and its local +Z
/// pointing in toward the hall, with its back face on the wall.
///
/// `Quat::from_rotation_y(phi)` sends local +Z to `(sin phi, 0, cos phi)`, so seating a module
/// against the wall needs `phi = -(bearing + 90°)`. Using `-bearing` — the convention some of
/// the older room furniture uses — leaves modules standing edge-on to the room.
pub fn wall_module_yaw(bearing: f32) -> f32 {
    -(bearing + FRAC_PI_2)
}

/// Where a bay module's origin sits: on the wall face, at the level's deck.
pub fn arcade_bay_position(level: usize, index: usize) -> Vec3 {
    let bearing = arcade_bay_bearing(index);
    Vec3::new(bearing.cos(), 0.0, bearing.sin()) * GALLERY_OUTER_RADIUS
        + Vec3::Y * gallery_y(level)
}

/// Ceiling for free flight: just under the wall head, so the top gallery is reachable and the
/// player cannot leave the building through the vault.
pub fn flight_ceiling() -> f32 {
    CASTLE_WALL_HEIGHT - 2.0
}

pub fn room_centre(angle: f32) -> Vec2 {
    Vec2::new(angle.cos(), angle.sin()) * OUTER_ROOM_DISTANCE
}

pub fn room_centres() -> [Vec2; 6] {
    ROOM_ANGLES.map(room_centre)
}

pub fn gallery_y(level: usize) -> f32 {
    FIRST_GALLERY_Y + level as f32 * GALLERY_RISE
}

/// Height the flight for `level` starts from: the promenade for the first, else the gallery
/// below it.
pub fn flight_base_y(level: usize) -> f32 {
    if level == 0 {
        PROMENADE_Y
    } else {
        gallery_y(level - 1)
    }
}

pub fn flight_top_y(level: usize) -> f32 {
    gallery_y(level)
}

pub fn riser_height(level: usize) -> f32 {
    (flight_top_y(level) - flight_base_y(level)) / STAIR_RISERS_PER_LEVEL as f32
}

/// Going + landings for one storey.
pub fn stair_run_length() -> f32 {
    STAIR_RISERS_PER_LEVEL as f32 * STAIR_TREAD + 2.0 * STAIR_LANDING_LENGTH
}

/// Radians of the circle one storey's stair consumes.
pub fn stair_sweep() -> f32 {
    stair_run_length() / STAIR_CENTRE_RADIUS
}

pub fn stair_start_bearing(level: usize) -> f32 {
    STAIR_BASE_BEARING + level as f32 * LEVEL_ANGULAR_ADVANCE
}

/// Pitch of the flight, in degrees.
pub fn stair_pitch_degrees(level: usize) -> f32 {
    (riser_height(level) / STAIR_TREAD).atan().to_degrees()
}

/// Blondel's comfort rule, `2R + T`, in metres. Comfortable stairs land in 0.60-0.66.
pub fn blondel(level: usize) -> f32 {
    2.0 * riser_height(level) + STAIR_TREAD
}

/// Total distance a player travels from the promenade to the top gallery: every flight plus
/// every connecting stretch of walkway.
pub fn ascent_route_length() -> f32 {
    let walkway_sweep = LEVEL_ANGULAR_ADVANCE - stair_sweep();
    let per_level = stair_run_length() + walkway_sweep * STAIR_CENTRE_RADIUS;
    per_level * GALLERY_LEVELS as f32
}

pub fn stair_inner_radius() -> f32 {
    STAIR_CENTRE_RADIUS - STAIR_WIDTH * 0.5
}

pub fn stair_outer_radius() -> f32 {
    STAIR_CENTRE_RADIUS + STAIR_WIDTH * 0.5
}

/// Walking height along one storey's flight, `run` metres in from its foot.
///
/// Stepped, not ramped: this is the profile of the treads that are actually built, so the
/// surface the player stands on is the surface they can see.
pub fn stair_profile_y(level: usize, run: f32) -> f32 {
    let base = flight_base_y(level);
    let riser = riser_height(level);
    let flight_going = STAIR_FLIGHT_RISERS as f32 * STAIR_TREAD;

    if run < flight_going {
        let tread_index = (run / STAIR_TREAD).floor() as usize;
        return base + (tread_index + 1).min(STAIR_FLIGHT_RISERS) as f32 * riser;
    }
    let after_first = run - flight_going;
    if after_first < STAIR_LANDING_LENGTH {
        return base + STAIR_FLIGHT_RISERS as f32 * riser;
    }
    let second = after_first - STAIR_LANDING_LENGTH;
    if second < flight_going {
        let tread_index = (second / STAIR_TREAD).floor() as usize;
        let risers = STAIR_FLIGHT_RISERS + (tread_index + 1).min(STAIR_FLIGHT_RISERS);
        return base + risers as f32 * riser;
    }
    flight_top_y(level)
}

/// The stair surface beneath a position, if the player is on a flight at roughly this height.
///
/// Seven storeys advancing 60° each wrap past a full circle, so the seventh flight sits
/// directly above the first in plan. Height disambiguates them.
pub fn stair_surface_y(position: Vec2, feet_y: f32) -> Option<f32> {
    let radius = position.length();
    if radius < stair_inner_radius() || radius > stair_outer_radius() {
        return None;
    }
    let bearing = position.y.atan2(position.x);
    let sweep = stair_sweep();
    let mut best: Option<f32> = None;
    for level in 0..GALLERY_LEVELS {
        let delta = (bearing - stair_start_bearing(level)).rem_euclid(TAU);
        if delta > sweep {
            continue;
        }
        let surface = stair_profile_y(level, delta * STAIR_CENTRE_RADIUS);
        if (surface - feet_y).abs() > GALLERY_RISE * 0.75 {
            continue;
        }
        if best.map_or(true, |current| {
            (surface - feet_y).abs() < (current - feet_y).abs()
        }) {
            best = Some(surface);
        }
    }
    best
}

/// The gallery floor beneath a position, if any is near the player's feet.
pub fn gallery_surface_y(position: Vec2, feet_y: f32) -> Option<f32> {
    let radius = position.length();
    if !(GALLERY_INNER_RADIUS..=GALLERY_OUTER_RADIUS).contains(&radius) {
        return None;
    }
    (0..GALLERY_LEVELS)
        .map(gallery_y)
        .filter(|y| (*y - feet_y).abs() <= GALLERY_RISE * 0.75)
        .min_by(|left, right| {
            (left - feet_y)
                .abs()
                .total_cmp(&(right - feet_y).abs())
        })
}

fn bridge_surface_y(position: Vec2) -> Option<f32> {
    for angle in ROOM_ANGLES {
        let radial = Vec2::new(angle.cos(), angle.sin());
        let along = position.dot(radial);
        let across = position.dot(Vec2::new(-radial.y, radial.x)).abs();
        if across <= BRIDGE_HALF_WIDTH
            && (COUNCIL_RADIUS..=COUNCIL_RADIUS + BRIDGE_LENGTH).contains(&along)
        {
            return Some(BRIDGE_Y);
        }
    }
    None
}

/// Picks the surface a player standing here is actually on: the highest candidate they could
/// step up onto, and otherwise the lowest one they would fall to.
fn choose_surface(candidates: &[f32], feet_y: f32) -> Option<f32> {
    let step_up_limit = feet_y + 1.0;
    let steppable = candidates
        .iter()
        .copied()
        .filter(|y| *y <= step_up_limit)
        .fold(f32::NEG_INFINITY, f32::max);
    if steppable.is_finite() {
        return Some(steppable);
    }
    candidates
        .iter()
        .copied()
        .min_by(|left, right| left.total_cmp(right))
}

/// The top surface beneath a player, anywhere in the castle.
///
/// Stepping off a circle, a bridge or a gallery is a real fall into the under-castle floor,
/// not an invisible plane disguised as an abyss.
pub fn castle_surface_y(position: Vec2, feet_y: f32) -> f32 {
    if position.length() <= COUNCIL_RADIUS {
        return GROUND_Y;
    }
    for centre in room_centres() {
        if (position - centre).length() <= OUTER_ROOM_RADIUS {
            return GROUND_Y;
        }
    }
    if let Some(y) = bridge_surface_y(position) {
        return y;
    }

    let radius = position.length();
    let mut candidates: Vec<f32> = Vec::new();
    if (PROMENADE_INNER_RADIUS..=GALLERY_OUTER_RADIUS).contains(&radius) {
        candidates.push(PROMENADE_Y);
    }
    if let Some(y) = gallery_surface_y(position, feet_y) {
        candidates.push(y);
    }
    if let Some(y) = stair_surface_y(position, feet_y) {
        candidates.push(y);
    }
    choose_surface(&candidates, feet_y).unwrap_or(ABYSS_Y)
}

/// Keeps the player inside the enclosing wall. The wall previously had no collision at all —
/// a square clamp at ±100 let a player walk out past a 92m wall entirely.
pub fn clamp_inside_wall(position: Vec2) -> Vec2 {
    let limit = castle_inner_face() - 0.6;
    if position.length() > limit {
        position.normalize() * limit
    } else {
        position
    }
}

/// Collides the player against one outer room's wall, treating it as a shell with thickness.
pub fn resolve_room_walls(position: Vec2) -> Vec2 {
    let inner_face = ROOM_WALL_RADIUS - ROOM_WALL_HALF_THICKNESS;
    let outer_face = ROOM_WALL_RADIUS + ROOM_WALL_HALF_THICKNESS;
    let mut position = position;
    for centre in room_centres() {
        let to_player = position - centre;
        let distance = to_player.length();
        if distance < inner_face || distance > outer_face {
            continue;
        }
        let door_bearing = (-centre.y).atan2(-centre.x);
        let bearing = to_player.y.atan2(to_player.x);
        let mut diff = bearing - door_bearing;
        diff = ((diff + PI).rem_euclid(TAU)) - PI;
        if diff.abs() < ROOM_DOOR_HALF_ARC {
            continue;
        }
        let face = if distance < ROOM_WALL_RADIUS { inner_face } else { outer_face };
        position = centre + (to_player / distance) * face;
    }
    position
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_storey_is_climbed_by_a_stair_a_person_could_actually_use() {
        for level in 0..GALLERY_LEVELS {
            let riser = riser_height(level);
            // Public-stair riser limits sit near 180mm; nothing here may exceed that, and a
            // riser small enough to be a ramp is exactly the defect being replaced.
            assert!(riser <= 0.18, "level {level} riser {riser} is too tall");
            assert!(riser >= 0.14, "level {level} riser {riser} is a ramp, not a stair");

            let comfort = blondel(level);
            assert!(
                (0.60..=0.66).contains(&comfort),
                "level {level} fails Blondel: 2R+T = {comfort}"
            );

            let pitch = stair_pitch_degrees(level);
            assert!(
                (25.0..=33.0).contains(&pitch),
                "level {level} pitch {pitch}° is not a monumental stair"
            );
        }
    }

    #[test]
    fn the_replaced_ascent_was_not_a_stair_at_all() {
        // The old geometry: 1,120 treads, 21.08m of rise, seven circuits of an ~84m radius.
        let old_riser = 21.08 / 1119.0;
        let old_run = 7.0 * TAU * 84.25;
        let old_pitch = (21.08_f32 / old_run).atan().to_degrees();
        assert!(old_riser < 0.02, "sanity: the old riser really was ~19mm");
        assert!(old_pitch < 0.5, "sanity: the old pitch really was under half a degree");
        // Everything built now is an order of magnitude steeper than that.
        assert!(stair_pitch_degrees(1) > old_pitch * 50.0);
    }

    #[test]
    fn one_storey_of_stair_cannot_span_a_whole_circuit_at_this_radius() {
        // This is why "a full circuit per level" and "real stairs" are mutually exclusive
        // here, and why the walkways do the circling instead.
        let sweep_degrees = stair_sweep().to_degrees();
        assert!(
            (14.0..=15.5).contains(&sweep_degrees),
            "a storey's flight sweeps {sweep_degrees}°"
        );
        assert!(sweep_degrees < LEVEL_ANGULAR_ADVANCE.to_degrees());
    }

    #[test]
    fn the_ascent_is_a_real_journey_without_being_a_marathon() {
        let metres = ascent_route_length();
        assert!(
            (700.0..=900.0).contains(&metres),
            "ascent route is {metres}m"
        );
        // At the player's 6.5 m/s that is around two minutes of running.
        let seconds = metres / 6.5;
        assert!((100.0..=140.0).contains(&seconds), "ascent takes {seconds}s");
    }

    #[test]
    fn the_seven_flights_advance_around_the_building_and_overlap_once() {
        let total = LEVEL_ANGULAR_ADVANCE * GALLERY_LEVELS as f32;
        assert!(total > TAU, "the ascent should pass its own starting point");
        assert!(total < TAU * 1.25, "but only just over one circuit");
    }

    #[test]
    fn each_flight_starts_where_the_one_below_it_finished_climbing() {
        for level in 1..GALLERY_LEVELS {
            assert_eq!(flight_base_y(level), flight_top_y(level - 1));
        }
        assert_eq!(flight_base_y(0), PROMENADE_Y);
        assert_eq!(flight_top_y(GALLERY_LEVELS - 1), gallery_y(GALLERY_LEVELS - 1));
    }

    #[test]
    fn a_flight_profile_climbs_from_its_base_to_its_gallery() {
        for level in 0..GALLERY_LEVELS {
            assert!(stair_profile_y(level, 0.0) >= flight_base_y(level));
            assert!(stair_profile_y(level, 0.0) < flight_base_y(level) + 0.2);
            let top = stair_profile_y(level, stair_run_length());
            assert!((top - flight_top_y(level)).abs() < 0.001, "level {level} tops out at {top}");
        }
    }

    #[test]
    fn the_mid_flight_landing_is_level() {
        let flight_going = STAIR_FLIGHT_RISERS as f32 * STAIR_TREAD;
        let a = stair_profile_y(1, flight_going + 0.5);
        let b = stair_profile_y(1, flight_going + STAIR_LANDING_LENGTH - 0.5);
        assert_eq!(a, b, "a landing that climbs is not a landing");
    }

    #[test]
    fn the_first_flight_is_reachable_from_the_ground_promenade() {
        let bearing = stair_start_bearing(0);
        let foot = Vec2::new(bearing.cos(), bearing.sin()) * STAIR_CENTRE_RADIUS;
        let surface = castle_surface_y(foot, PROMENADE_Y);
        assert!(
            (surface - PROMENADE_Y).abs() < 0.25,
            "the first tread sits at {surface}, too far above the promenade to step onto"
        );
    }

    #[test]
    fn the_top_of_a_flight_meets_its_gallery_floor() {
        for level in 0..GALLERY_LEVELS {
            let bearing = stair_start_bearing(level) + stair_sweep();
            let head = Vec2::new(bearing.cos(), bearing.sin()) * STAIR_CENTRE_RADIUS;
            let surface = castle_surface_y(head, flight_top_y(level) - 0.2);
            assert!(
                (surface - gallery_y(level)).abs() < 0.3,
                "level {level} arrives at {surface} but its gallery is at {}",
                gallery_y(level)
            );
        }
    }

    #[test]
    fn galleries_are_wide_enough_to_be_walkways_not_ledges() {
        assert!(GALLERY_OUTER_RADIUS - GALLERY_INNER_RADIUS >= 10.0);
        // And the stair sits within the gallery footprint rather than cantilevering past it.
        assert!(stair_inner_radius() >= GALLERY_INNER_RADIUS);
        assert!(stair_outer_radius() <= GALLERY_OUTER_RADIUS);
    }

    #[test]
    fn the_building_reads_as_tall_rather_than_as_a_pancake() {
        let interior_height = CASTLE_WALL_HEIGHT + VAULT_RISE;
        let interior_diameter = castle_inner_face() * 2.0;
        // The previous shell was 24m tall across 184m — a height-to-width ratio of 0.13, which
        // is why it read as a field with a fence rather than as a hall.
        assert!(interior_height / interior_diameter > 0.5);
        assert!(interior_height > 120.0);
    }

    #[test]
    fn the_arcade_bay_module_matches_the_asset_it_is_modelled_against() {
        // `scripts/author_arcade_bay.py` bakes 9.9452m as the chord its cornice spans. If the
        // wall radius or the bay count changes here without re-authoring the module, every
        // storey opens 72 gaps or overlaps around a 716m circumference.
        let width = arcade_bay_width();
        assert!(
            (width - 9.9452).abs() < 0.001,
            "bay chord is now {width}; re-run scripts/author_arcade_bay.py"
        );
    }

    #[test]
    fn the_building_has_a_known_number_of_arches() {
        // 72 bays on each of 7 gallery storeys. Museum planning is sized against this, so it
        // is stated here rather than recounted by hand each time.
        assert_eq!(total_arches(), 504);
        assert_eq!(ARCADE_BAYS_PER_LEVEL, 72);
        assert_eq!(GALLERY_LEVELS, 7);
    }

    #[test]
    fn the_vault_fresco_module_matches_the_shell_it_caps() {
        // `scripts/author_vault_fresco.py` builds the dome from these two numbers and asserts
        // its own export spans 228.0m and rises 30.0m. If the shell changes without the dome
        // being re-authored, the ceiling either shows a gap at the wall head or buries itself
        // in the masonry.
        assert!((castle_inner_face() - 114.0).abs() < 0.001);
        assert!((VAULT_RISE - 30.0).abs() < 0.001);

        // The dome is a spherical cap cut from this sphere; the script derives the same value.
        let sphere_radius =
            (castle_inner_face().powi(2) + VAULT_RISE.powi(2)) / (2.0 * VAULT_RISE);
        assert!((sphere_radius - 231.6).abs() < 0.01, "sphere radius is {sphere_radius}");
        // Shallow enough to read as a painted saucer dome rather than foreshortening away.
        assert!(VAULT_RISE / castle_inner_face() < 0.35);
    }

    #[test]
    fn bays_close_the_ring_exactly() {
        let total = arcade_bay_width() * ARCADE_BAYS_PER_LEVEL as f32;
        let polygon = 2.0 * GALLERY_OUTER_RADIUS * (PI / ARCADE_BAYS_PER_LEVEL as f32).sin()
            * ARCADE_BAYS_PER_LEVEL as f32;
        assert!((total - polygon).abs() < 0.001);
        // The inscribed polygon is a little shorter than the true circle, as it must be.
        assert!(total < TAU * GALLERY_OUTER_RADIUS);
        assert!(total > TAU * GALLERY_OUTER_RADIUS * 0.999);
    }

    #[test]
    fn a_wall_module_faces_into_the_hall() {
        for index in [0usize, 7, 18, 51] {
            let bearing = arcade_bay_bearing(index);
            let yaw = wall_module_yaw(bearing);
            let rotation = Quat::from_rotation_y(yaw);
            let inward = rotation * Vec3::Z;
            let expected = -Vec3::new(bearing.cos(), 0.0, bearing.sin());
            assert!(
                (inward - expected).length() < 0.001,
                "bay {index} local +Z points {inward:?}, expected {expected:?}"
            );
            // And its length runs along the wall, not across it.
            let along = rotation * Vec3::X;
            let tangent = Vec3::new(-bearing.sin(), 0.0, bearing.cos());
            assert!((along - tangent).length() < 0.001);
        }
    }

    #[test]
    fn every_bay_is_seated_on_the_wall_face_at_its_own_deck() {
        for level in 0..GALLERY_LEVELS {
            for index in [0usize, 30, 71] {
                let position = arcade_bay_position(level, index);
                let radius = Vec2::new(position.x, position.z).length();
                assert!((radius - GALLERY_OUTER_RADIUS).abs() < 0.001);
                assert!((position.y - gallery_y(level)).abs() < 0.001);
            }
        }
    }

    #[test]
    fn the_outer_wall_actually_stops_the_player() {
        let outside = Vec2::new(0.0, 200.0);
        let clamped = clamp_inside_wall(outside);
        assert!(clamped.length() < castle_inner_face());
        let inside = Vec2::new(10.0, 10.0);
        assert_eq!(clamp_inside_wall(inside), inside);
    }

    #[test]
    fn standing_on_a_gallery_does_not_drop_the_player_to_the_floor_below() {
        let bearing: f32 = 2.0;
        let spot = Vec2::new(bearing.cos(), bearing.sin()) * STAIR_CENTRE_RADIUS;
        for level in 0..GALLERY_LEVELS {
            let y = gallery_y(level);
            let surface = castle_surface_y(spot, y);
            assert!(
                (surface - y).abs() < 0.3,
                "level {level}: standing at {y} resolved to {surface}"
            );
        }
    }

    #[test]
    fn the_council_rooms_and_bridges_still_have_ground_under_them() {
        assert_eq!(castle_surface_y(Vec2::ZERO, GROUND_Y), GROUND_Y);
        for centre in room_centres() {
            assert_eq!(castle_surface_y(centre, GROUND_Y), GROUND_Y);
        }
        let bridge = Vec2::new(0.0, -(COUNCIL_RADIUS + BRIDGE_LENGTH * 0.5));
        assert_eq!(castle_surface_y(bridge, GROUND_Y), BRIDGE_Y);
    }

    #[test]
    fn stepping_off_a_bridge_into_the_gap_is_a_real_fall() {
        // Midway between two room bearings, out past the council circle: open air, no bridge,
        // no room, no promenade.
        let bearing = -FRAC_PI_2 + LEVEL_ANGULAR_ADVANCE * 0.5;
        let gap = Vec2::new(bearing.cos(), bearing.sin()) * 36.0;
        assert_eq!(castle_surface_y(gap, GROUND_Y), ABYSS_Y);
    }

    #[test]
    fn rooms_sit_between_the_council_circle_and_the_promenade() {
        assert!(OUTER_ROOM_DISTANCE - OUTER_ROOM_RADIUS > COUNCIL_RADIUS);
        assert!(OUTER_ROOM_DISTANCE + OUTER_ROOM_RADIUS <= PROMENADE_INNER_RADIUS);
        assert!(PROMENADE_INNER_RADIUS < GALLERY_INNER_RADIUS);
    }

    #[test]
    fn a_player_can_stand_at_the_workshop_bench_behind_the_room_centre() {
        // The old ring collision ejected anything inside the ring radius outward, which
        // teleported a walking player away from the bench and made the back half of every
        // room reachable only by flying.
        let centre = room_centre(ROOM_ANGLES[0]);
        let radial = centre.normalize();
        let bench = centre + radial * WORKSHOP_TABLE_RADIAL_OFFSET;
        let resolved = resolve_room_walls(bench);
        assert!((resolved - bench).length() < 0.001, "the bench position was moved to {resolved:?}");
        assert!((resolved - centre).length() < ROOM_WALL_RADIUS);
    }

    #[test]
    fn a_room_wall_keeps_an_inside_player_inside_and_an_outside_player_outside() {
        let centre = room_centre(ROOM_ANGLES[0]);
        let inner_face = ROOM_WALL_RADIUS - ROOM_WALL_HALF_THICKNESS;
        let outer_face = ROOM_WALL_RADIUS + ROOM_WALL_HALF_THICKNESS;
        // Due east of the room centre, well away from the doorway arc.
        let pushed_in = resolve_room_walls(centre + Vec2::new(ROOM_WALL_RADIUS - 0.2, 0.0));
        let pushed_out = resolve_room_walls(centre + Vec2::new(ROOM_WALL_RADIUS + 0.3, 0.0));
        assert!(((pushed_in - centre).length() - inner_face).abs() < 0.01);
        assert!(((pushed_out - centre).length() - outer_face).abs() < 0.01);
    }

    #[test]
    fn the_room_doorway_arc_stays_open() {
        let centre = room_centre(ROOM_ANGLES[0]);
        let toward_hub = -centre.normalize();
        let in_the_doorway = centre + toward_hub * ROOM_WALL_RADIUS;
        assert!((resolve_room_walls(in_the_doorway) - in_the_doorway).length() < 0.001);
    }

    #[test]
    fn a_player_out_on_the_bridge_is_untouched_by_the_room_wall() {
        let on_the_bridge = Vec2::new(0.0, -(COUNCIL_RADIUS + BRIDGE_LENGTH * 0.5));
        assert!((resolve_room_walls(on_the_bridge) - on_the_bridge).length() < 0.001);
    }

    #[test]
    fn a_bridge_reaches_from_the_council_circle_to_its_room() {
        let reach = COUNCIL_RADIUS + BRIDGE_LENGTH;
        assert!(reach >= OUTER_ROOM_DISTANCE - OUTER_ROOM_RADIUS);
    }
}
