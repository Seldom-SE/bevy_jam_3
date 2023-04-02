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
const WALL_LAYER: f32 = 1.;
const FLOOR_LAYER: f32 = 0.;
const SCALE: f32 = 4.;

// TODO Chunking https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/chunking.rs
fn init(mut commands: Commands, assets: Res<AssetServer>) {
    let wall_image: Handle<Image> = assets.load("art/atlas_wall.png");
    let floor_image: Handle<Image> = assets.load("art/atlas_floor.png");

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

    let mut wall_storage = TileStorage::empty(map_size);
    let wall_map = commands.spawn_empty().id();

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = TilePos { x, y };
            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(match rng.gen_bool(0.05) {
                        true => 20,
                        false => 1,
                    }),
                    tilemap_id: TilemapId(wall_map),
                    ..default()
                })
                .id();
            wall_storage.set(&position, tile);
        }
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
}
