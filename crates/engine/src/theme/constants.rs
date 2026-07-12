use super::{Archetype, ArchetypeMotion, ArchetypeTheme};
use bevy::prelude::*;
use std::time::Duration;

// Helper to define srgb_u8 cleanly
const fn hex_color(r: u8, g: u8, b: u8) -> Color {
    Color::srgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

const fn rgba_color(r: u8, g: u8, b: u8, a: f32) -> Color {
    Color::srgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a)
}

pub fn theme_architect() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Architect,
        name: "Luminous Blueprint",
        bg_void: hex_color(242, 244, 248), // #f2f4f8
        bg_elevated: rgba_color(255, 255, 255, 0.65),
        text_primary: hex_color(11, 14, 20),     // #0b0e14
        text_secondary: hex_color(58, 63, 75),   // #3a3f4b
        accent_primary: hex_color(59, 130, 246), // #3b82f6 (blueprint cyan)
        accent_secondary: Some(hex_color(147, 197, 253)), // #93c5fd
        border_soft: rgba_color(0, 0, 0, 0.08),
        border_hard: rgba_color(0, 0, 0, 0.2),
        glow_primary: rgba_color(59, 130, 246, 0.4),
        glow_intensity: 0.6,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(120),
            normal: Duration::from_millis(240),
            slow: Duration::from_millis(420),
            easing: (0.25, 0.1, 0.25, 1.0), // Smooth mathematically eased
        },
        harmonic_signature: "C Major Chord",
    }
}

pub fn theme_sentinel() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Sentinel,
        name: "Null Aegis",
        bg_void: hex_color(7, 9, 13),                   // #07090d
        bg_elevated: hex_color(13, 17, 23),             // #0d1117
        text_primary: hex_color(230, 232, 235),         // #e6e8eb
        text_secondary: hex_color(154, 163, 173),       // #9aa3ad
        accent_primary: hex_color(122, 162, 247),       // #7aa2f7 (lawful blue)
        accent_secondary: Some(hex_color(255, 95, 86)), // #ff5f56 (critical)
        border_soft: rgba_color(255, 255, 255, 0.06),
        border_hard: rgba_color(255, 255, 255, 0.18),
        glow_primary: rgba_color(122, 162, 247, 0.25),
        glow_intensity: 0.4,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(90),
            normal: Duration::from_millis(180),
            slow: Duration::from_millis(360),
            easing: (0.0, 0.0, 1.0, 1.0), // Linear, Rectilinear, no curves
        },
        harmonic_signature: "F# Tritone Tension",
    }
}

pub fn theme_jester() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Jester,
        name: "Mulligan Engine",
        bg_void: hex_color(245, 245, 244), // #f5f5f4
        bg_elevated: hex_color(230, 230, 230),
        text_primary: hex_color(28, 25, 23),            // #1c1917
        text_secondary: hex_color(68, 64, 60),          // #44403c
        accent_primary: hex_color(67, 56, 202),         // #4338ca (bruise)
        accent_secondary: Some(hex_color(185, 28, 28)), // #b91c1c (blood)
        border_soft: rgba_color(0, 0, 0, 0.1),
        border_hard: hex_color(67, 56, 202),
        glow_primary: rgba_color(185, 28, 28, 0.4),
        glow_intensity: 0.8,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(50),
            normal: Duration::from_millis(200),
            slow: Duration::from_millis(600),
            easing: (0.68, -0.55, 0.265, 1.55), // Asymmetric, unpredictable
        },
        harmonic_signature: "Dissonant Glitch",
    }
}

pub fn theme_mentor() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Mentor,
        name: "Ancient Resonance",
        bg_void: hex_color(0, 43, 38), // #002b26
        bg_elevated: rgba_color(0, 150, 136, 0.15),
        text_primary: hex_color(224, 242, 241),   // #e0f2f1
        text_secondary: hex_color(128, 203, 196), // #80cbc4
        accent_primary: hex_color(0, 150, 136),   // #009688 (Emerald)
        accent_secondary: None,
        border_soft: rgba_color(255, 255, 255, 0.1),
        border_hard: rgba_color(0, 150, 136, 0.4),
        glow_primary: rgba_color(0, 150, 136, 0.4),
        glow_intensity: 0.5,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(240),
            normal: Duration::from_millis(480),
            slow: Duration::from_millis(960),
            easing: (0.25, 0.46, 0.45, 0.94), // Slow unfolding
        },
        harmonic_signature: "Resonant Hum (Tibetan Bowl)",
    }
}

pub fn theme_explorer() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Explorer,
        name: "Frontier Flare",
        bg_void: hex_color(13, 13, 13),                  // #0d0d0d
        bg_elevated: hex_color(26, 26, 26),              // #1a1a1a
        text_primary: hex_color(255, 204, 128),          // #ffcc80
        text_secondary: hex_color(255, 136, 0),          // #ff8800
        accent_primary: hex_color(255, 136, 0),          // #ff8800 (Vanta Orange)
        accent_secondary: Some(hex_color(255, 213, 79)), // #ffd54f
        border_soft: rgba_color(255, 136, 0, 0.2),
        border_hard: rgba_color(255, 136, 0, 0.4),
        glow_primary: rgba_color(255, 136, 0, 0.6),
        glow_intensity: 0.8,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(150),
            normal: Duration::from_millis(300),
            slow: Duration::from_millis(600),
            easing: (0.175, 0.885, 0.32, 1.275), // Buoyant cubic-bounce
        },
        harmonic_signature: "Rising E Major Arpeggio",
    }
}

