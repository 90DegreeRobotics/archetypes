use bevy::prelude::*;

pub struct MerkabaPlugin;

impl Plugin for MerkabaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, animate_merkaba);
    }
}
fn animate_merkaba(time: Res<Time>, mut query: Query<(&Name, &mut Transform)>) {
    for (name, mut transform) in &mut query {
        let speed = match name.as_str() {
            "Merkaba_Diamond" => 0.16,
            "Merkaba_Emerald" => -0.11,
            _ => continue,
        };
        transform.rotate_y(speed * time.delta_secs());
    }
}
