use std::f32::consts::{PI, TAU};

use bevy::{ecs::system::EntityCommands, math::Vec3Swizzles};
use rand::{thread_rng, Rng};

use crate::{
    asset::GameAssets,
    map::as_object_vec3,
    physics::{DespawnOnCollide, Vel},
    player::Player,
    prelude::*,
    stats::{stat_propegation, RadiationSource, Stat, StatBundle, Stats},
};
use enum_map::enum_map;

pub fn animation_plugin(app: &mut App) {
    app.fn_plugin(state_machine_plugin)
        .fn_plugin(trigger_plugin::<RandomTrigger>)
        .fn_plugin(trigger_plugin::<NearPlayer>)
        .add_startup_system(init)
        .add_system(animation)
        .add_system(follow_player_test)
        .add_system(spawn_rustaches)
        .add_systems(
            (update_facing, apply_system_buffers)
                .chain()
                .before(stat_propegation),
        )
        .add_system(play_animation)
        .add_system(wander)
        .add_system(follow)
        .add_system(fire)
        .add_system(lifetime);

    app.register_type::<Animation>();
}

#[derive(Component, Default, Deref, DerefMut)]
struct WanderDirection(Option<Vec2>);

fn follow_player_test(
    player: Query<&Transform, With<Player>>,
    mut enemies: Query<(&Transform, &Stats, &mut Vel, &mut WanderDirection), Without<Player>>,
) {
    let Ok(player_transform) = player.get_single() else { return };
    let player_pos = player_transform.translation.xy();
    let mut rng = rand::thread_rng();
    for (transform, stats, mut vel, mut direction) in enemies.iter_mut() {
        let pos = transform.translation.xy();
        if pos.distance_squared(player_pos) < 256.0 * 256.0 {
            **direction = None;
            vel.0 = (player_pos - pos).normalize_or_zero() * stats.get(Stat::Speed);
        } else if let Some(dir) = **direction {
            // TODO: Make this framerate independent
            if rng.gen_bool(0.01) {
                **direction = None;
            } else {
                vel.0 = dir.normalize_or_zero() * stats.get(Stat::Speed);
            }
        } else if rng.gen_bool(0.01) {
            **direction = Some(
                Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize_or_zero(),
            );
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
    rustache: Handle<TextureAtlas>,
}

fn init(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let slime_texture_handle = asset_server.load("art/slime/atlas.png");
    let slime_texture_atlas = TextureAtlas::from_grid(
        slime_texture_handle,
        Vec2::new(32.0, 28.0),
        19,
        1,
        None,
        None,
    );
    let slime = texture_atlases.add(slime_texture_atlas);

    let rustache_texture_handle = asset_server.load("art/rustache.png");
    let rustache_texture_atlas = TextureAtlas::from_grid(
        rustache_texture_handle,
        Vec2::new(24.0, 24.0),
        2,
        2,
        None,
        None,
    );
    let rustache = texture_atlases.add(rustache_texture_atlas);

    commands.insert_resource(TextureAtlases { slime, rustache });
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
                    )
                    .try_normalize()
                    .unwrap_or(Vec3::Y);
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
                playing.frame = match current_clip.start == current_clip.end {
                    true => current_clip.start,
                    false => current_clip.end - current_clip.start - 1,
                };
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

#[derive(Component)]
pub struct EnemyMarker;

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
        WanderDirection::default(),
        RadiationSource {
            strength: 0.04,
            radius: 128.,
            active: true,
        },
        EnemyMarker,
    ))
}

