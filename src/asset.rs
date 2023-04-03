use enum_map::{enum_map, EnumMap};

use crate::{construct::Construct, item::Item, prelude::*};

pub fn asset_plugin(app: &mut App) {
    app.add_startup_system(load.in_base_set(StartupSet::PreStartup));
}

#[derive(Resource)]
pub struct GameAssets {
    pub items: EnumMap<Item, Handle<Image>>,
    pub constructs: EnumMap<Construct, Handle<Image>>,
    pub empty_item: Handle<Image>,
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        items: enum_map! {
            Item::Circuit => asset_server.load("art/circuit.png"),
            Item::Metal => asset_server.load("art/metal.png"),
            Item::CannedFood => asset_server.load("art/canned_food.png"),
            Item::Plant => asset_server.load("art/plant.png"),
            Item::Assembler => asset_server.load("art/assembler/on.png"),
            Item::Generator => asset_server.load("art/generator/generator_4.png"),
        },
        constructs: enum_map! {
            Construct::Generator => asset_server.load("art/generator/generator_0.png"),
            Construct::Assembler => asset_server.load("art/assembler/off.png"),
        },
        empty_item: asset_server.load("art/empty_item.png"),
    })
}
