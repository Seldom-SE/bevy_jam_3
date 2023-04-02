use bevy::{math::Vec3Swizzles, utils::HashMap};

use crate::{prelude::*, SCREEN_SIZE};

pub fn map_plugin(app: &mut App) {
    app.init_resource::<ChunkManager>()
        .add_system(spawn_chunks_around_camera)
        .add_system(despawn_outofrange_chunks);
}

// TODO Make this between 2 and 3
pub fn get_object_z(y: f32) -> f32 {
    Z_BASE_OBJECTS - y / SCREEN_SIZE.1
}

pub fn as_object_vec3(vec: Vec2) -> Vec3 {
    vec.extend(get_object_z(vec.y))
}

const CHUNK_SIZE: u32 = 32;
const TILE_SIZE: f32 = 16.;
const Z_BASE_OBJECTS: f32 = 200.; // Ground object sprites.
const FLOOR_LAYER: f32 = 0.;
const WALL_LAYER: f32 = 1.;
const SCALE: f32 = 1.;
const RANDOM_WALL_CHANCE: f32 = 0.05;
const STRUCTURE_DENSITY: f32 = 0.005;
const MIN_STRUCTURE_SIZE: u32 = 4;
const MAX_STRUCTURE_SIZE: u32 = 16;
const MIN_STRUCTURE_DAMAGE: f32 = 0.2;
const MAX_STRUCTURE_DAMAGE: f32 = 0.6;
const NUCLEAR_POOL_CHANCE: f32 = 0.3;
const NUCLEAR_POOL_SIZE: u32 = 4;
const NUCLEAR_POOL_NOISE: f32 = 4.;

struct Chunk {
    floor: Entity,
    // detail: Entity,
    walls: Entity,
}

#[derive(Default, Resource)]
struct ChunkManager {
    chunks: HashMap<IVec2, Chunk>,
}