fn spawn_rustache<'w, 's, 'a>(
    commands: &'a mut Commands<'w, 's>,
    atlases: &TextureAtlases,
    player_pos: Vec2,
) -> EntityCommands<'w, 's, 'a> {
    let disp = Vec2::from_angle(thread_rng().gen_range(0.0..TAU));

    commands.spawn((
        SpriteSheetBundle {
            sprite: TextureAtlasSprite::new(0),
            texture_atlas: atlases.rustache.clone(),
            transform: Transform::from_translation(as_object_vec3(player_pos + disp * 384.))
                .with_scale(Vec2::splat(2.).extend(1.)),
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
                Clip::new(0, 0),
                Clip::new(2, 2),
                Clip::new(0, 1),
                Clip::new(2, 3),
            ],
            playing: Playing::default(),
        },
        Vel::default(),
        StateMachine::new(Wander(-disp))
            .insert_on_enter::<Wander>(PlayAnimation(2, 3))
            .trans::<Wander>(RandomTrigger(0.0003), Idle)
            .insert_on_enter::<Idle>(PlayAnimation(0, 1))
            .trans_builder::<Idle, _, _>(RandomTrigger(0.0003), |_| {
                Some(Wander(Vec2::from_angle(thread_rng().gen_range(0.0..TAU))))
            })
            .trans_builder::<Wander, _, _>(NearPlayer(256.), |&player| Some(Follow(player)))
            .trans_builder::<Idle, _, _>(NearPlayer(256.), |&player| Some(Follow(player)))
            .insert_on_enter::<Follow>(PlayAnimation(2, 3))
            .trans::<Follow>(NotTrigger(NearPlayer(384.)), Idle)
            .trans_builder::<Follow, _, _>(NearPlayer(128.), |&player| {
                Some(Fire {
                    target: player,
                    cooldown: 1.5,
                    timer: 1.5,
                })
            })
            .insert_on_enter::<Fire>(PlayAnimation(0, 1))
            .trans_builder::<Fire, _, _>(NotTrigger(NearPlayer(256.)), |&player| {
                Some(Follow(player?))
            })
            .trans::<AnyState>(DoneTrigger::Failure, Idle),
        match disp.x >= 0. {
            true => Facing::Left,
            false => Facing::Right,
        },
        EnemyMarker,
    ))
}

#[derive(Clone, Copy)]
pub enum Enemy {
    Slime,
    Rustache,
}

impl Enemy {
    /// If the enemy is Rustache, the position is interpreted as the player position
    pub fn spawn<'w, 's, 'a>(
        &self,
        position: Vec2,
        commands: &'a mut Commands<'w, 's>,
        atlases: &TextureAtlases,
    ) -> EntityCommands<'w, 's, 'a> {
        match self {
            Enemy::Slime => spawn_slime(position, commands, atlases),
            Enemy::Rustache => spawn_rustache(commands, atlases, position),
        }
    }
}

#[derive(Clone, Component, Reflect)]
struct Wander(Vec2);

fn wander(mut wanderers: Query<(&mut Transform, &Wander)>, time: Res<Time>) {
    for (mut transform, &Wander(direction)) in &mut wanderers {
        let translation = &mut transform.translation;
        *translation = as_object_vec3(translation.xy() + direction * 80. * time.delta_seconds());
    }
}

#[derive(Clone, Component, Reflect)]
struct Idle;

#[derive(Clone, Component, Deref, DerefMut, Reflect)]
struct Follow(Entity);

// TODO Refactor rustaches to use `Vel`
fn follow(
    mut commands: Commands,
    followers: Query<(Entity, &Follow)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (entity, &Follow(target)) in &followers {
        let Ok([mut transform, target_transform]) = transforms.get_many_mut([entity, target]) else {
            commands.entity(entity).insert(Done::Failure);
            continue;
        };

        let translation = &mut transform.translation;
        *translation = as_object_vec3(
            translation.xy()
                + (target_transform.translation.xy() - translation.xy()).normalize_or_zero()
                    * 80.
                    * time.delta_seconds(),
        );
    }
}

#[derive(Clone, Component, Reflect)]
struct Fire {
    target: Entity,
    cooldown: f32,
    timer: f32,
}

#[derive(Component)]
pub struct EnemyBullet;

fn fire(
    mut commands: Commands,
    mut firers: Query<(Entity, &mut Fire)>,
    transforms: Query<&Transform>,
    time: Res<Time>,
    assets: Res<GameAssets>,
) {
    for (entity, mut fire) in &mut firers {
        fire.timer += time.delta_seconds();
        if fire.timer < fire.cooldown {
            continue;
        }
        fire.timer -= fire.cooldown;

        let Ok([&transform, target_transform]) = transforms.get_many([entity, fire.target]) else {
            commands.entity(entity).insert(Done::Failure);
            continue;
        };

        commands.spawn((
            SpriteBundle {
                texture: assets.nuclear_bullet.clone(),
                transform,
                ..default()
            },
            Vel(
                (target_transform.translation.xy() - transform.translation.xy())
                    .normalize_or_zero()
                    * 200.,
            ),
            Lifetime(5.),
            DespawnOnCollide,
            EnemyBullet,
        ));
    }
}

