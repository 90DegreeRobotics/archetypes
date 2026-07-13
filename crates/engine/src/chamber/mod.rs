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
        // Feet land near the authored chamber floor; the portal surface sits above it.
        Transform::from_xyz(0.0, -1.1, 0.0).with_scale(Vec3::splat(5.0)),
        Name::new("PortalTable"),
    ));
}
