use bevy_kira_audio::AudioSource;
use enum_map::{enum_map, EnumMap};

use crate::{construct::Construct, item::Item, prelude::*};

pub fn asset_plugin(app: &mut App) {
    app.add_startup_system(load.in_base_set(StartupSet::PreStartup));
}

#[derive(Resource)]
pub struct GameAssets {
    pub items: EnumMap<Item, Handle<Image>>,
    pub empty_item: Handle<Image>,
    pub constructs: EnumMap<Construct, Handle<Image>>,
    pub generators: [Handle<Image>; 5],
    pub assemblers: [Handle<Image>; 2],
    pub turrets: [Handle<Image>; 4],
    pub assembler_sound: Handle<AudioSource>,
    pub nuclear_bullet: Handle<Image>,
    pub turret_bullet: Handle<Image>,
    pub player: [Handle<Image>; 4],
}

fn load(mut commands: Commands, asset_server: Res<AssetServer>) {
    let generator_item = asset_server.load("art/generator/generator_4.png");
    let generator_off = asset_server.load("art/generator/generator_0.png");
    let assembler_off = asset_server.load("art/assembler/off.png");
    let assembler_on = asset_server.load("art/assembler/on.png");
    let turret_left_on = asset_server.load("art/turret/left_on.png");
    let turret_left_off = asset_server.load("art/turret/left_off.png");

    commands.insert_resource(GameAssets {
        items: enum_map! {
            Item::Circuit => asset_server.load("art/circuit.png"),
            Item::Metal => asset_server.load("art/metal.png"),
            Item::CannedFood => asset_server.load("art/canned_food.png"),
            Item::Plant => asset_server.load("art/plant.png"),
            Item::FuelTank => asset_server.load("art/fuel_tank.png"),
            Item::Assembler => assembler_on.clone(),
            Item::Generator => generator_item.clone(),
            Item::Turret => turret_left_on.clone(),
        },
        empty_item: asset_server.load("art/empty_item.png"),
        constructs: enum_map! {
            Construct::Generator => generator_off.clone(),
            Construct::Assembler => assembler_off.clone(),
            Construct::Turret => turret_left_off.clone(),
        },
        generators: [
            generator_off,
            asset_server.load("art/generator/generator_1.png"),
            asset_server.load("art/generator/generator_2.png"),
            asset_server.load("art/generator/generator_3.png"),
            generator_item,
        ],
        assemblers: [assembler_off, assembler_on],
        turrets: [
            turret_left_off,
            turret_left_on,
            asset_server.load("art/turret/right_off.png"),
            asset_server.load("art/turret/right_on.png"),
        ],
        assembler_sound: asset_server.load("sound/engine.ogg"),
        nuclear_bullet: asset_server.load("art/nuclear_bullet.png"),
        turret_bullet: asset_server.load("art/turret_bullet.png"),
        player: [
            asset_server.load("art/player/north_east.png"),
            asset_server.load("art/player/north_west.png"),
            asset_server.load("art/player/south_east.png"),
            asset_server.load("art/player/south_west.png"),
        ],
    })
}
