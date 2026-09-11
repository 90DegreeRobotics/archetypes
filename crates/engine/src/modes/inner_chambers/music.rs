//! Inner Castle music lifecycle.
//!
//! The currently supplied tracks are MP3-derived WAV runtime assets with no verified
//! loop points.  They therefore play once; this plugin must not create a click-prone
//! hard loop merely to claim that the castle has "ambient music".

use bevy::prelude::*;
use bevy::audio::Volume;

use super::world::InnerWorldElement;
use super::InnerChambersState;

pub const INNER_CASTLE_BASE_TRACK: &str = "audio/music/inner_castle_base_loop.wav";
pub const SEVEN_HEAVENS_TRACK: &str = "audio/music/seven_heavens_overlook_loop.wav";

#[derive(Component)]
pub struct InnerCastleMusic;

pub struct InnerCastleMusicPlugin;

impl Plugin for InnerCastleMusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Navigating), start_inner_castle_music);
    }
}

fn start_inner_castle_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(INNER_CASTLE_BASE_TRACK)),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(0.22)),
        InnerCastleMusic,
        InnerWorldElement,
        Name::new("InnerCastleBaseMusic_OneShotUntilLoopVerified"),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_tracks_have_stable_installed_asset_paths() {
        assert!(INNER_CASTLE_BASE_TRACK.ends_with("inner_castle_base_loop.wav"));
        assert!(SEVEN_HEAVENS_TRACK.ends_with("seven_heavens_overlook_loop.wav"));
    }
}
