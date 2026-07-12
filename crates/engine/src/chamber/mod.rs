use bevy::prelude::*;

pub mod boot;
pub mod camera;
pub mod council;
pub mod interior;
pub mod portal;
pub mod ritual;
pub mod speech;
pub mod spheres;

#[derive(States, Default, Debug, Clone, Eq, PartialEq, Hash)]
pub enum ChamberState {
    /// Title / loading screen. Nothing of the chamber is shown until the authored
    /// scene has loaded.
    #[default]
    Booting,
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
                portal::PortalPlugin,
                interior::InteriorPlugin,
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
}
