use crate::{
    map::as_object_vec3,
    physics::Vel,
    prelude::*,
    stats::{Stat, StatBundle, Stats}, camera::PlayerCamera,
};
use enum_map::enum_map;

pub fn player_plugin(app: &mut App) {
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .init_resource::<CursorPos>()
        .add_startup_system(init)
        .add_system(player_move)
        .add_system(update_cursor_pos);
}

#[derive(Actionlike, Clone)]
pub enum Action {
    Move,
    Collect,
}

#[derive(Component)]
pub struct Player;

fn init(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let player_image = assets.load("art/player.png");
    let mut player_atlas = TextureAtlas::new_empty(player_image, Vec2::new(24.0, 24.0));

    let player_rect_1 = player_atlas.add_texture(Rect {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(24.0, 24.0),
    });

    let texture_atlas_handle = atlases.add(player_atlas);

    commands.spawn((
        SpriteSheetBundle {
            transform: Transform {
                translation: as_object_vec3(Vec2::splat(0.)),
                scale: Vec2::splat(0.2).extend(0.),
                ..default()
            },
            sprite: TextureAtlasSprite::new(player_rect_1),
            texture_atlas: texture_atlas_handle,
            ..default()
        },
        InputManagerBundle::<Action> {
            input_map: InputMap::default()
                .insert(VirtualDPad::wasd(), Action::Move)
                .insert(DualAxis::left_stick(), Action::Move)
                .insert(KeyCode::Space, Action::Collect)
                .insert(GamepadButtonType::South, Action::Collect)
                .build(),
            ..default()
        },
        StatBundle {
            stats: Stats::new(enum_map! {
                Stat::Speed => 5.0,
                Stat::Health => 30.0,
                Stat::Sight => 1.0,
                Stat::RadiationResistence => 0.0,
            }),
            ..default()
        },
        Player,
        Vel::default(),
        PointLight2d {
            color: Color::ORANGE_RED,
            strength: 5.0,
            falloff: 0.45,
        },
    ));
}

const PLAYER_SPEED: f32 = 200.0;

fn player_move(
    mut players: Query<(&mut Vel, &Transform, &ActionState<Action>), With<Player>>,
    mut cameras: Query<&mut Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    let Ok((mut vel, transform, state)) = players.get_single_mut() else { return };

    if state.pressed(Action::Move) {
        vel.0 = state
            .clamped_axis_pair(Action::Move)
            .unwrap()
            .xy()
            // TODO Avoid normalizing control stick
            .normalize_or_zero()
            * PLAYER_SPEED;
    } else {
        vel.0 = Vec2::ZERO;
    }

    let camera_translation = &mut cameras.single_mut().translation;
    *camera_translation = transform
        .translation
        .truncate()
        .extend(camera_translation.z);
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct CursorPos(Vec2);

fn update_cursor_pos(
    windows: Query<&Window>,
    cameras: Query<(&Camera, &GlobalTransform)>,
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
