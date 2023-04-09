use bevy::prelude::*;

use crate::map::{get_object_z, ChunkManager, ChunkQuery};

#[derive(Component, Default)]
pub struct Vel(pub Vec2);

#[derive(Component)]
pub struct DespawnOnCollide;

pub struct TilePhysics {
    pub friction: f32,
}

fn entity_terrain(
    mut commands: Commands,
    mut collider_query: Query<(Entity, &mut Transform, &Vel, Option<&DespawnOnCollide>)>,
    chunk_query: ChunkQuery,
    chunks: Res<ChunkManager>,
    time: Res<Time>,
) {
    for (entity, mut transform, vel, despawn_on_collide) in collider_query.iter_mut() {
        let x = transform.translation.x + vel.0.x * time.delta_seconds();
        let mut despawn = false;
        if chunks
            .get_wall_tile(Vec2::new(x, transform.translation.y), &chunk_query)
            .is_none()
        {
            transform.translation.x = x;
        } else {
            despawn = true;
        }
        let y = transform.translation.y + vel.0.y * time.delta_seconds();
        if chunks
            .get_wall_tile(Vec2::new(transform.translation.x, y), &chunk_query)
            .is_none()
        {
            transform.translation.y = y;
        } else {
            despawn = true;
        }
        transform.translation.z = get_object_z(transform.translation.y);

        if despawn && despawn_on_collide.is_some() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn physics_plugin(app: &mut App) {
    app.add_system(entity_terrain);
}
