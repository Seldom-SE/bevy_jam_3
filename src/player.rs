use std::cmp::Ordering;

use crate::{
    asset::GameAssets,
    camera::PlayerCamera,
    construct::PowerSource,
    entities::EnemyBullet,
    map::as_object_vec3,
    physics::Vel,
    prelude::*,
    stats::{Radiation, Stat, StatBundle, Stats},
};
use bevy_kira_audio::prelude::AudioReceiver;
use enum_map::enum_map;
use leafwing_input_manager::user_input::InputKind;

pub fn player_plugin(app: &mut App) {
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .init_resource::<CursorPos>()
        .add_startup_system(init)
        .add_system(player_move)
        .add_system(update_cursor_pos)
        .add_system(update_player_power)
        .add_system(audio_follow_player)
        .add_system(player_hit_bullets);
}

#[derive(Actionlike, Clone)]
pub enum Action {
    Move,
    Collect,
}

#[derive(Component)]
pub struct Player;

fn init(mut commands: Commands, assets: Res<GameAssets>) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: as_object_vec3(Vec2::splat(0.)),
                scale: Vec2::splat(2.).extend(0.),
                ..default()
            },
            texture: assets.player[2].clone(),
            ..default()
        },
        InputManagerBundle::<Action> {
            input_map: InputMap::default()
                .insert(
                    VirtualDPad {
                        up: InputKind::Keyboard(KeyCode::W),
                        down: InputKind::Keyboard(KeyCode::S),
                        left: InputKind::Keyboard(KeyCode::A),
                        right: InputKind::Keyboard(KeyCode::D),
                    },
                    Action::Move,
                )
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(KeyCode::Space, Action::Collect)
                .insert(GamepadButtonType::South, Action::Collect)
                .build(),
            ..default()
        },
        StatBundle {
            stats: Stats::new(enum_map! {
                Stat::Speed => 200.0,
                Stat::Health => 30.0,
                Stat::Sight => 1.0,
                Stat::RadiationResistence => 1.,
            }),
            ..default()
        },
        PowerSource::default(),
        Player,
        Vel::default(),
        PointLight2d {
            color: Color::ORANGE_RED,
            strength: 5.0,
            falloff: 0.45,
        },
    ));
    commands.spawn((AudioReceiver, SpatialBundle::default()));
}

struct CurrDirection {
    north: bool,
    east: bool,
}

impl Default for CurrDirection {
    fn default() -> Self {
        Self {
            north: false,
            east: true,
        }
    }
}

fn player_move(
    mut players: Query<
        (
            &mut Vel,
            &mut Handle<Image>,
            &Transform,
            &Stats,
            &ActionState<Action>,
        ),
        With<Player>,
    >,
    mut cameras: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut curr_direction: Local<CurrDirection>,
) {
    let Ok((mut vel, mut image, transform, stats, state)) = players.get_single_mut() else {
        return
    };

    if state.pressed(Action::Move) {
        vel.0 = state
            .clamped_axis_pair(Action::Move)
            .unwrap()
            .xy()
            // TODO Avoid normalizing control stick
            .normalize_or_zero()
            * stats.get(Stat::Speed);
    } else {
        vel.0 = Vec2::ZERO;
    }

    let new_direction = CurrDirection {
        north: match vel.0.y.partial_cmp(&0.0) {
            Some(Ordering::Greater) => true,
            Some(Ordering::Less) => false,
            _ => curr_direction.north,
        },
        east: match vel.0.x.partial_cmp(&0.0) {
            Some(Ordering::Greater) => true,
            Some(Ordering::Less) => false,
            _ => curr_direction.east,
        },
    };

    *image = assets.player[match new_direction {
        CurrDirection {
            north: true,
            east: true,
        } => 0,
        CurrDirection {
            north: true,
            east: false,
        } => 1,
        CurrDirection {
            north: false,
            east: true,
        } => 2,
        CurrDirection {
            north: false,
            east: false,
        } => 3,
    }]
    .clone();

    *curr_direction = new_direction;

    let camera_translation = &mut cameras.single_mut().translation;
    let target = transform
        .translation
        .truncate()
        .extend(camera_translation.z);
    let dir = (target - *camera_translation).truncate();
    let l = dir.length();
    const CAM_SPEED: f32 = 8.0;
    let m = if l == 0.0 { Vec2::ZERO } else { dir / l }
        * (l * time.delta_seconds() * CAM_SPEED)
            .max(time.delta_seconds() * CAM_SPEED)
            .min(l);
    *camera_translation += m.extend(0.0);
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CursorPos(Vec2);

fn update_cursor_pos(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<PlayerCamera>>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    let (camera, camera_transform) = cameras.single();

    if let Some(world_position) = windows
        .single()
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        **cursor_pos = world_position;
    }
}

const RADIATION_POWER_THRESHOLD: f32 = 0.6;

fn update_player_power(mut players: Query<(&mut PowerSource, &Radiation), With<Player>>) {
    for (mut power_source, radiation) in players.iter_mut() {
        **power_source = **radiation > RADIATION_POWER_THRESHOLD;
    }
}

fn audio_follow_player(
    mut audio_receivers: Query<&mut Transform, With<AudioReceiver>>,
    players: Query<&Transform, (With<Player>, Without<AudioReceiver>)>,
) {
    if let Ok(player_transform) = players.get_single() {
        if let Ok(mut audio_receiver_transform) = audio_receivers.get_single_mut() {
            audio_receiver_transform.translation = player_transform.translation;
        }
    }
}

fn player_hit_bullets(
    mut commands: Commands,
    mut players: Query<(&mut Radiation, &Transform), With<Player>>,
    bullets: Query<(Entity, &Transform), With<EnemyBullet>>,
) {
    for (mut radiation, transform) in &mut players {
        for (bullet, bullet_transform) in &bullets {
            if (transform.translation.truncate() - bullet_transform.translation.truncate())
                .length_squared()
                < 30. * 30.
            {
                **radiation += 0.1;
                commands.entity(bullet).despawn();
            }
        }
    }
}
