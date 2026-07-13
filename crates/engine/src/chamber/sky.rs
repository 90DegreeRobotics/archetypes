//! Starfield sky + reflections.
//!
//! The ceremonial chamber used to render against an absolute-black void, which
//! left the glass vessels and the gilded table with nothing to catch — every shiny
//! surface read flat and dead, and the operator reported the black sky "broke the
//! whole look." This module replaces the black background with a procedural
//! starfield: a [`Skybox`] makes the stars visible behind the open temple, and an
//! [`EnvironmentMapLight`] built from the same cubemap lets the gold and glass
//! actually reflect starlight. The cubemap is generated in code, so no external
//! asset is introduced and the build stays self-contained.

use bevy::asset::RenderAssetUsages;
use bevy::core_pipeline::Skybox;
use bevy::light::EnvironmentMapLight;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use super::camera::WitnessCamera;

/// Visible brightness of the starfield background (cd/m², scaled by camera exposure).
const SKYBOX_BRIGHTNESS: f32 = 850.0;
/// Strength of the starlight reflected onto gold/glass surfaces.
const ENV_INTENSITY: f32 = 430.0;
/// Per-cube-face resolution of the generated star map.
const FACE: usize = 512;
/// Restrained stars scattered on each face. The council assets remain the subject;
/// the sky is sparse depth punctuation, not visual snow.
const STARS_PER_FACE: usize = 180;

pub struct SkyPlugin;

/// Handle to the generated star cubemap, shared by the skybox and the env map.
#[derive(Resource)]
struct StarSky(Handle<Image>);

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_star_sky)
            .add_systems(Update, attach_sky_to_camera);
    }
}

fn build_star_sky(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let handle = images.add(star_cubemap());
    commands.insert_resource(StarSky(handle));
}

/// Attach the skybox + reflective environment map to the Witness camera the frame
/// it appears. The camera is spawned once at Startup and never respawned, so the
/// `Added` filter makes this run exactly once.
fn attach_sky_to_camera(
    mut commands: Commands,
    sky: Option<Res<StarSky>>,
    camera: Query<Entity, Added<WitnessCamera>>,
) {
    let Some(sky) = sky else { return };
    for entity in &camera {
        commands.entity(entity).insert((
            Skybox {
                image: sky.0.clone(),
                brightness: SKYBOX_BRIGHTNESS,
                rotation: Quat::IDENTITY,
            },
            EnvironmentMapLight {
                diffuse_map: sky.0.clone(),
                specular_map: sky.0.clone(),
                intensity: ENV_INTENSITY,
                rotation: Quat::IDENTITY,
                affects_lightmapped_mesh_diffuse: true,
            },
        ));
    }
}

/// Build a deep-space star cubemap in code: a dark navy base with scattered stars,
/// a minority tinted warm or cool and bloomed into a small cluster. Deterministic
/// (fixed seed) so the sky is identical every run — consistent with the chamber's
/// "nothing drifts on its own" discipline.
fn star_cubemap() -> Image {
    let layers = 6usize;
    let mut data = vec![0u8; FACE * FACE * layers * 4];

    // Deep-space navy base (sRGB8).
    let base = [5u8, 7, 16, 255];
    for texel in data.chunks_exact_mut(4) {
        texel.copy_from_slice(&base);
    }

    // Small deterministic xorshift PRNG — no external rand dependency.
    let mut seed: u32 = 0x9E37_79B9;
    let mut rng = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        seed
    };

    for layer in 0..layers {
        for _ in 0..STARS_PER_FACE {
            let x = rng() as usize % FACE;
            let y = rng() as usize % FACE;
            let roll = rng();
            let brightness = 105u16 + (roll % 151) as u16; // 105..=255
            let (r, g, b) = match roll % 12 {
                0 => (brightness, brightness * 82 / 100, brightness * 60 / 100), // warm gold
                1 => (brightness * 70 / 100, brightness * 82 / 100, brightness), // cool blue
                _ => (brightness, brightness, brightness),                       // white
            };
            put(&mut data, layer, x, y, r as u8, g as u8, b as u8);
            // One texel only: no square two-pixel clusters at gameplay resolution.
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: FACE as u32,
            height: FACE as u32,
            depth_or_array_layers: layers as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    // Present the six array layers as a cubemap for the skybox and env map.
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });
    image
}

fn put(data: &mut [u8], layer: usize, x: usize, y: usize, r: u8, g: u8, b: u8) {
    let idx = (layer * FACE * FACE + y * FACE + x) * 4;
    data[idx] = r;
    data[idx + 1] = g;
    data[idx + 2] = b;
    data[idx + 3] = 255;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn star_cubemap_has_six_square_faces() {
        let image = star_cubemap();
        let size = image.texture_descriptor.size;
        assert_eq!(size.width, FACE as u32);
        assert_eq!(size.height, FACE as u32);
        assert_eq!(size.depth_or_array_layers, 6);
        assert_eq!(
            image
                .texture_view_descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.dimension),
            Some(TextureViewDimension::Cube)
        );
    }

    #[test]
    fn star_cubemap_is_mostly_dark_with_some_bright_stars() {
        let image = star_cubemap();
        let data = image.data.as_ref().expect("cubemap has cpu data");
        let bright = data
            .chunks_exact(4)
            .filter(|texel| texel[0] > 120 || texel[1] > 120 || texel[2] > 120)
            .count();
        let total = data.len() / 4;
        // Stars are sparse: a clear majority of texels are the dark navy base.
        assert!(bright > 0, "expected some stars");
        assert!(
            bright * 200 < total,
            "expected stars to be sparse, got {bright} of {total}"
        );
    }
}
