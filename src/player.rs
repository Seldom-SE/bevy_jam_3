use crate::{map::as_object_vec3, prelude::*};

pub fn player_plugin(app: &mut App) {
    app.add_plugin(InputManagerPlugin::<Action>::default())
        .add_startup_system(init)
        .add_system(player_move);
}

#[derive(Actionlike, Clone)]
enum Action {
    Move,
}

#[derive(Component)]
struct Player;

fn init(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    let player_image = assets.load("art/atlas_decoration.png");
    let mut player_atlas = TextureAtlas::new_empty(player_image, Vec2::new(256.0, 256.0));

    let player_rect_1 = player_atlas.add_texture(Rect {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(16.0, 16.0),
    });

    let texture_atlas_handle = atlases.add(player_atlas);

    commands.spawn((
        SpriteSheetBundle {
            transform: Transform {
                translation: as_object_vec3(Vec2::splat(0.)),
                scale: Vec2::splat(4.).extend(0.),
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
                .build(),
            ..default()
        },
        Player,
    ));
}

const PLAYER_SPEED: f32 = 100.0;

fn player_move(
    mut players: Query<(&mut Transform, &ActionState<Action>), With<Player>>,
    mut cameras: Query<&mut Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((mut transform, state)) = players.get_single_mut() else { return };

    if state.pressed(Action::Move) {
        let translation = &mut transform.translation;
        *translation = as_object_vec3(
            translation.truncate()
                + state
                    .clamped_axis_pair(Action::Move)
                    .unwrap()
                    .xy()
                    // TODO Avoid normalizing control stick
                    .normalize_or_zero()
                    * time.delta_seconds()
                    * PLAYER_SPEED,
        )
    }

    let camera_translation = &mut cameras.single_mut().translation;
    *camera_translation = transform
        .translation
        .truncate()
        .extend(camera_translation.z);
}
