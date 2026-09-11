//! Archetypes engine entry — council chamber runtime.

// Release Desktop launches must not flash a console behind the game window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::winit::{UpdateMode, WinitSettings};
use std::time::Duration;

pub mod chamber;
pub mod modes;
pub mod services;
pub mod theme;

use chamber::CouncilChamberPlugin;
use modes::ModesPlugin;
use services::settings::SettingsPlugin;

const PRODUCT_VERSION: &str = env!("ARCHETYPES_PRODUCT_VERSION");
const BUILD_SERIAL: &str = env!("ARCHETYPES_BUILD_SERIAL");

fn main() {
    App::new()
        .insert_resource(foreground_frame_settings(Duration::from_secs_f64(
            1.0 / 60.0,
        )))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: window_title(),
                        name: Some("archetypes.council.chamber".to_owned()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, maximize_primary_window)
        .add_plugins((SettingsPlugin, ModesPlugin, CouncilChamberPlugin))
        .run();
}

fn foreground_frame_settings(interval: Duration) -> WinitSettings {
    WinitSettings {
        focused_mode: UpdateMode::reactive_low_power(interval),
        unfocused_mode: UpdateMode::reactive_low_power(Duration::from_secs_f64(
            1.0 / 15.0,
        )),
    }
}

fn window_title() -> String {
    format!("Archetypes {PRODUCT_VERSION} (build {BUILD_SERIAL}) — Council Chamber")
}

fn maximize_primary_window(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = windows.single_mut() {
        window.set_maximized(true);
    }
}

fn asset_root() -> String {
    if cfg!(debug_assertions) {
        format!("{}/../../assets", env!("CARGO_MANIFEST_DIR"))
    } else {
        "assets".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_asset_root_points_at_workspace_assets() {
        assert!(asset_root().replace('\\', "/").ends_with("/../../assets"));
    }

    #[test]
    fn native_window_title_always_carries_release_identity() {
        assert_eq!(
            window_title(),
            format!("Archetypes {PRODUCT_VERSION} (build {BUILD_SERIAL}) — Council Chamber")
        );
    }

    #[test]
    fn chamber_has_a_bounded_foreground_frame_rate() {
        let interval = Duration::from_secs_f64(1.0 / 60.0);
        assert_eq!(
            foreground_frame_settings(interval).focused_mode,
            UpdateMode::reactive_low_power(interval)
        );
    }
}
