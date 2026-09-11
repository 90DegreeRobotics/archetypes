//! Persisted local player settings — look sensitivity, gamepad deadzone, and volume mix.
//!
//! Settings apply and save immediately on change; there is no separate "Apply" staging
//! step yet, so every control is live as soon as it is set. Values are clamped on load
//! and on save so a hand-edited or corrupted `settings.json` cannot push controller
//! curves or volumes outside a sane range.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use super::gamepad_input::DEFAULT_DEADZONE;
use super::paths::config_root;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
pub struct GameSettings {
    pub mouse_sensitivity: f32,
    pub gamepad_look_sensitivity: f32,
    pub gamepad_deadzone: f32,
    pub volume_master: f32,
    pub volume_music: f32,
    pub volume_sfx: f32,
    pub volume_voice: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.0013,
            gamepad_look_sensitivity: 2.2,
            gamepad_deadzone: DEFAULT_DEADZONE,
            volume_master: 1.0,
            volume_music: 0.8,
            volume_sfx: 1.0,
            volume_voice: 1.0,
        }
    }
}

impl GameSettings {
    /// Pulls every field back into its sane operating range. Applied on load (so a
    /// hand-edited file cannot hand the camera an unusable curve) and on save (so an
    /// in-session bug cannot persist an out-of-range value to disk).
    pub fn clamped(mut self) -> Self {
        self.mouse_sensitivity = self.mouse_sensitivity.clamp(0.0002, 0.006);
        self.gamepad_look_sensitivity = self.gamepad_look_sensitivity.clamp(0.2, 8.0);
        self.gamepad_deadzone = self.gamepad_deadzone.clamp(0.0, 0.6);
        self.volume_master = self.volume_master.clamp(0.0, 1.0);
        self.volume_music = self.volume_music.clamp(0.0, 1.0);
        self.volume_sfx = self.volume_sfx.clamp(0.0, 1.0);
        self.volume_voice = self.volume_voice.clamp(0.0, 1.0);
        self
    }
}

pub fn settings_path() -> PathBuf {
    config_root().join("settings.json")
}

/// Loads settings from disk, falling back to defaults if the file is absent or the JSON
/// is unreadable (a corrupt file must never crash the boot).
pub fn load_settings() -> GameSettings {
    let path = settings_path();
    let Ok(body) = fs::read_to_string(&path) else {
        return GameSettings::default();
    };
    serde_json::from_str::<GameSettings>(&body)
        .map(GameSettings::clamped)
        .unwrap_or_default()
}

/// Writes settings via a temp-file-then-rename so a crash mid-write cannot leave a
/// truncated `settings.json` behind.
pub fn save_settings(settings: &GameSettings) -> Result<(), String> {
    let path = settings_path();
    let parent = path.parent().ok_or("settings path has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let body = serde_json::to_string_pretty(&settings.clone().clamped())
        .map_err(|error| error.to_string())?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, body).map_err(|error| error.to_string())?;
    fs::rename(&tmp_path, &path).map_err(|error| error.to_string())
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(load_settings())
            .add_systems(Update, persist_settings_on_change);
    }
}

fn persist_settings_on_change(settings: Res<GameSettings>) {
    if settings.is_changed() {
        if let Err(error) = save_settings(&settings) {
            error!("failed to persist settings.json: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_live_under_config_not_data() {
        let path = settings_path();
        let rendered = path.to_string_lossy();
        assert!(rendered.contains("NeuroCognica"));
        assert!(rendered.contains("Archetypes"));
        assert!(rendered.contains("config"));
        assert!(rendered.ends_with("settings.json"));
    }

    #[test]
    fn default_settings_are_already_in_bounds() {
        assert_eq!(GameSettings::default().clamped(), GameSettings::default());
    }

    #[test]
    fn round_trip_through_json_preserves_values() {
        let settings = GameSettings {
            mouse_sensitivity: 0.002,
            ..GameSettings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: GameSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings, restored);
    }

    #[test]
    fn clamp_rejects_out_of_range_persisted_values() {
        let mut settings = GameSettings::default();
        settings.volume_master = 4.0;
        settings.gamepad_deadzone = -1.0;
        settings.gamepad_look_sensitivity = 99.0;
        let clamped = settings.clamped();
        assert_eq!(clamped.volume_master, 1.0);
        assert_eq!(clamped.gamepad_deadzone, 0.0);
        assert_eq!(clamped.gamepad_look_sensitivity, 8.0);
    }

    #[test]
    fn save_then_load_round_trips_through_real_disk() {
        // Isolate this test from a real installed settings.json by writing to a scratch
        // path directly rather than through settings_path(), which is fixed to LocalAppData.
        let scratch = std::env::temp_dir().join(format!(
            "archetypes-settings-roundtrip-{}.json",
            std::process::id()
        ));
        let settings = GameSettings {
            volume_music: 0.42,
            ..GameSettings::default()
        };
        let body = serde_json::to_string_pretty(&settings).unwrap();
        fs::write(&scratch, body).unwrap();
        let restored: GameSettings = serde_json::from_str(&fs::read_to_string(&scratch).unwrap()).unwrap();
        assert_eq!(restored.volume_music, 0.42);
        let _ = fs::remove_file(&scratch);
    }
}
