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
    /// Title / loading screen. Nothing of the chamber is shown until the authored
    /// scene has loaded.
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

/// Canonical world-space center of the raised council constellation. The authored
/// `Council_Assembly`, runtime solid star, and deliberation camera all share it.
pub(super) const COUNCIL_CENTER: Vec3 = Vec3::new(0.0, 2.0, 0.0);

#[derive(Resource, Default, Debug, Clone)]
pub struct CurrentFocus(pub Option<Archetype>);

pub struct CouncilChamberPlugin;

impl Plugin for CouncilChamberPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<ChamberState>()
            .init_resource::<CurrentFocus>()
            .add_systems(Startup, load_authoritative_chamber)
            .add_plugins((
                boot::BootPlugin,
                camera::CameraPlugin,
                spheres::SpheresPlugin,
                panels::PanelsPlugin,
                portal::PortalPlugin,
                interior::InteriorPlugin,
                sky::SkyPlugin,
                star::StarPlugin,
                council::CouncilPlugin,
                ritual::RitualPlugin,
                speech::SpeechPlugin,
            ));
    }
}

fn load_authoritative_chamber(mut commands: Commands, asset_server: Res<AssetServer>) {
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
