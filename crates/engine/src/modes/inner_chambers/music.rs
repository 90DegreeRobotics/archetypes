//! Inner Castle music lifecycle.
//!
//! The currently supplied tracks are MP3-derived WAV runtime assets with no verified
//! loop points.  They therefore play once; this plugin must not create a click-prone
//! hard loop merely to claim that the castle has "ambient music".

use bevy::prelude::*;
use bevy::audio::Volume;

use crate::services::settings::GameSettings;

use super::world::InnerWorldElement;
use super::InnerChambersState;

pub const INNER_CASTLE_BASE_TRACK: &str = "audio/music/inner_castle_base_loop.wav";
pub const SEVEN_HEAVENS_TRACK: &str = "audio/music/seven_heavens_overlook_loop.wav";

#[derive(Component)]
pub struct InnerCastleMusic;

/// The level the track was mixed to sit at when every slider is full. The settings menu's
/// music slider scales this rather than replacing it, so "100%" means the intended mix, not a
/// track played at full scale over everything else.
pub const MUSIC_REFERENCE_GAIN: f32 = 0.22;

/// Music gain from the player's settings: master multiplied by music, against the reference
/// mix. Both at 100% gives exactly the level the track was authored for.
pub fn music_gain(settings: &GameSettings) -> f32 {
    MUSIC_REFERENCE_GAIN * settings.volume_master * settings.volume_music
}

pub struct InnerCastleMusicPlugin;

impl Plugin for InnerCastleMusicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(InnerChambersState::Navigating), start_inner_castle_music)
            .add_systems(
                Update,
                apply_music_volume.run_if(in_state(InnerChambersState::Navigating)),
            );
    }
}

fn start_inner_castle_music(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
) {
    commands.spawn((
        AudioPlayer::new(asset_server.load(INNER_CASTLE_BASE_TRACK)),
        PlaybackSettings::ONCE.with_volume(Volume::Linear(music_gain(&settings))),
        InnerCastleMusic,
        InnerWorldElement,
        Name::new("InnerCastleBaseMusic_OneShotUntilLoopVerified"),
    ));
}

/// Moving the slider has to change what the player hears *now*, not on the next launch.
/// Without this the menu would show a music slider that only takes effect after a restart,
/// which is indistinguishable from a slider that does nothing.
fn apply_music_volume(
    settings: Res<GameSettings>,
    mut sinks: Query<&mut AudioSink, With<InnerCastleMusic>>,
) {
    if !settings.is_changed() {
        return;
    }
    for mut sink in &mut sinks {
        sink.set_volume(Volume::Linear(music_gain(&settings)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supplied_tracks_have_stable_installed_asset_paths() {
        assert!(INNER_CASTLE_BASE_TRACK.ends_with("inner_castle_base_loop.wav"));
        assert!(SEVEN_HEAVENS_TRACK.ends_with("seven_heavens_overlook_loop.wav"));
    }

    /// The music slider has to actually reach the mix. It was a hard-coded 0.22 that ignored
    /// the persisted setting entirely, so a settings menu showing a music slider would have
    /// been a control that changed nothing.
    #[test]
    fn music_gain_follows_both_the_master_and_the_music_slider() {
        let full = GameSettings::default();
        let full = GameSettings {
            volume_master: 1.0,
            volume_music: 1.0,
            ..full
        };
        assert_eq!(music_gain(&full), MUSIC_REFERENCE_GAIN);

        let silent_master = GameSettings {
            volume_master: 0.0,
            ..full.clone()
        };
        assert_eq!(music_gain(&silent_master), 0.0);

        let silent_music = GameSettings {
            volume_music: 0.0,
            ..full.clone()
        };
        assert_eq!(music_gain(&silent_music), 0.0);

        let half = GameSettings {
            volume_master: 0.5,
            volume_music: 0.5,
            ..full
        };
        assert_eq!(music_gain(&half), MUSIC_REFERENCE_GAIN * 0.25);
    }
}
