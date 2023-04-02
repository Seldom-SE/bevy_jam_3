use bevy::{math::Vec3Swizzles, prelude::*};

use crate::map::{get_object_z, ChunkManager, ChunkQuery};

#[derive(Component, Default)]
pub struct Vel(pub Vec2);

pub struct TilePhysics {
    pub friction: f32,
}

fn entity_terrain(
    mut collider_query: Query<(&mut Transform, &Vel)>,
    chunk_query: ChunkQuery,
    chunks: Res<ChunkManager>,
    time: Res<Time>,
) {
    for (mut transform, vel) in collider_query.iter_mut() {
        let x = transform.translation.x + vel.0.x * time.delta_seconds();
        if chunks
            .get_wall_tile(Vec2::new(x, transform.translation.y), &chunk_query)
            .is_none()
        {
            transform.translation.x = x;
        }
        let y = transform.translation.y + vel.0.y * time.delta_seconds();
        if chunks
            .get_wall_tile(Vec2::new(transform.translation.x, y), &chunk_query)
            .is_none()
        {
            transform.translation.y = y;
        }
        transform.translation.z = get_object_z(transform.translation.y);
    }
}

pub fn physics_plugin(app: &mut App) {
    app.add_system(entity_terrain);
}
