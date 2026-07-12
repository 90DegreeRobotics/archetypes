use bevy::prelude::*;

pub mod chamber;
pub mod theme;

use chamber::CouncilChamberPlugin;

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
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(CouncilChamberPlugin)
        .run();
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
