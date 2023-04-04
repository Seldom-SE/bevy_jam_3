mod gen;

use bevy::{
    math::{Vec3Swizzles, Vec4Swizzles},
    utils::HashMap,
};

use crate::{asset::GameAssets, entities::TextureAtlases, physics::Vel, prelude::*, SCREEN_SIZE};

pub fn map_plugin(app: &mut App) {
    app.add_plugin(TilemapPlugin)
        .init_resource::<ChunkManager>()
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

const SEED: u32 = 41;
pub struct Chunk {
    pub floor: Entity,
    // pub detail: Entity,
    pub walls: Entity,
}

#[derive(Component)]
struct ChunkMarker;

#[derive(Default, Resource)]
pub struct ChunkManager {
    chunks: HashMap<IVec2, Chunk>,
}
pub type ChunkQueryMut<'world, 'state, 'a> = Query<
    'world,
    'state,
    (
        &'a mut TileStorage,
        &'a TilemapSize,
        &'a TilemapGridSize,
        &'a TilemapType,
        &'a Transform,
    ),
    Without<Vel>,
>;
pub type ChunkQuery<'world, 'state, 'a> = Query<
    'world,
    'state,
    (
        &'a TileStorage,
        &'a TilemapSize,
        &'a TilemapGridSize,
        &'a TilemapType,
        &'a Transform,
    ),
    Without<Vel>,
>;

pub struct TileEntryMut<'a> {
    storage: Mut<'a, TileStorage>,
    chunk_pos: IVec2,
    tile_pos: TilePos,
    entity: Entity,
}

impl<'a> TileEntryMut<'a> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn despawn(&mut self, commands: &mut Commands) {
        self.storage.remove(&self.tile_pos);
        commands.entity(self.entity).despawn_recursive();
    }
}

pub struct TileEntry<'a> {
    storage: &'a TileStorage,
    chunk_pos: IVec2,
    tile_pos: TilePos,
    entity: Entity,
}

impl<'a> TileEntry<'a> {
    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn project_to_edge(&self, start: Vec2, end: Vec2) -> Option<Vec2> {
        let d = end - start;
        let tile_wpos = (self.chunk_pos.as_vec2() * CHUNK_SIZE as f32
            + Vec2::new(self.tile_pos.x as f32, self.tile_pos.y as f32))
            * TILE_SIZE;
        let half_tile = Vec2::splat(TILE_SIZE * 0.5);

        let l = tile_wpos - start;

        let t0 = (l - half_tile) / d;
        let t1 = (l + half_tile) / d;
        let tmin = t0.max_element();
        let tmax = t1.min_element();

        if tmax < tmin || tmax < 0.0 || tmin > 1.0 {
            return None;
        }

        if tmin > 0.0 {
            Some(start + d * tmin)
        } else {
            Some(end)
        }
    }
}

impl ChunkManager {
    fn get_tile_mut<'a>(
        &self,
        wpos: Vec2,
        chunk_query: &'a mut ChunkQueryMut,
        tilemap: impl FnOnce(&Chunk) -> Entity,
    ) -> Option<TileEntryMut<'a>> {
        let wpos = wpos - Vec2::ONE * 0.5;
        let cpos = wpos_to_cpos(wpos);
        self.chunks.get(&cpos).and_then(|chunk| {
            let (storage, size, grid_size, ty, transform) =
                chunk_query.get_mut(tilemap(chunk)).ok()?;
            let in_map_pos: Vec2 = {
                let pos = Vec4::from((wpos, 0.0, 1.0));
                let in_map_pos = transform.compute_matrix().inverse() * pos;
                in_map_pos.xy()
            };
            let in_map_pos = Vec2::new(
                in_map_pos.x.rem_euclid(CHUNK_SIZE as f32 * TILE_SIZE),
                in_map_pos.y.rem_euclid(CHUNK_SIZE as f32 * TILE_SIZE),
            );

            let tile_pos = TilePos::from_world_pos(&in_map_pos, size, grid_size, ty)?;
            storage.get(&tile_pos).map(|entity| TileEntryMut {
                storage,
                chunk_pos: cpos,
                tile_pos,
                entity,
            })
        })
    }

    fn get_tile<'a>(
        &self,
        wpos: Vec2,
        chunk_query: &'a ChunkQuery,
        tilemap: impl FnOnce(&Chunk) -> Entity,
    ) -> Option<TileEntry<'a>> {
        let wpos = wpos - Vec2::ONE * 0.5;
        let cpos = wpos_to_cpos(wpos);
        self.chunks.get(&cpos).and_then(|chunk| {
            let (storage, size, grid_size, ty, transform) = chunk_query.get(tilemap(chunk)).ok()?;
            let in_map_pos: Vec2 = {
                let pos = Vec4::from((wpos, 0.0, 1.0));
                let in_map_pos = transform.compute_matrix().inverse() * pos;
                in_map_pos.xy()
            };
            let in_map_pos = Vec2::new(
                in_map_pos.x.rem_euclid(CHUNK_SIZE as f32 * TILE_SIZE),
                in_map_pos.y.rem_euclid(CHUNK_SIZE as f32 * TILE_SIZE),
            );

            let tile_pos = TilePos::from_world_pos(&in_map_pos, size, grid_size, ty)?;
            storage.get(&tile_pos).map(|entity| TileEntry {
                storage,
                chunk_pos: cpos,
                tile_pos,
                entity,
            })
        })
    }

    pub fn get_wall_tile_mut<'a>(
        &self,
        wpos: Vec2,
        chunk_query: &'a mut ChunkQueryMut,
    ) -> Option<TileEntryMut<'a>> {
        self.get_tile_mut(wpos, chunk_query, |chunk| chunk.walls)
    }

    pub fn get_wall_tile<'a>(
        &self,
        wpos: Vec2,
        chunk_query: &'a ChunkQuery,
    ) -> Option<TileEntry<'a>> {
        self.get_tile(wpos, chunk_query, |chunk| chunk.walls)
    }

    pub fn get_floor_tile<'a>(
        &self,
        wpos: Vec2,
        chunk_query: &'a ChunkQuery,
    ) -> Option<TileEntry<'a>> {
        self.get_tile(wpos, chunk_query, |chunk| chunk.floor)
    }
}

