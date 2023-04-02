use enum_map::{enum_map, EnumMap};

use crate::{item::Item, prelude::*};

pub fn asset_plugin(app: &mut App) {
    app.add_system(load.in_base_set(StartupSet::PreStartup));
}

#[derive(Resource)]
pub struct Assets {
    pub items: EnumMap<Item, Handle<Image>>,
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Assets {
        items: enum_map! {
            Item::Circuit => asset_server.load("art/circuit.png"),
            Item::Metal => asset_server.load("art/metal.png"),
            Item::CannedFood => asset_server.load("art/canned_food.png"),
            Item::Plant => asset_server.load("art/plant.png"),
        },
    })
}
