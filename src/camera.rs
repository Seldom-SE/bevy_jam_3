use crate::prelude::*;

pub fn camera_plugin(app: &mut App) {
    app.add_startup_system(init);
}

fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