fn spawn_chunk(commands: &mut Commands, assets: &AssetServer, chunk_pos: IVec2) -> Chunk {
    let map_size = TilemapSize {
        x: CHUNK_SIZE,
        y: CHUNK_SIZE,
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let mut transform = Transform::from_translation(Vec3::new(
        chunk_pos.x as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
        chunk_pos.y as f32 * CHUNK_SIZE as f32 * TILE_SIZE,
        0.0,
    ));
    transform.scale = Vec2::splat(SCALE).extend(1.);
    let mut rng = SmallRng::seed_from_u64(((chunk_pos.x as u64) << 32) | chunk_pos.y as u64);

    let mut floors = vec![0; (CHUNK_SIZE * CHUNK_SIZE) as usize];

    let mut walls = (0..CHUNK_SIZE * CHUNK_SIZE)
        .map(|_| rng.gen_bool(RANDOM_WALL_CHANCE as f64))
        .collect::<Vec<_>>();

    for _ in 0..((CHUNK_SIZE * CHUNK_SIZE) as f32 * STRUCTURE_DENSITY) as u32 {
        let x_size = rng.gen_range(MIN_STRUCTURE_SIZE..MAX_STRUCTURE_SIZE);
        let y_size = rng.gen_range(MIN_STRUCTURE_SIZE..MAX_STRUCTURE_SIZE);
        let x_start = (rng.gen::<f32>() * (CHUNK_SIZE - x_size) as f32) as u32;
        let y_start = (rng.gen::<f32>() * (CHUNK_SIZE - y_size) as f32) as u32;
        let x_center = x_start + x_size / 2;
        let y_center = y_start + y_size / 2;

        let damage = rng.gen_range(MIN_STRUCTURE_DAMAGE..MAX_STRUCTURE_DAMAGE);
        let nuclear_pool = rng.gen_bool(NUCLEAR_POOL_CHANCE as f64);

        for x in x_start..x_start + x_size {
            for y in y_start..y_start + y_size {
                floors[(x + y * CHUNK_SIZE) as usize] = if nuclear_pool
                    && ((x as i32 - x_center as i32).pow(2)
                        + (y as i32 - y_center as i32).pow(2)
                        + rng.gen_range(0.0..NUCLEAR_POOL_NOISE) as i32)
                        < NUCLEAR_POOL_SIZE.pow(2) as i32
                {
                    match (x % 2, y % 2) {
                        (0, 0) => 8,
                        (1, 0) => 9,
                        (0, 1) => 12,
                        (1, 1) => 13,
                        _ => unreachable!(),
                    }
                } else {
                    [2, 3, 6, 7, 10, 11, 14, 15]
                        .choose(&mut rng)
                        .copied()
                        .unwrap()
                }
            }
        }

        for (x, y) in [
            (Some(x_start), None),
            (Some(x_start + x_size - 1), None),
            (None, Some(y_start)),
            (None, Some(y_start + y_size - 1)),
        ] {
            for x in x
                .map(|x| x..x + 1)
                .unwrap_or_else(|| x_start..x_start + x_size)
            {
                for y in y
                    .map(|y| y..y + 1)
                    .unwrap_or_else(|| y_start..y_start + y_size)
                {
                    if !rng.gen_bool(damage as f64) {
                        walls[(x + y * CHUNK_SIZE) as usize] = true;
                    }
                }
            }
        }
    }

    let floor = {
        let floor_image: Handle<Image> = assets.load("art/atlas_floor.png");
        let mut floor_storage = TileStorage::empty(map_size);
        let floor_map = commands.spawn_empty().id();

        for (i, floor) in floors.into_iter().enumerate() {
            let position = TilePos {
                x: i as u32 % CHUNK_SIZE,
                y: i as u32 / CHUNK_SIZE,
            };

            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(floor),
                    tilemap_id: TilemapId(floor_map),
                    ..default()
                })
                .id();
            floor_storage.set(&position, tile);
        }

        let mut floor_transform = transform;
        floor_transform.translation.z = FLOOR_LAYER;
        commands
            .entity(floor_map)
            .insert(TilemapBundle {
                grid_size,
                size: map_size,
                storage: floor_storage,
                texture: TilemapTexture::Single(floor_image),
                tile_size,
                transform: floor_transform,
                ..default()
            })
            .id()
    };

    let walls = {
        let wall_image: Handle<Image> = assets.load("art/atlas_wall.png");
        let mut wall_storage = TileStorage::empty(map_size);
        let wall_map = commands.spawn_empty().id();

        for (i, wall) in walls.into_iter().enumerate() {
            let position = TilePos {
                x: i as u32 % CHUNK_SIZE,
                y: i as u32 / CHUNK_SIZE,
            };

            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(match wall {
                        true => 20,
                        false => 1,
                    }),
                    tilemap_id: TilemapId(wall_map),
                    ..default()
                })
                .id();
            wall_storage.set(&position, tile);
        }

        let mut wall_transform = transform;
        wall_transform.translation.z = WALL_LAYER;
        commands
            .entity(wall_map)
            .insert(TilemapBundle {
                grid_size,
                size: map_size,
                storage: wall_storage,
                texture: TilemapTexture::Single(wall_image),
                tile_size,
                transform: wall_transform,
                ..default()
            })
            .id()
    };

    Chunk { floor, walls }
}

fn wpos_to_cpos(wpos: Vec2) -> IVec2 {
    let wpos = (wpos - (CHUNK_SIZE as f32 * TILE_SIZE) * 0.5).as_ivec2();
    wpos / (CHUNK_SIZE as i32 * TILE_SIZE as i32)
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    assets: Res<AssetServer>,
    camera_query: Query<&Transform, With<Camera>>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = wpos_to_cpos(transform.translation.xy());
        for y in (camera_chunk_pos.y - 2)..=(camera_chunk_pos.y + 2) {
            for x in (camera_chunk_pos.x - 2)..=(camera_chunk_pos.x + 2) {
                if !chunk_manager.chunks.contains_key(&IVec2::new(x, y)) {
                    let chunk = spawn_chunk(&mut commands, &assets, IVec2::new(x, y));
                    chunk_manager.chunks.insert(IVec2::new(x, y), chunk);
                    // Don't generate more than one chunk per tick.
                    return;
                }
            }
        }
    }
}

fn despawn_outofrange_chunks(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    chunks_query: Query<(Entity, &Transform)>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for (entity, chunk_transform) in chunks_query.iter() {
        let cpos = wpos_to_cpos(chunk_transform.translation.xy());

        if camera_query.iter().all(|camera_transform| {
            let camera_cpos = wpos_to_cpos(camera_transform.translation.xy());
            let distance = (cpos - camera_cpos).abs().max_element();
            distance > 3
        }) {
            chunk_manager.chunks.remove(&cpos);
            commands.entity(entity).despawn_recursive();
        }
    }
}
