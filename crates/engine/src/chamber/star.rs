//! The central star — a solid glowing crystal, not a wireframe.
//!
//! The authored `Star_Tetra_A/B` geometry baked a Wireframe modifier into thin
//! emissive edges that read as a cheap 1980s wireframe effect. This module retires
//! that look entirely: it force-hides the authored star nodes and spawns a single
//! **solid** stellated-octahedron (star tetrahedron) built in code, centred on the
//! council and glowing softly. It stays fixed — only the camera moves — and only
//! appears once the ritual has left the table for deliberation.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

use super::{spheres::ArchetypeSphere, ChamberState, COUNCIL_CENTER};

pub struct StarPlugin;

/// Marker for the engine-authored solid star.
#[derive(Component)]
struct SolidStar;

/// Fraction of the vessel-shell radius the solid star's points reach toward. Below
/// 1.0 so the star reads as a crystal core with the vessels floating around it.
const STAR_FILL: f32 = 0.8;

impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (hide_wireframe_star, spawn_solid_star, gate_solid_star).chain(),
        );
    }
}

/// Permanently hide the authored wireframe star in every state. The scene loads
/// asynchronously, so this runs each frame and simply keeps the nodes hidden.
fn hide_wireframe_star(mut named: Query<(&Name, &mut Visibility)>) {
    for (name, mut visibility) in &mut named {
        let name = name.as_str();
        if (name.contains("Star_Tetra") || name.contains("Merkaba"))
            && *visibility != Visibility::Hidden
        {
            *visibility = Visibility::Hidden;
        }
    }
}

/// Spawn the solid star once the seven vessels are bound, centred on their centroid
/// and sized from their mean radius so its points reach toward the vessel shell.
fn spawn_solid_star(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing: Query<(), With<SolidStar>>,
    spheres: Query<&GlobalTransform, With<ArchetypeSphere>>,
) {
    if !existing.is_empty() {
        return;
    }
    let positions: Vec<Vec3> = spheres.iter().map(|t| t.translation()).collect();
    if positions.len() < 7 {
        return;
    }
    // Do not derive placement from an asynchronous GLTF transform on the spawn
    // frame. The authored assembly and camera share this explicit raised center.
    let center = COUNCIL_CENTER;
    let mean_radius =
        positions.iter().map(|p| p.distance(center)).sum::<f32>() / positions.len() as f32;
    let half = (mean_radius * STAR_FILL) / 3.0_f32.sqrt();

    let material = materials.add(StandardMaterial {
        // A deep sapphire crystal that catches the surrounding starfield (via the
        // environment map) with a faint inner glow. Kept dark with low roughness and
        // real metalness so its facets read by reflection and directional light —
        // not flooded flat by ambient. Winding is outward, so back-face culling is
        // correct and each facet shades by its true orientation.
        base_color: Color::srgb(0.02, 0.07, 0.20),
        emissive: LinearRgba::new(0.015, 0.07, 0.16, 1.0),
        metallic: 0.55,
        perceptual_roughness: 0.08,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(stellated_octahedron(half))),
        MeshMaterial3d(material),
        // Tilt off the symmetry axes so the camera sees a foreshortened 3/4 view of
        // a real 3D crystal, not the flat head-on star silhouette.
        Transform::from_translation(center).with_rotation(Quat::from_euler(
            EulerRot::XYZ,
            0.62,
            0.5,
            0.15,
        )),
        Visibility::Hidden,
        SolidStar,
        Name::new("SolidStar"),
    ));
}

/// Reveal the solid star only once the ritual leaves the table for deliberation —
/// matching the authored choreography (table hidden → star revealed on submission).
fn gate_solid_star(
    state: Res<State<ChamberState>>,
    mut star: Query<&mut Visibility, With<SolidStar>>,
) {
    let visible = matches!(
        state.get(),
        ChamberState::Deliberating
            | ChamberState::CouncilSpeaking
            | ChamberState::WitnessVerdict
            | ChamberState::ArtifactPending
    );
    for mut visibility in &mut star {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Build a solid stellated octahedron (two interpenetrating tetrahedra — the star
/// tetrahedron) with flat-shaded facets, inscribed in a cube of the given half-size.
fn stellated_octahedron(half: f32) -> Mesh {
    let s = half;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x * s, y * s, z * s);
    // Two tetrahedra on alternating cube corners.
    let tetra_a = [
        v(1.0, 1.0, 1.0),
        v(1.0, -1.0, -1.0),
        v(-1.0, 1.0, -1.0),
        v(-1.0, -1.0, 1.0),
    ];
    let tetra_b = [
        v(-1.0, -1.0, -1.0),
        v(-1.0, 1.0, 1.0),
        v(1.0, -1.0, 1.0),
        v(1.0, 1.0, -1.0),
    ];

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(24);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(24);
    // Each tetrahedron is centred on the origin, so a face's outward direction is
    // simply its centroid direction. Emit vertices wound so the geometric normal
    // points outward — then default back-face culling keeps each facet's true
    // orientation and the solid shades as real 3D form.
    let mut push_tri = |a: Vec3, b: Vec3, c: Vec3| {
        let mut b = b;
        let mut c = c;
        let mut normal = (b - a).cross(c - a);
        let outward = (a + b + c) / 3.0;
        if normal.dot(outward) < 0.0 {
            std::mem::swap(&mut b, &mut c);
            normal = (b - a).cross(c - a);
        }
        let normal = normal.normalize_or_zero().to_array();
        for point in [a, b, c] {
            positions.push(point.to_array());
            normals.push(normal);
        }
    };
    for tetra in [tetra_a, tetra_b] {
        let [p0, p1, p2, p3] = tetra;
        push_tri(p0, p1, p2);
        push_tri(p0, p2, p3);
        push_tri(p0, p3, p1);
        push_tri(p1, p3, p2);
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stellated_octahedron_has_eight_triangular_facets() {
        let mesh = stellated_octahedron(2.0);
        let count = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|values| values.as_float3())
            .map(|values| values.len())
            .unwrap_or(0);
        // Two tetrahedra × 4 faces × 3 vertices, flat-shaded (non-indexed).
        assert_eq!(count, 24);
    }
}
