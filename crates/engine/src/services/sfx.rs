//! The sound-effects bus.
//!
//! The settings menu has shipped a "Sound effects volume" slider labelled `(no SFX bus yet)`,
//! because there was no bus: nothing in the castle played a sound effect at all. This is the
//! bus, and that qualifier comes off.
//!
//! Every cue is authored by `scripts/author_sound_effects.py` from noise, decaying partials and
//! filtered transients — no sample pack, no licence to check, nothing that has to be torn out
//! later. See `memory: build-it-dont-wait-for-a-clean-license`.
//!
//! Gain is `master * sfx` from `GameSettings`, read at the moment of playing. A cue already in
//! flight keeps the level it started at, which is correct: a footstep is 160ms and re-levelling
//! it mid-flight would be a click, not a courtesy.

use bevy::audio::{PlaybackSettings, Volume};
use bevy::prelude::*;

use super::settings::GameSettings;

/// Every cue the game can play. Named for what happens, not for what it sounds like, so a cue
/// can be re-authored without renaming its callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sfx {
    Footstep,
    Jump,
    Land,
    PickUp,
    Place,
    Duplicate,
    Swing,
    AltarCharge,
    ManifestSuccess,
    ManifestFailure,
    MenuMove,
    MenuAdjust,
    MenuConfirm,
}

/// Cues with more than one authored variant, so a repeated action does not sound like one
/// sample looping. Footsteps are the only one that repeats fast enough to need it.
const FOOTSTEP_VARIANTS: usize = 4;

impl Sfx {
    /// Asset path for this cue, choosing a variant where there is more than one.
    ///
    /// `variant` is any counter the caller already has — a step index, a frame count. It is
    /// taken modulo the variant count, so callers need not know how many exist.
    pub fn asset_path(self, variant: usize) -> String {
        let name = match self {
            Sfx::Footstep => {
                return format!(
                    "audio/sfx/footstep_stone_{}.wav",
                    variant % FOOTSTEP_VARIANTS + 1
                )
            }
            Sfx::Jump => "jump",
            Sfx::Land => "land",
            Sfx::PickUp => "pick_up",
            Sfx::Place => "place",
            Sfx::Duplicate => "duplicate",
            Sfx::Swing => "swing",
            Sfx::AltarCharge => "altar_charge",
            Sfx::ManifestSuccess => "manifest_success",
            Sfx::ManifestFailure => "manifest_failure",
            Sfx::MenuMove => "menu_move",
            Sfx::MenuAdjust => "menu_adjust",
            Sfx::MenuConfirm => "menu_confirm",
        };
        format!("audio/sfx/{name}.wav")
    }

    /// Per-cue trim, so the bus mixes rather than merely plays.
    ///
    /// The authored files already sit at sensible relative levels; this is the last 20% — a
    /// menu tick should sit under a manifestation flourish even when both are "on".
    pub fn trim(self) -> f32 {
        match self {
            Sfx::Footstep => 0.55,
            Sfx::Jump => 0.7,
            Sfx::Land => 0.8,
            Sfx::PickUp | Sfx::Place => 0.85,
            Sfx::Duplicate => 0.8,
            Sfx::Swing => 0.65,
            Sfx::AltarCharge => 0.7,
            Sfx::ManifestSuccess => 1.0,
            Sfx::ManifestFailure => 0.9,
            Sfx::MenuMove | Sfx::MenuAdjust => 0.6,
            Sfx::MenuConfirm => 0.75,
        }
    }
}

/// Gain a cue plays at: the player's master and SFX sliders, times the cue's own trim.
pub fn sfx_gain(settings: &GameSettings, cue: Sfx) -> f32 {
    settings.volume_master * settings.volume_sfx * cue.trim()
}

/// Marks a playing one-shot, so they can all be cleared when a mode tears down.
#[derive(Component)]
pub struct SfxVoice;

/// How many cues have actually been played, by kind.
///
/// A sound that does not fire is indistinguishable from one that fires silently, and neither
/// shows up in a screenshot. The capture harness writes this into its report, so "the footsteps
/// work" is a number rather than a claim.
#[derive(Resource, Default, Debug, Clone)]
pub struct SfxTally {
    pub played: usize,
    pub footsteps: usize,
    pub suppressed_silent: usize,
}

