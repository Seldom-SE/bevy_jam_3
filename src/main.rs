mod asset;
mod camera;
mod day_night;
mod item;
mod map;
mod physics;
mod player;
mod stats;

use asset::asset_plugin;
use bevy::render::render_resource::{FilterMode, SamplerDescriptor};
use camera::camera_plugin;
use day_night::day_night_plugin;
use map::map_plugin;
use physics::physics_plugin;
use player::player_plugin;
use stats::stat_plugin;

use prelude::*;

pub const SCREEN_SIZE: (f32, f32) = (768.0, 768.0);

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
                        title: "Bevy Jam 3".into(),
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
        .fn_plugin(asset_plugin)
        .fn_plugin(camera_plugin)
        .fn_plugin(map_plugin)
        .fn_plugin(player_plugin)
        .fn_plugin(day_night_plugin)
        .fn_plugin(stat_plugin)
        .fn_plugin(physics_plugin)
        .run();
}

mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use leafwing_input_manager::prelude::*;
    pub use rand::prelude::*;
    pub use seldom_fn_plugin::FnPluginExt;
}
