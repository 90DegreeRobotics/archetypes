//! Shared, testable gamepad-to-action helpers.
//!
//! Bevy 0.18 represents each connected controller as its own `Gamepad` component, so a
//! player with one controller still has a query that could yield zero, one, or (if they
//! plug in a second pad) more than one. Every helper here folds across all connected pads
//! rather than assuming a single fixed "gamepad 0", so a mid-session reconnect or a second
//! controller never silently goes dead.

use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::prelude::*;

/// Stick deflection at or below this magnitude is physical rest-state drift, not intent.
/// Matches the Xbox One S controller's typical center-drift band.
pub const DEFAULT_DEADZONE: f32 = 0.18;

/// Rescales a raw stick vector so anything inside the deadzone is exactly zero and the
/// remaining travel is remapped back onto the full [0.0, 1.0] magnitude range, instead of
/// leaving a dead jump at the deadzone boundary (push-past-deadzone snapping to speed).
pub fn apply_radial_deadzone(raw: Vec2, deadzone: f32) -> Vec2 {
    let deadzone = deadzone.clamp(0.0, 0.9);
    let magnitude = raw.length();
    if magnitude <= deadzone {
        return Vec2::ZERO;
    }
    let rescaled = ((magnitude - deadzone) / (1.0 - deadzone)).min(1.0);
    raw.normalize() * rescaled
}

/// True if any connected pad registers `button` as pressed this frame.
pub fn any_just_pressed(gamepads: &Query<&Gamepad>, button: GamepadButton) -> bool {
    gamepads.iter().any(|pad| pad.just_pressed(button))
}

/// True if any connected pad currently holds `button` down.
pub fn any_pressed(gamepads: &Query<&Gamepad>, button: GamepadButton) -> bool {
    gamepads.iter().any(|pad| pad.pressed(button))
}

/// Deadzone-corrected left stick, summed across every connected pad.
pub fn combined_left_stick(gamepads: &Query<&Gamepad>, deadzone: f32) -> Vec2 {
    gamepads
        .iter()
        .map(|pad| apply_radial_deadzone(pad.left_stick(), deadzone))
        .fold(Vec2::ZERO, |sum, stick| sum + stick)
}

/// Deadzone-corrected right stick, summed across every connected pad.
pub fn combined_right_stick(gamepads: &Query<&Gamepad>, deadzone: f32) -> Vec2 {
    gamepads
        .iter()
        .map(|pad| apply_radial_deadzone(pad.right_stick(), deadzone))
        .fold(Vec2::ZERO, |sum, stick| sum + stick)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadzone_zeroes_small_center_drift() {
        assert_eq!(apply_radial_deadzone(Vec2::new(0.05, 0.02), DEFAULT_DEADZONE), Vec2::ZERO);
    }

    #[test]
    fn full_deflection_still_reaches_unit_magnitude_after_rescale() {
        let result = apply_radial_deadzone(Vec2::new(1.0, 0.0), DEFAULT_DEADZONE);
        assert!((result.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn just_past_deadzone_starts_near_zero_not_full_speed() {
        let result = apply_radial_deadzone(Vec2::new(DEFAULT_DEADZONE + 0.01, 0.0), DEFAULT_DEADZONE);
        assert!(result.length() < 0.1);
    }

    #[test]
    fn diagonal_stick_noise_never_exceeds_unit_magnitude() {
        // Some pads report slightly out-of-range raw diagonal values; the helper must clamp.
        let result = apply_radial_deadzone(Vec2::new(1.3, 1.3), DEFAULT_DEADZONE);
        assert!(result.length() <= 1.0001);
    }

    #[test]
    fn zero_deadzone_passes_input_through_unscaled_at_full_deflection() {
        let result = apply_radial_deadzone(Vec2::new(0.5, 0.0), 0.0);
        assert!((result.x - 0.5).abs() < 1e-5);
    }
}
