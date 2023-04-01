mod camera;
mod day_night;
mod map;
mod player;

use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use bevy::render::view::RenderLayers;
use camera::camera_plugin;
use day_night::day_night_plugin;
use map::{get_floor_z, get_object_z};
use player::player_plugin;
use rand::prelude::*;

use prelude::*;

pub const TILE_SIZE: f32 = 16.0;
pub const SPRITE_SCALE: f32 = 4.0;
pub const Z_BASE_FLOOR: f32 = 100.0; // Base z-coordinate for 2D layers.
pub const Z_BASE_OBJECTS: f32 = 200.0; // Ground object sprites.
pub const SCREEN_SIZE: (f32, f32) = (768.0, 768.0);

// Misc components.
#[derive(Component)]
pub struct MouseLight;
#[derive(Component)]
pub struct Movable;

fn main() {
    // Basic setup.
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: SCREEN_SIZE.into(),
                        title: "Bevy Magic Light 2D: Krypta Example".into(),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin {
                    default_sampler: SamplerDescriptor {
                        mag_filter: FilterMode::Nearest,
                        min_filter: FilterMode::Nearest,
                        ..default()
                    },
                }),
        )
        .add_plugin(BevyMagicLight2DPlugin)
        .fn_plugin(camera_plugin)
        .fn_plugin(player_plugin)
        .fn_plugin(day_night_plugin)
        .insert_resource(BevyMagicLight2DSettings {
            light_pass_params: LightPassParams {
                reservoir_size: 16,
                smooth_kernel_size: (2, 1),
                direct_light_contrib: 0.2,
                indirect_light_contrib: 0.8,
                ..default()
            },
        })
        .add_startup_system(setup.after(setup_post_processing_camera))
        .run();
}

