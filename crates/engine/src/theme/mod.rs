pub mod constants;

use bevy::prelude::*;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Archetype {
    #[default]
    Architect,
    Sentinel,
    Jester,
    Mentor,
    Explorer,
    Oracle,
    Empath,
    Codex,
    Viren,
}

impl Archetype {
    pub fn theme(&self) -> ArchetypeTheme {
        match self {
            Archetype::Architect => constants::theme_architect(),
            Archetype::Sentinel => constants::theme_sentinel(),
            Archetype::Jester => constants::theme_jester(),
            Archetype::Mentor => constants::theme_mentor(),
            Archetype::Explorer => constants::theme_explorer(),
            Archetype::Oracle => constants::theme_oracle(),
            Archetype::Empath => constants::theme_empath(),
            Archetype::Codex => constants::theme_codex(),
            Archetype::Viren => constants::theme_viren(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArchetypeMotion {
    pub fast: Duration,
    pub normal: Duration,
    pub slow: Duration,
    // (x1, y1, x2, y2) for cubic-bezier
    pub easing: (f32, f32, f32, f32),
}

#[derive(Debug, Clone, Resource)]
pub struct ArchetypeTheme {
    pub archetype: Archetype,
    pub name: &'static str,

    pub bg_void: Color,
    pub bg_elevated: Color,

    pub text_primary: Color,
    pub text_secondary: Color,

    pub accent_primary: Color,
    pub accent_secondary: Option<Color>,

    pub border_soft: Color,
    pub border_hard: Color,

    pub glow_primary: Color,
    pub glow_intensity: f32,

    pub motion: ArchetypeMotion,

    pub harmonic_signature: &'static str,
}