pub fn theme_oracle() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Oracle,
        name: "Noctis Veil",
        bg_void: hex_color(15, 11, 26), // #0f0b1a
        bg_elevated: rgba_color(92, 77, 125, 0.1),
        text_primary: hex_color(209, 196, 233),   // #d1c4e9
        text_secondary: hex_color(149, 117, 205), // #9575cd
        accent_primary: hex_color(92, 77, 125),   // #5c4d7d (Noctis Violet)
        accent_secondary: None,
        border_soft: rgba_color(149, 117, 205, 0.2),
        border_hard: rgba_color(149, 117, 205, 0.4),
        glow_primary: rgba_color(92, 77, 125, 0.5),
        glow_intensity: 0.8,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(300),
            normal: Duration::from_millis(600),
            slow: Duration::from_millis(1200),
            easing: (0.4, 0.0, 0.2, 1.0), // Ethereal
        },
        harmonic_signature: "B Minor Pad + Distant Bells",
    }
}

pub fn theme_empath() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Empath,
        name: "Luma Resonance",
        bg_void: hex_color(26, 15, 15), // #1a0f0f
        bg_elevated: rgba_color(247, 202, 201, 0.1),
        text_primary: hex_color(255, 235, 238),   // #ffebee
        text_secondary: hex_color(240, 98, 146),  // #f06292
        accent_primary: hex_color(247, 202, 201), // #f7cac1 (Luma Rose Quartz)
        accent_secondary: None,
        border_soft: rgba_color(247, 202, 201, 0.3),
        border_hard: rgba_color(247, 202, 201, 0.6),
        glow_primary: rgba_color(247, 202, 201, 0.4),
        glow_intensity: 0.6,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(400),
            normal: Duration::from_millis(1600),
            slow: Duration::from_millis(3200), // Rhythmic breath
            easing: (0.42, 0.0, 0.58, 1.0),    // Smooth sine-like breath
        },
        harmonic_signature: "Warm D Major + Ambient Choral",
    }
}

pub fn theme_codex() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Codex,
        name: "Midnight Manuscript",
        bg_void: hex_color(26, 29, 63), // #1a1d3f
        bg_elevated: rgba_color(255, 255, 255, 0.05),
        text_primary: hex_color(245, 245, 220), // Pale parchment
        text_secondary: rgba_color(245, 245, 220, 0.7),
        accent_primary: hex_color(212, 175, 55), // Soft gold
        accent_secondary: None,
        border_soft: rgba_color(212, 175, 55, 0.2),
        border_hard: rgba_color(212, 175, 55, 0.5),
        glow_primary: rgba_color(212, 175, 55, 0.3),
        glow_intensity: 0.4,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(200),
            normal: Duration::from_millis(400),
            slow: Duration::from_millis(800),
            easing: (0.25, 0.1, 0.25, 1.0), // Deliberate inscription
        },
        harmonic_signature: "Inscriptive Scratch",
    }
}

pub fn theme_viren() -> ArchetypeTheme {
    ArchetypeTheme {
        archetype: Archetype::Viren,
        name: "Viren Flamebearer",
        bg_void: hex_color(11, 11, 14),         // #0B0B0E Carbon Void
        bg_elevated: hex_color(22, 22, 26),     // #16161A Ash Black
        text_primary: hex_color(237, 237, 237), // #EDEDED Bone Light
        text_secondary: hex_color(154, 154, 154), // #9A9A9A Ash Text
        accent_primary: hex_color(255, 159, 28),
        accent_secondary: Some(hex_color(255, 209, 102)), // #FFD166 Burnished Gold
        border_soft: rgba_color(255, 159, 28, 0.35),
        border_hard: hex_color(109, 90, 142), // #6D5A8E Afterimage Violet
        glow_primary: rgba_color(0, 0, 0, 0.0), // NEVER any ambient glow
        glow_intensity: 0.0,
        motion: ArchetypeMotion {
            fast: Duration::from_millis(140),   // Ignition
            normal: Duration::from_millis(220), // Containment settle
            slow: Duration::from_millis(480),   // Respectful fade
            easing: (0.65, 0.05, 0.36, 1.0),
        },
        harmonic_signature: "Single struck E-flat + low ember hiss",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viren_theme_obeys_ember_covenant_timing_and_silence_law() {
        let theme = theme_viren();

        assert_eq!(theme.motion.fast, Duration::from_millis(140));
        assert_eq!(theme.motion.normal, Duration::from_millis(220));
        assert_eq!(theme.motion.slow, Duration::from_millis(480));
        assert_eq!(theme.glow_intensity, 0.0);
        assert_eq!(
            theme.harmonic_signature,
            "Single struck E-flat + low ember hiss"
        );
    }
}
