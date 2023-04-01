use bevy::render::{camera::RenderTarget, view::RenderLayers};

use crate::prelude::*;

pub fn camera_plugin(app: &mut App) {
    app.add_startup_system(init.after(setup_post_processing_camera));
}

fn init(mut commands: Commands, post_processing: Res<PostProcessingTarget>) {
    let (floor_target, walls_target, objects_target) =
        post_processing.handles.as_ref().unwrap().clone();

    enum CameraTy {
        Floor,
        Walls,
        Objects,
    }

    for (ty, target, layer) in [
        (CameraTy::Floor, floor_target, CAMERA_LAYER_FLOOR),
        (CameraTy::Walls, walls_target, CAMERA_LAYER_WALLS),
        (CameraTy::Objects, objects_target, CAMERA_LAYER_OBJECTS),
    ] {
        let mut camera = commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    hdr: false,
                    target: RenderTarget::Image(target),
                    ..default()
                },
                ..default()
            },
            Name::new("main_camera_floor"),
            SpriteCamera,
            RenderLayers::from_layers(layer),
            UiCameraConfig { show_ui: false },
        ));

        match ty {
            CameraTy::Floor => {
                camera.insert(FloorCamera);
            }
            CameraTy::Walls => {
                camera.insert(WallsCamera);
            }
            CameraTy::Objects => {
                camera.insert(ObjectsCamera);
            }
        }
    }
}
