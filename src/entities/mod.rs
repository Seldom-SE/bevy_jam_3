use std::f32::consts::PI;

use bevy::{ecs::system::EntityCommands, math::Vec3Swizzles, prelude::*};

use crate::{
    map::as_object_vec3,
    physics::Vel,
    player::Player,
    stats::{Stat, StatBundle, Stats},
};
use enum_map::enum_map;

pub fn animation_plugin(app: &mut App) {
    app.add_startup_system(init)
        .add_system(animation)
        .add_system(follow_player_test);

    app.register_type::<Animation>();
}

pub fn follow_player_test(
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&Transform, &Stats, &mut Vel), Without<Player>>,
) {
    let Ok(player_transform) = player.get_single() else { return };
    let player_pos = player_transform.translation.xy();
    for (transform, stats, mut vel) in enemies.iter_mut() {
        let pos = transform.translation.xy();
        if pos.distance_squared(player_pos) < 80.0 * 80.0 {
            vel.0 = (player_pos - pos).normalize_or_zero() * stats.get(Stat::Speed);
        } else {
            vel.0 = Vec2::ZERO;
        }
    }
}

#[derive(Reflect, FromReflect, Clone, Debug)]
enum ClipMeta {
    /// Rotate towards velocity vector with an angle offset (in radians).
    RotateTowardsVelocity(f32),
    // Scale clip frame time with velocity.
    SpeedupWithVelocity,
}

#[derive(Reflect, FromReflect, Clone, Debug)]
struct Clip {
    start: usize,
    end: usize,
    frame_time: f32,
    meta: Vec<ClipMeta>,
}

impl Clip {
    fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            frame_time: 0.2,
            meta: Vec::new(),
        }
    }

    fn with_frame_time(mut self, frame_time: f32) -> Self {
        self.frame_time = frame_time;
        self
    }

    fn with_meta(mut self, meta: ClipMeta) -> Self {
        self.meta.push(meta);
        self
    }
}

#[derive(Default, Clone, Copy, Reflect, FromReflect, Debug)]
enum OnFinish {
    #[default]
    Repeat,
    Destroy,
    ReturnTo(usize),
}

#[derive(Default, Clone, Copy, Reflect, FromReflect, Debug)]
struct Playing {
    clip: usize,
    frame: usize,
    time: f32,
    on_finish: OnFinish,
}

const IDLE_ANIMATION: usize = 0;
const MOVE_ANIMATION: usize = 1;

#[derive(Component, Reflect, FromReflect, Default, Clone, Debug)]
#[reflect(Component)]
struct Animation {
    clips: Vec<Clip>,
    playing: Playing,
}

#[derive(Resource)]
pub struct TextureAtlases {
    slime: Handle<TextureAtlas>,
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("art/slime/atlas.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 28.0), 19, 1, None, None);
    let slime = texture_atlases.add(texture_atlas);

    commands.insert_resource(TextureAtlases { slime });
}

fn animation(
    mut commands: Commands,
    mut animations: Query<(
        Entity,
        &mut Animation,
        &mut TextureAtlasSprite,
        &mut Transform,
        &Vel,
    )>,
    time: Res<Time>,
) {
    for (entity, mut animation, mut sprite, mut transform, vel) in animations.iter_mut() {
        let mut playing = animation.playing;
        let current_clip = &animation.clips[animation.playing.clip];
        let mut frame_time = current_clip.frame_time;
        for meta in current_clip.meta.iter() {
            match meta {
                ClipMeta::RotateTowardsVelocity(a) => {
                    let target = transform.translation + Vec3::NEG_Z;
                    let up = Vec3::new(
                        vel.0.x * a.cos() - vel.0.y * a.sin(),
                        vel.0.x * a.sin() + vel.0.y * a.cos(),
                        0.0,
                    ).try_normalize().unwrap_or(Vec3::Y);
                    transform.look_at(target, up);
                }
                ClipMeta::SpeedupWithVelocity => frame_time /= vel.0.length(),
            }
        }
        playing.time += time.delta_seconds();
        if animation.playing.time > frame_time {
            playing.frame += 1;
            playing.time = 0.0;
            if animation.playing.frame >= current_clip.end - current_clip.start {
                playing.frame = current_clip.end - current_clip.start - 1;
                match animation.playing.on_finish {
                    OnFinish::Repeat => playing.frame = 0,
                    OnFinish::Destroy => commands.entity(entity).despawn(),
                    OnFinish::ReturnTo(clip) => playing = Playing { clip, ..default() },
                }
            }
        }
        match (playing.clip, vel.0.length() > 0.01) {
            (IDLE_ANIMATION, true) => {
                playing = Playing {
                    clip: MOVE_ANIMATION,
                    ..default()
                }
            }
            (MOVE_ANIMATION, false) => {
                playing = Playing {
                    clip: IDLE_ANIMATION,
                    ..default()
                }
            }
            _ => {}
        }
        animation.playing = playing;
        let sprite_index = animation.clips[playing.clip].start + playing.frame;
        if sprite_index != sprite.index {
            sprite.index = sprite_index;
        }
    }
}

pub fn spawn_slime<'w, 's, 'a>(
    position: Vec2,
    commands: &'a mut Commands<'w, 's>,
    atlases: &TextureAtlases,
) -> EntityCommands<'w, 's, 'a> {
    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlases.slime.clone(),
            transform: Transform::from_translation(as_object_vec3(position)),
            ..default()
        },
        StatBundle {
            stats: Stats::new(enum_map! {
                Stat::Speed => 80.0,
                Stat::Health => 10.0,
                Stat::Sight => 0.7,
                Stat::RadiationResistence => f32::INFINITY,
            }),
            ..default()
        },
        Animation {
            clips: vec![
                Clip::new(0, 11),
                Clip::new(12, 18)
                    .with_meta(ClipMeta::RotateTowardsVelocity(PI * 0.3))
                    .with_meta(ClipMeta::SpeedupWithVelocity)
                    .with_frame_time(10.0),
            ],
            playing: Playing::default(),
        },
        Vel::default(),
    ))
}