fn spawn_chunk(
    commands: &mut Commands,
    assets: &GameAssets,
    asset_server: &AssetServer,
    atlases: &TextureAtlases,
    chunk_pos: IVec2,
) -> Chunk {
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
    let seed = SEED;
    let chunk_data = gen::gen_chunk(chunk_pos, seed);
    let mut rng = SmallRng::seed_from_u64(
        (((chunk_pos.x as u64) << 32) | chunk_pos.y as u64).rotate_right(11)
            ^ ((seed as u64) << 15),
    );

    for (pos, item) in chunk_data.items {
        commands.spawn((
            SpriteBundle {
                texture: assets.items[item].clone(),
                transform: Transform::from_translation(as_object_vec3(pos * TILE_SIZE)),
                ..default()
            },
            item,
        ));
    }

    for (pos, enemy) in chunk_data.enemies {
        enemy.spawn(pos * TILE_SIZE, commands, atlases);
    }

    let floor = {
        let floor_image: Handle<Image> = asset_server.load("art/atlas_floor.png");
        let mut floor_storage = TileStorage::empty(map_size);
        let floor_map = commands.spawn(ChunkMarker).id();

        for (i, floor) in chunk_data.floor.into_iter().enumerate() {
            let position = TilePos {
                x: i as u32 % CHUNK_SIZE,
                y: i as u32 / CHUNK_SIZE,
            };
            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(match floor {
                        gen::FloorTile::Ground => 0,
                        gen::FloorTile::Water => match (position.x % 2, position.y % 2) {
                            (0, 0) => 8,
                            (1, 0) => 9,
                            (0, 1) => 12,
                            (1, 1) => 13,
                            _ => unreachable!(),
                        },
                        gen::FloorTile::Concrete => [2, 3, 6, 7, 10, 11, 14, 15]
                            .choose(&mut rng)
                            .copied()
                            .unwrap(),
                        gen::FloorTile::Floor => 3,
                    }),
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
        let wall_image: Handle<Image> = asset_server.load("art/atlas_wall.png");
        let mut wall_storage = TileStorage::empty(map_size);
        let wall_map = commands.spawn(ChunkMarker).id();

        for (i, wall) in chunk_data.walls.into_iter().enumerate() {
            let tile_texture_index = match wall {
                gen::WallTile::None => continue,
                gen::WallTile::Wall => 20,
            };
            let position = TilePos {
                x: i as u32 % CHUNK_SIZE,
                y: i as u32 / CHUNK_SIZE,
            };

            let tile = commands
                .spawn(TileBundle {
                    position,
                    texture_index: TileTextureIndex(tile_texture_index),
                    tilemap_id: TilemapId(wall_map),
                    ..default()
                })
                .id();
            wall_storage.set(&position, tile);
        }

        let mut wall_transform = transform;
        wall_transform.translation.z = WALL_LAYER * 10.0;
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
    (wpos / (CHUNK_SIZE as f32 * TILE_SIZE)).floor().as_ivec2()
}

fn spawn_chunks_around_camera(
    mut commands: Commands,
    camera_query: Query<&Transform, With<Camera>>,
    asset_server: Res<AssetServer>,
    assets: Res<GameAssets>,
    atlases: Res<TextureAtlases>,
    mut chunk_manager: ResMut<ChunkManager>,
) {
    for transform in camera_query.iter() {
        let camera_chunk_pos = wpos_to_cpos(transform.translation.xy());
        for y in (camera_chunk_pos.y - 2)..=(camera_chunk_pos.y + 2) {
            for x in (camera_chunk_pos.x - 2)..=(camera_chunk_pos.x + 2) {
                let cpos = IVec2::new(x, y);
                if !chunk_manager.chunks.contains_key(&cpos) {
                    let chunk = spawn_chunk(&mut commands, &assets, &asset_server, &atlases, cpos);
                    chunk_manager.chunks.insert(cpos, chunk);
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
    chunks_query: Query<(Entity, &Transform), With<ChunkMarker>>,
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