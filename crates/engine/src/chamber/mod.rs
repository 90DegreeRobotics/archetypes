use bevy::prelude::*;

pub mod boot;
pub mod camera;
pub mod council;
pub mod interior;
pub mod panels;
pub mod portal;
pub mod ritual;
pub mod sky;
pub mod speech;
pub mod spheres;
pub mod star;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChamberState {
    /// Title / loading screen. The black veil owns the first frames before the
    /// current chamber-backed main menu is revealed.
    #[default]
    Booting,
    MainMenu,
    Onboarding,
    IdleAtTable,
    Deliberating,
    CouncilSpeaking,
    WitnessVerdict,
    ArtifactPending,
    ArtifactResult,
}

use crate::theme::Archetype;

/// Canonical world-space center of the raised council constellation — the single
/// source of truth for its height. The authored `Council_Assembly` node (baked into
/// `uiscene1.glb` at [`AUTHORED_COUNCIL_Y`]) is re-pinned to this at runtime by
/// `spheres::raise_council_constellation`; the runtime solid star and the
/// deliberation camera share it too. Raise this one value to lift the whole star
/// tetrahedron — vessels, portrait panels, and crystal core — as a group above the
/// table. At y = 6.0 the vessel tips span ≈ 2.5..9.5, clearing the table top (≈ −1.8).
pub(super) const COUNCIL_CENTER: Vec3 = Vec3::new(0.0, 6.0, 0.0);

/// The y at which `Council_Assembly` and the seven `*_PanelSpinner` roots were baked
/// into `uiscene1.glb`. The runtime lift applied to the constellation is
/// `COUNCIL_CENTER.y - AUTHORED_COUNCIL_Y`.
pub(super) const AUTHORED_COUNCIL_Y: f32 = 2.0;

#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentFocus(pub Option<Archetype>);

#[derive(Resource, Debug, Clone)]
pub struct ActiveGameMode(pub crate::modes::game_mode::GameMode);

pub struct CouncilChamberPlugin;

impl Plugin for CouncilChamberPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChamberState>()
            .init_resource::<CurrentFocus>();

        if legacy_chamber_visuals_enabled() {
            app.add_systems(Startup, load_authoritative_chamber)
                .add_plugins((
                    spheres::SpheresPlugin,
                    panels::PanelsPlugin,
                    portal::PortalPlugin,
                    sky::SkyPlugin,
                    star::StarPlugin,
                ));
        } else {
            app.add_systems(Startup, load_lore_chamber);
        }

        app.add_plugins((
            boot::BootPlugin,
            camera::CameraPlugin,
            interior::InteriorPlugin,
            council::CouncilPlugin,
            ritual::RitualPlugin,
            speech::SpeechPlugin,
        ));
    }
}

fn legacy_chamber_visuals_enabled() -> bool {
    std::env::var_os("ARCHETYPES_LEGACY_CHAMBER").is_some()
}

const LORE_CHAMBER_SCENE: &str = "scenes/lore_chamber.glb#Scene0";

fn load_lore_chamber(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Hidden until Booting ends so the first frames are pure black title veil,
    // never a flash of the chamber-backed main menu world.
    commands.spawn((
        SceneRoot(asset_server.load(LORE_CHAMBER_SCENE)),
        Name::new("LoreCouncilChamber"),
        Visibility::Hidden,
    ));
}

fn load_authoritative_chamber(mut commands: Commands, asset_server: Res<AssetServer>) {
    spawn_authoritative_chamber(&mut commands, &asset_server);
}

pub(crate) fn spawn_authoritative_chamber(commands: &mut Commands, asset_server: &AssetServer) {
    commands.spawn((
        SceneRoot(asset_server.load("scenes/uiscene1.glb#Scene0")),
        Name::new("AuthoritativeCouncilChamber"),
    ));
    // The portal table sits on the chamber floor, centred beneath the star. The user
    // places intent into its stargate portal; the camera then sweeps up to the star.
    commands.spawn((
        SceneRoot(asset_server.load("scenes/table.glb#Scene0")),
        // Furniture-scale (×2.6), not a monument. Feet authored at local z = -0.784;
        // chamber floor top is world y = -5.03, so y offset -2.99 (= -5.03 + 0.784×2.6)
        // lands the feet exactly on the floor. At ×2.6 the top radius is ~2.47, inside
        // the council vessels' inner edge (they ring the centre at radius ~4, reaching
        // in to ~2.9), so the table no longer intersects the vessels.
        Transform::from_xyz(0.0, -2.99, 0.0).with_scale(Vec3::splat(2.6)),
        Name::new("PortalTable"),
    ));
}

#[cfg(test)]
mod tests {
    #[test]
    fn lore_chamber_asset_exists_in_workspace_assets() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("assets")
            .join("scenes")
            .join("lore_chamber.glb");
        assert!(path.is_file(), "missing lore chamber asset at {path:?}");
    }
}
