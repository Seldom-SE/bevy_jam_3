#![allow(clippy::type_complexity, clippy::too_many_arguments)]
#![feature(int_roundings)]
mod asset;
mod camera;
mod construct;
mod day_night;
mod ecs;
mod entities;
mod item;
mod map;
mod physics;
mod player;
mod sprite;
mod stats;

use asset::asset_plugin;
use bevy::{
    render::render_resource::{FilterMode, SamplerDescriptor},
    sprite::SpritePlugin,
};
use bevy_kira_audio::{prelude::SpacialAudio, AudioPlugin};
use camera::camera_plugin;
use construct::construct_plugin;
use day_night::day_night_plugin;
use entities::animation_plugin;
use item::item_plugin;
use map::map_plugin;
use physics::physics_plugin;
use player::player_plugin;
use stats::stat_plugin;

use prelude::*;

pub const SCREEN_SIZE: (f32, f32) = (768.0, 768.0);

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: SCREEN_SIZE.into(),
                    // fit_canvas_to_parent: true,
                    // Allow keys like `F11` to work in the browser.
                    prevent_default_event_handling: false,
                    title: "Bevy Jam 3".into(),
                    resizable: true,
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
            })
            .disable::<SpritePlugin>(),
    )
    .add_plugin(sprite::SpritePlugin);
    #[cfg(feature = "editor")]
    app.add_plugin(bevy_editor_pls::prelude::EditorPlugin::default());
    // Basic setup.
    app.insert_resource(ClearColor(Color::rgb_u8(0, 0, 0)))
        .insert_resource(SpacialAudio { max_distance: 500. })
        .add_plugin(AudioPlugin)
        .fn_plugin(asset_plugin)
        .fn_plugin(camera_plugin)
        .fn_plugin(construct_plugin)
        .fn_plugin(item_plugin)
        .fn_plugin(map_plugin)
        .fn_plugin(player_plugin)
        .fn_plugin(day_night_plugin)
        .fn_plugin(stat_plugin)
        .fn_plugin(physics_plugin)
        .fn_plugin(animation_plugin)
        .run();
}

mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use leafwing_input_manager::prelude::*;
    pub use rand::prelude::*;
    pub use seldom_fn_plugin::FnPluginExt;
    pub use seldom_state::prelude::*;
}