/// Queue a cue. Written as an event rather than a direct spawn so gameplay systems do not each
/// need `Commands`, `AssetServer` and `GameSettings` just to make a noise.
#[derive(Message)]
pub struct PlaySfx {
    pub cue: Sfx,
    pub variant: usize,
}

impl PlaySfx {
    pub fn new(cue: Sfx) -> Self {
        Self { cue, variant: 0 }
    }

    pub fn variant(cue: Sfx, variant: usize) -> Self {
        Self { cue, variant }
    }
}

pub struct SfxPlugin;

impl Plugin for SfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySfx>()
            .init_resource::<SfxTally>()
            .add_systems(Update, play_queued_sfx);
    }
}

fn play_queued_sfx(
    mut commands: Commands,
    mut requests: MessageReader<PlaySfx>,
    asset_server: Res<AssetServer>,
    settings: Res<GameSettings>,
    mut tally: ResMut<SfxTally>,
) {
    for request in requests.read() {
        let gain = sfx_gain(&settings, request.cue);
        if gain <= 0.0 {
            // Silence is silence: do not spawn a voice that will be inaudible, or a muted game
            // still pays for every footstep.
            tally.suppressed_silent += 1;
            continue;
        }
        tally.played += 1;
        if request.cue == Sfx::Footstep {
            tally.footsteps += 1;
        }
        commands.spawn((
            AudioPlayer::new(asset_server.load(request.cue.asset_path(request.variant))),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
            SfxVoice,
            Name::new("SfxVoice"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVERY_CUE: [Sfx; 13] = [
        Sfx::Footstep,
        Sfx::Jump,
        Sfx::Land,
        Sfx::PickUp,
        Sfx::Place,
        Sfx::Duplicate,
        Sfx::Swing,
        Sfx::AltarCharge,
        Sfx::ManifestSuccess,
        Sfx::ManifestFailure,
        Sfx::MenuMove,
        Sfx::MenuAdjust,
        Sfx::MenuConfirm,
    ];

    /// Every cue must have a file on disk. A missing one is silent, and silence is
    /// indistinguishable from a bus that does not work.
    #[test]
    fn every_cue_has_an_authored_file() {
        let root = std::path::Path::new("../../assets");
        for cue in EVERY_CUE {
            let variants = if cue == Sfx::Footstep {
                FOOTSTEP_VARIANTS
            } else {
                1
            };
            for variant in 0..variants {
                let path = root.join(cue.asset_path(variant));
                assert!(
                    path.is_file(),
                    "missing {}; run `python scripts/author_sound_effects.py`",
                    path.display()
                );
            }
        }
    }

    /// Footsteps must actually cycle, or a run is one sample repeating.
    #[test]
    fn footsteps_cycle_through_every_variant() {
        let paths: std::collections::HashSet<String> = (0..FOOTSTEP_VARIANTS * 3)
            .map(|index| Sfx::Footstep.asset_path(index))
            .collect();
        assert_eq!(paths.len(), FOOTSTEP_VARIANTS);
    }

    /// The slider has to reach the bus, both ends of it.
    #[test]
    fn gain_follows_the_master_and_sfx_sliders() {
        let full = GameSettings {
            volume_master: 1.0,
            volume_sfx: 1.0,
            ..GameSettings::default()
        };
        assert_eq!(sfx_gain(&full, Sfx::ManifestSuccess), Sfx::ManifestSuccess.trim());

        let muted_master = GameSettings {
            volume_master: 0.0,
            ..full.clone()
        };
        assert_eq!(sfx_gain(&muted_master, Sfx::Footstep), 0.0);

        let muted_sfx = GameSettings {
            volume_sfx: 0.0,
            ..full.clone()
        };
        assert_eq!(sfx_gain(&muted_sfx, Sfx::Footstep), 0.0);

        let half = GameSettings {
            volume_master: 0.5,
            volume_sfx: 0.5,
            ..full
        };
        assert_eq!(sfx_gain(&half, Sfx::Land), 0.25 * Sfx::Land.trim());
    }

    /// A menu tick must not be as loud as a manifestation arriving.
    #[test]
    fn the_bus_mixes_rather_than_merely_playing() {
        assert!(Sfx::MenuMove.trim() < Sfx::ManifestSuccess.trim());
        assert!(Sfx::Footstep.trim() < Sfx::Land.trim());
        for cue in EVERY_CUE {
            assert!(cue.trim() > 0.0 && cue.trim() <= 1.0, "{cue:?}");
        }
    }
}
