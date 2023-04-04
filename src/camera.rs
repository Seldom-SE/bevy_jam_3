use crate::prelude::*;

#[derive(Component)]
pub struct PlayerCamera;

pub fn camera_plugin(app: &mut App) {
    app.add_startup_system(init);
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.7,
                ..default()
            },
            ..default()
        },
        PlayerCamera,
    ));
}
