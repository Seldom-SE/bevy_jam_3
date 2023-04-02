use crate::{prelude::*, SCREEN_SIZE};

pub fn map_plugin(app: &mut App) {
    app.add_startup_system(init);
}

// TODO Make this between 2 and 3
pub fn get_object_z(y: f32) -> f32 {
    Z_BASE_OBJECTS - y / SCREEN_SIZE.1
}

pub fn as_object_vec3(vec: Vec2) -> Vec3 {
    vec.extend(get_object_z(vec.y))
}

const MAP_SIZE: u32 = 128;
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

// TODO Chunking https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/chunking.rs
fn init(mut commands: Commands, assets: Res<AssetServer>) {
    let map_size = TilemapSize {
        x: MAP_SIZE,
        y: MAP_SIZE,
    };
    let tile_size = TilemapTileSize {
        x: TILE_SIZE,
        y: TILE_SIZE,
    };
    let grid_size = tile_size.into();
    let mut transform = get_tilemap_center_transform(&map_size, &grid_size, &default(), 0.);
    transform.scale = Vec2::splat(SCALE).extend(1.);
    let mut rng = thread_rng();

    let floor_image: Handle<Image> = assets.load("art/atlas_floor.png");
    let mut floor_storage = TileStorage::empty(map_size);
    let floor_map = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = TilePos { x, y };
            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(0),
                    tilemap_id: TilemapId(floor_map),
                    ..default()
                })
                .id();
            floor_storage.set(&position, tile);
        }
    }

    let mut floor_transform = transform;
    floor_transform.translation.z = FLOOR_LAYER;
    commands.entity(floor_map).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: floor_storage,
        texture: TilemapTexture::Single(floor_image),
        tile_size,
        transform: floor_transform,
        ..default()
    });

    let wall_image: Handle<Image> = assets.load("art/atlas_wall.png");
    let mut wall_storage = TileStorage::empty(map_size);
    let wall_map = commands.spawn_empty().id();

    let mut walls = (0..MAP_SIZE * MAP_SIZE)
        .map(|_| rng.gen_bool(RANDOM_WALL_CHANCE as f64))
        .collect::<Vec<_>>();

    for _ in 0..((MAP_SIZE * MAP_SIZE) as f32 * STRUCTURE_DENSITY) as u32 {
        println!();
        let x_size = rng.gen_range(MIN_STRUCTURE_SIZE..MAX_STRUCTURE_SIZE);
        let y_size = rng.gen_range(MIN_STRUCTURE_SIZE..MAX_STRUCTURE_SIZE);
        let x_start = (rng.gen::<f32>() * (MAP_SIZE - x_size) as f32) as u32;
        let y_start = (rng.gen::<f32>() * (MAP_SIZE - y_size) as f32) as u32;

        let damage = rng.gen_range(MIN_STRUCTURE_DAMAGE..MAX_STRUCTURE_DAMAGE);

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
                        walls[(x + y * MAP_SIZE) as usize] = true;
                    }
                }
            }
        }
    }

    for (i, wall) in walls.into_iter().enumerate() {
        let position = TilePos {
            x: i as u32 % MAP_SIZE,
            y: i as u32 / MAP_SIZE,
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
    commands.entity(wall_map).insert(TilemapBundle {
        grid_size,
        size: map_size,
        storage: wall_storage,
        texture: TilemapTexture::Single(wall_image),
        tile_size,
        transform: wall_transform,
        ..default()
    });
}
