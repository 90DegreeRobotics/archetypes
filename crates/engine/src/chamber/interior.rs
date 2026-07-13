//! Archetype world encapsulation.
//!
//! Translates the focused archetype's design tokens (`ArchetypeTheme`) into an
//! actual rendered environment. Before this system the theme registry was dead
//! code: fully specified palettes that were never applied to anything the player
//! sees. Crossing into an archetype now inverts the *law of the world* — the
//! ceremonial dark void gives way to that archetype's void and ambient light —
//! rather than merely swapping a caption.
//!
//! The mechanism is general (driven by `CurrentFocus`), so any archetype the
//! chamber can focus is honoured; only the Architect is reachable in the current
//! vertical slice.

use bevy::prelude::*;

use super::{ChamberState, CurrentFocus};

/// Near-black ceremonial void — the resting law of the chamber.
const CEREMONIAL_VOID: Color = Color::BLACK;
/// Cool, dim ambient so the authored Blender lights carry the resting scene.
const CEREMONIAL_AMBIENT: Color = Color::srgb(0.60, 0.65, 0.82);
const CEREMONIAL_BRIGHTNESS: f32 = 90.0;

/// How aggressively the archetype floods its world with ambient light.
const INTERIOR_BRIGHTNESS: f32 = 1500.0;

/// Per-second lerp response for the environment transition. Kept smooth so the
/// crossing feels inevitable rather than a hard cut.
const ENV_RESPONSE: f32 = 1.6;

pub struct InteriorPlugin;

impl Plugin for InteriorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(CEREMONIAL_VOID))
            .insert_resource(GlobalAmbientLight {
                color: CEREMONIAL_AMBIENT,
                brightness: CEREMONIAL_BRIGHTNESS,
                ..default()
            })
            .add_systems(Update, drive_interior_environment);
    }
}

/// Lerp the world void and ambient light toward the focused archetype's tokens
/// while inside its world, and back to the ceremonial void otherwise.
fn drive_interior_environment(
    state: Res<State<ChamberState>>,
    focus: Res<CurrentFocus>,
    time: Res<Time>,
    mut clear: ResMut<ClearColor>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    // While a council member holds the floor, the world takes on that archetype's
    // environment; at all other times it rests in the ceremonial void.
    let inside_world = matches!(state.get(), ChamberState::CouncilSpeaking);

    let (ambient_color, brightness) = match (inside_world, focus.0) {
        (true, Some(archetype)) => {
            let theme = archetype.theme();
            let light = theme.accent_secondary.unwrap_or(theme.bg_void);
            (light, INTERIOR_BRIGHTNESS)
        }
        _ => (CEREMONIAL_AMBIENT, CEREMONIAL_BRIGHTNESS),
    };

    let t = (time.delta_secs() * ENV_RESPONSE).min(1.0);
    clear.0 = CEREMONIAL_VOID;
    ambient.color = lerp_color(ambient.color, ambient_color, t);
    ambient.brightness += (brightness - ambient.brightness) * t;
}

/// Frame-rate independent color lerp in linear space.
fn lerp_color(from: Color, to: Color, t: f32) -> Color {
    let a = from.to_linear();
    let b = to.to_linear();
    Color::LinearRgba(LinearRgba {
        red: a.red + (b.red - a.red) * t,
        green: a.green + (b.green - a.green) * t,
        blue: a.blue + (b.blue - a.blue) * t,
        alpha: a.alpha + (b.alpha - a.alpha) * t,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_color_reaches_endpoints() {
        let black = Color::srgb(0.0, 0.0, 0.0);
        let white = Color::srgb(1.0, 1.0, 1.0);

        let start = lerp_color(black, white, 0.0).to_linear();
        assert!(start.red < 0.001);

        let end = lerp_color(black, white, 1.0).to_linear();
        assert!(end.red > 0.999);
    }

    #[test]
    fn the_render_void_is_absolute_black() {
        let void = CEREMONIAL_VOID.to_linear();
        assert!(void.red == 0.0 && void.green == 0.0 && void.blue == 0.0);
    }
}
