//! Skinned character avatars.
//!
//! A character GLB authored by `scripts/rig_character_avatar.py` carries one skin and named
//! animation clips. Spawning it through [`avatar_scene`] and observing
//! [`start_avatar_animation`] finds the scene's `AnimationPlayer` once the instance exists and
//! loops the requested clip on it.

use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;

pub const JESTER_RIGGED: &str = "scenes/nebula_jester_rigged.glb";
pub const IDLE_CLIP: &str = "Idle";

/// Which clip a spawned avatar scene should loop once it is ready.
#[derive(Component, Clone)]
pub struct AvatarAnimation {
    pub gltf: Handle<Gltf>,
    pub clip: &'static str,
}

pub fn avatar_scene(
    asset_server: &AssetServer,
    path: &'static str,
    clip: &'static str,
) -> (SceneRoot, AvatarAnimation) {
    (
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
        AvatarAnimation {
            gltf: asset_server.load(path),
            clip,
        },
    )
}

pub fn start_avatar_animation(
    ready: On<SceneInstanceReady>,
    mut commands: Commands,
    avatars: Query<&AvatarAnimation>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let root = ready.entity;
    let Ok(avatar) = avatars.get(root) else {
        return;
    };
    let Some(clip) = gltfs
        .get(&avatar.gltf)
        .and_then(|gltf| gltf.named_animations.get(avatar.clip))
    else {
        warn!("avatar: clip {:?} is not in the loaded glTF", avatar.clip);
        return;
    };
    let (graph, node) = AnimationGraph::from_clip(clip.clone());
    let graph = graphs.add(graph);
    let mut started = false;
    for entity in children.iter_descendants(root) {
        if let Ok(mut player) = players.get_mut(entity) {
            player.play(node).repeat();
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(graph.clone()));
            started = true;
        }
    }
    if !started {
        warn!("avatar: scene has no AnimationPlayer; is the GLB skinned?");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn glb_json(path: &str) -> serde_json::Value {
        let bytes = std::fs::read(format!("{}/../../assets/{path}", env!("CARGO_MANIFEST_DIR")))
            .expect("rigged avatar GLB is shipped");
        let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        serde_json::from_slice(&bytes[20..20 + len]).expect("GLB JSON chunk")
    }

    #[test]
    fn the_rigged_jester_ships_a_skin_and_the_clip_the_game_plays() {
        let json = glb_json(JESTER_RIGGED);
        let joints = json["skins"][0]["joints"].as_array().map_or(0, Vec::len);
        assert!(joints >= 15, "the Jester skin has {joints} joints");
        let names: Vec<&str> = json["animations"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|clip| clip["name"].as_str())
            .collect();
        assert!(names.contains(&IDLE_CLIP), "clips shipped: {names:?}");
    }
}