// TODO Make framerate independent
#[derive(Deref, DerefMut, Reflect)]
struct RandomTrigger(f32);

impl BoolTrigger for RandomTrigger {
    type Param<'w, 's> = ();

    fn trigger(&self, _: Entity, (): &Self::Param<'_, '_>) -> bool {
        thread_rng().gen::<f32>() < **self
    }
}

#[derive(Deref, DerefMut, Reflect)]
struct NearPlayer(f32);

impl Trigger for NearPlayer {
    type Param<'w, 's> = (
        Query<'w, 's, &'static Transform>,
        Query<'w, 's, Entity, With<Player>>,
    );
    type Ok = Entity;
    type Err = Option<Entity>;

    fn trigger(
        &self,
        entity: Entity,
        (transforms, players): &Self::Param<'_, '_>,
    ) -> Result<Entity, Option<Entity>> {
        let player = players.get_single().map_err(|_| None)?;
        (transforms
            .get(entity)
            .unwrap()
            .translation
            .xy()
            .distance_squared(transforms.get(player).unwrap().translation.xy())
            < self.powi(2))
        .then_some(player)
        .ok_or(Some(player))
    }
}

// Incredibly jank
fn update_facing(
    mut commands: Commands,
    mut facings: Query<(Entity, &mut Facing, AnyOf<(&Wander, &Follow, &Fire)>)>,
    transforms: Query<&Transform>,
) {
    for (entity, mut facing, (wander, follow, fire)) in &mut facings {
        *facing = match wander
            .map(|Wander(direction)| direction.x)
            .unwrap_or_else(|| {
                transforms
                    .get(
                        follow
                            .map(|&Follow(target)| target)
                            .unwrap_or_else(|| fire.unwrap().target),
                    )
                    .map(|target_transform| target_transform.translation.x)
                    .unwrap_or(0.)
                    - match transforms.get(entity) {
                        Ok(transform) => transform.translation.x,
                        Err(_) => 0.0,
                    }
            })
            >= 0.
        {
            true => Facing::Right,
            false => Facing::Left,
        };

        commands.entity(entity).insert(PlayAnimation(2, 3));
    }
}

#[derive(Clone, Component)]
struct PlayAnimation(usize, usize);

fn play_animation(
    mut commands: Commands,
    mut animations: Query<(Entity, &mut Animation, &PlayAnimation, &Facing)>,
) {
    for (entity, mut animation, &PlayAnimation(left, right), facing) in &mut animations {
        animation.playing = Playing {
            clip: match facing {
                Facing::Left => left,
                Facing::Right => right,
            },
            ..default()
        };

        commands.entity(entity).remove::<PlayAnimation>();
    }
}

#[derive(Component)]
pub enum Facing {
    Left,
    Right,
}

#[derive(Default, Deref, DerefMut)]
struct Repeating(f32);

fn spawn_rustaches(
    mut commands: Commands,
    mut timer: Local<Repeating>,
    players: Query<&Transform, With<Player>>,
    atlases: Res<TextureAtlases>,
    time: Res<Time>,
) {
    **timer += time.delta_seconds();
    if **timer < 1. {
        return;
    }
    **timer -= 1.;

    if thread_rng().gen_bool(1. / (time.elapsed_seconds_f64() / 400. + 1.).sqrt()) {
        return;
    }

    let Ok(player_pos) = players.get_single() else { return };
    Enemy::Rustache.spawn(player_pos.translation.xy(), &mut commands, &atlases);
}

#[derive(Component, Deref, DerefMut)]
pub struct Lifetime(pub f32);

fn lifetime(
    mut commands: Commands,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut lifetimes {
        **lifetime -= time.delta_seconds();
        if **lifetime <= 0. {
            commands.entity(entity).despawn();
        }
    }
}