#[rustfmt::skip]
fn setup(
    mut commands:               Commands,
        asset_server:           Res<AssetServer>,
    mut texture_atlases:        ResMut<Assets<TextureAtlas>>,
) {

    // Maze map. 1 represents wall.
    let walls_info: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    // Generate square occluders from walls_info.
    let block_size    = Vec2::splat(TILE_SIZE * SPRITE_SCALE);
    let center_offset = Vec2::new(-1024.0, 1024.0) / 2.0
                      + block_size / 2.0
                      - Vec2::new(0.0, block_size.y);

    let get_block_translation = |i: usize, j: usize| {
        center_offset + Vec2::new((j as f32) * block_size.x, -(i as f32) * block_size.y)
    };

    let mut walls = vec![];

    // Load floor tiles.
    let floor_atlas_rows = 4;
    let floor_atlas_cols = 4;
    let floor_atlas_size = Vec2::new(16.0, 16.0);
    let floor_image = asset_server.load("art/atlas_floor.png");
    let floor_atlas = texture_atlases.add(TextureAtlas::from_grid(
        floor_image,
        floor_atlas_size,
        floor_atlas_cols,
        floor_atlas_rows,
        None,
        None,
    ));

    // Load wall tiles.
    let wall_atlas_rows = 5;
    let wall_atlas_cols = 6;
    let wall_atlas_size = Vec2::new(16.0, 16.0);
    let wall_image = asset_server.load("art/atlas_wall.png");
    let wall_atlas = texture_atlases.add(TextureAtlas::from_grid(
        wall_image,
        wall_atlas_size,
        wall_atlas_cols,
        wall_atlas_rows,
        None,
        None,
    ));

    // Load decoration sprites.
    let decorations_image = asset_server.load("art/atlas_decoration.png");

    // Spawn floor tiles.
    let mut rng = thread_rng();
    let mut floor_tiles = vec![];
    for (i, row) in walls_info.iter().enumerate() {
        for (j, _) in row.iter().enumerate() {

            let xy = get_block_translation(i, j);
            let z  = get_floor_z(xy.y);
            let id = rng.gen_range(0..(floor_atlas_cols * floor_atlas_rows));

            floor_tiles.push( commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(xy.x, xy.y, z),
                        scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                        ..default()
                    },
                    sprite: TextureAtlasSprite::new(id),
                    texture_atlas: floor_atlas.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR)).id());
        }
    }

    commands
        .spawn(Name::new("floor_tiles"))
        .insert(SpatialBundle::default())
        .push_children(&floor_tiles);

    let maze_rows = walls_info.len() as i32;
    let maze_cols = walls_info[0].len() as i32;

    // Get wall value clamping to edge.
    let get_wall_safe = |r: i32, c: i32, offset: (i32, i32)| {
        let r1 = r + offset.0;
        let c1 = c + offset.1;
        if r1 < 0 || r1 >= maze_rows {
            return 1;
        }
        if c1 < 0 || c1 >= maze_cols {
            return 1;
        }
        walls_info[r1 as usize][c1 as usize]
    };

    // Get atlas sprite index for wall.
    let get_wall_sprite_index = |r: usize, c: usize| {
        let r = r as i32;
        let c = c as i32;

        let w_up    = get_wall_safe(r, c, (-1,  0));
        let w_down  = get_wall_safe(r, c, ( 1,  0));
        let w_left  = get_wall_safe(r, c, ( 0, -1));
        let w_right = get_wall_safe(r, c, ( 0,  1));

        let total_walls = w_up + w_down + w_left + w_right;

        if total_walls == 4 {
            return 0;
        }

        if total_walls == 3 {
            if w_up == 0 {
                return wall_atlas_cols;
            }
            if w_left == 0 {
                return wall_atlas_cols + 1;
            }
            if w_down == 0 {
                return wall_atlas_cols + 2;
            }
            if w_right == 0 {
                return wall_atlas_cols + 3;
            }
        }

        if total_walls == 2 {
            if w_left == 1 && w_right == 1 {
                return wall_atlas_cols * 2;
            }

            if w_up == 1 && w_down == 1 {
                return wall_atlas_cols * 2 + 1;
            }

            if w_up == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 2;
            }

            if w_down == 1 && w_left == 1 {
                return wall_atlas_cols * 2 + 3;
            }

            if w_up == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 4;
            }

            if w_down == 1 && w_right == 1 {
                return wall_atlas_cols * 2 + 5;
            }
        }

        if total_walls == 1 {
            if w_left == 1 {
                return wall_atlas_cols * 3;
            }
            if w_down == 1 {
                return wall_atlas_cols * 3 + 1;
            }
            if w_up == 1 {
                return wall_atlas_cols * 3 + 2;
            }
            if w_right == 1 {
                return wall_atlas_cols * 3 + 3;
            }
        }

        wall_atlas_cols * 4
    };

    // Add walls with occluder component.
    let occluder_data = LightOccluder2D { h_size: block_size / 2.0 };
    for (i, row) in walls_info.iter().enumerate() {
        for (j, cell) in row.iter().enumerate() {
            if *cell == 1 {
                let xy = get_block_translation(i, j);
                let z  = get_object_z(xy.y);
                let id = get_wall_sprite_index(i, j);

                walls.push(commands.spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(xy.x, xy.y, z),
                        scale: Vec2::splat(SPRITE_SCALE).extend(0.0),
                        ..default()
                    },
                    sprite: TextureAtlasSprite::new(id),
                    texture_atlas: wall_atlas.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_WALLS))
                .insert(occluder_data).id());
            }
        }
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("walls"))
        .push_children(&walls);

    // Add decorations.
    // TODO: consider adding some utility function to avoid code duplication.
    let mut decorations = vec![];
    {
        let mut decorations_atlas = TextureAtlas::new_empty(
            decorations_image,
            Vec2::new(256.0, 256.0));

        let candle_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(16.0, 16.0),
        });
        let candle_rect_2 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(16.0, 0.0),
            max: Vec2::new(32.0, 16.0),
        });
        let candle_rect_3 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(32.0, 0.0),
            max: Vec2::new(48.0, 16.0),
        });
        let candle_rect_4 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(48.0, 0.0),
            max: Vec2::new(64.0, 16.0),
        });
        let tomb_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(32.0, 16.0),
            max: Vec2::new(80.0, 48.0),
        });
        let sewerage_rect_1 = decorations_atlas.add_texture(Rect {
            min: Vec2::new(0.0, 16.0),
            max: Vec2::new(32.0, 48.0),
        });

        let texture_atlas_handle = texture_atlases.add(decorations_atlas);

        // Candle 1.
        {
            let x = 100.0;
            let y = -388.5;
            let mut sprite = TextureAtlasSprite::new(candle_rect_1);
            sprite.color = Color::rgb_u8(180, 180, 180);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_1")).id());

        }

        // Candle 2.
        {
            let x = -32.1;
            let y = -384.2;
            let mut sprite = TextureAtlasSprite::new(candle_rect_2);
            sprite.color = Color::rgb_u8(180, 180, 180);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_2")).id());
        }

        // Candle 3.
        {
            let x = -351.5;
            let y = -126.0;
            let mut sprite = TextureAtlasSprite::new(candle_rect_3);
            sprite.color = Color::rgb_u8(180, 180, 180);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_3")).id());
        }

        // Candle 4.
        {
            let x = 413.0;
            let y = -124.6;
            let mut sprite = TextureAtlasSprite::new(candle_rect_4);
            sprite.color = Color::rgb_u8(180, 180, 180);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::splat(2.0),
                })
                .insert(Name::new("candle_4")).id());
        }

        // Tomb 1.
        {
            let x = 31.5;
            let y = -220.0;
            let mut sprite = TextureAtlasSprite::new(tomb_rect_1);
            sprite.color = Color::rgb_u8(255, 255, 255);
            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::new(72.8, 31.0),
                })
                .insert(Name::new("tomb_1")).id());
        }

        // Tomb 1.
        {
            let x = 300.5;
            let y = -500.0;
            let mut sprite = TextureAtlasSprite::new(tomb_rect_1);
            sprite.color = Color::rgb_u8(255, 255, 255);
            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle.clone(),
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_OBJECTS))
                .insert(LightOccluder2D {
                    h_size: Vec2::new(72.8, 31.0),
                })
                .insert(Name::new("tomb_1")).id());
        }

        // Sewerage 1.
        {
            let x = 31.5;
            let y = -38.5;
            let mut sprite = TextureAtlasSprite::new(sewerage_rect_1);
            sprite.color = Color::rgb_u8(255, 255, 255);

            decorations.push(commands
                .spawn(SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(x, y, get_object_z(y)),
                        scale: Vec2::splat(4.0).extend(0.0),
                        ..default()
                    },
                    sprite,
                    texture_atlas: texture_atlas_handle,
                    ..default()
                })
                .insert(RenderLayers::from_layers(CAMERA_LAYER_FLOOR)) // Add to floor
                .insert(Name::new("sewerage_1")).id());
        }
    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("decorations"))
        .push_children(&decorations);

    // Add lights.
    let mut lights = vec![];
    {
        let spawn_light =
            |cmd: &mut Commands, x: f32, y: f32, name: &'static str, light_source: OmniLightSource2D| {
                return cmd
                    .spawn(Name::new(name))
                    .insert(light_source)
                    .insert(SpatialBundle {
                        transform: Transform {
                            translation: Vec3::new(x, y, 0.0),
                            ..default()
                        },
                        ..default()
                    })
                    .insert(RenderLayers::all())
                    .id();
            };

        let base = OmniLightSource2D {
            falloff: Vec3::new(50.0, 20.0, 0.05),
            intensity: 10.0,
            ..default()
        };
        lights.push(spawn_light(
            &mut commands,
            90.667,
            -393.8,
            "outdoor_krypta_torch_1",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -36.000,
            -393.8,
            "outdoor_krypta_torch_2",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 8.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            230.9,
            -284.6,
            "indoor_krypta_light_1",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -163.5,
            -292.7,
            "indoor_krypta_light_2",
            OmniLightSource2D {
                intensity: 10.0,
                color: Color::rgb_u8(76, 57, 211),
                jitter_intensity: 2.0,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            -352.000,
            -131.2,
            "outdoor_krypta_torch_3",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            410.667,
            -141.8,
            "outdoor_krypta_torch_4",
            OmniLightSource2D {
                intensity: 4.5,
                color: Color::rgb_u8(137, 79, 24),
                jitter_intensity: 2.5,
                jitter_translation: 3.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            28.0,
            -34.0,
            "indoor_krypta_ghost_1",
            OmniLightSource2D {
                intensity: 0.8,
                color: Color::rgb_u8(6, 53, 6),
                jitter_intensity: 0.2,
                jitter_translation: 0.0,
                ..base
            },
        ));
        lights.push(spawn_light(
            &mut commands,
            31.392,
            -168.3,
            "indoor_krypta_tomb_1",
            OmniLightSource2D {
                intensity: 0.4,
                color: Color::rgb_u8(252, 182, 182),
                jitter_intensity: 0.05,
                jitter_translation: 4.7,
                ..base
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            40.0,
            -1163.2,
            "outdoor_light_9",
            OmniLightSource2D {
                intensity: 1.2,
                falloff: Vec3::new(50.0, 40.0, 0.03),
                color: Color::rgb_u8(0, 206, 94),
                jitter_intensity: 0.7,
                jitter_translation: 3.0,
            },
        ));

        lights.push(spawn_light(
            &mut commands,
            182.3,
            -1210.0,
            "outdoor_light_10",
            OmniLightSource2D {
                intensity: 1.2,
                falloff: Vec3::new(50.0, 40.0, 0.03),
                color: Color::rgb_u8(0, 206, 94),
                jitter_intensity: 0.7,
                jitter_translation: 3.0,
            },
        ));

    }
    commands
        .spawn(SpatialBundle::default())
        .insert(Name::new("lights"))
        .push_children(&lights);
}

// Might need this for structures
//
// // Add roofs.
// commands
//     .spawn(SpatialBundle {
//         transform: Transform {
//             translation: Vec3::new(30.0, -180.0, 0.0),
//             ..default()
//         },
//         ..default()
//     })
//     .insert(Name::new("skylight_mask_1"))
//     .insert(SkylightMask2D {
//         h_size: Vec2::new(430.0, 330.0),
//     });
// commands
//     .spawn(SpatialBundle {
//         transform: Transform {
//             translation: Vec3::new(101.6, -989.4, 0.0),
//             ..default()
//         },
//         ..default()
//     })
//     .insert(Name::new("skylight_mask_2"))
//     .insert(SkylightMask2D {
//         h_size: Vec2::new(163.3, 156.1),
//     });

mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_magic_light_2d::prelude::*;
    pub use leafwing_input_manager::prelude::*;
    pub use seldom_fn_plugin::FnPluginExt;
}
