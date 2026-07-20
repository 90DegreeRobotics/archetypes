//! Archetypes engine entry — council chamber runtime.

// Release Desktop launches must not flash a console behind the game window.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub mod chamber;
pub mod modes;
pub mod services;
pub mod theme;

use chamber::CouncilChamberPlugin;
use modes::ModesPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Archetypes — Council Chamber".to_owned(),
                        name: Some("archetypes.council.chamber".to_owned()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_systems(Startup, maximize_primary_window)
        .add_plugins((ModesPlugin, CouncilChamberPlugin))
        .run();
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
}
