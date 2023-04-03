use std::char::MAX;

use bevy::ecs::system::SystemState;
use enum_map::Enum;

use crate::{
    asset::GameAssets,
    item::{remove_item_at, Inventory, InventorySlot, Item, INTERACT_RADIUS},
    player::Player,
    prelude::*,
};

pub fn construct_plugin(app: &mut App) {
    app.add_system(update_generators)
        .add_system(update_generator_sprites);
}

#[derive(Clone, Component, Copy, Enum)]
pub enum Construct {
    Generator,
    Assembler,
}

impl TryFrom<Item> for Construct {
    type Error = ();

    fn try_from(item: Item) -> Result<Self, Self::Error> {
        match item {
            Item::Generator => Ok(Construct::Generator),
            Item::Assembler => Ok(Construct::Assembler),
            _ => Err(()),
        }
    }
}

#[derive(Component)]
struct Generator {
    fuel: f32,
}

const CONSTRUCT_SPACING: f32 = 32.;
const CONSTRUCT_SCALE: f32 = 2.;

pub fn spawn_construct(slot: usize, construct: Construct) -> impl Fn(&mut World) {
    move |world: &mut World| {
        let mut system_state = SystemState::<(
            Query<&Transform, With<Player>>,
            Query<&Transform, With<Construct>>,
            Query<&mut InventorySlot>,
            Query<&Inventory>,
            Res<GameAssets>,
        )>::new(world);
        let (players, constructs, mut slots, inventory, assets) = system_state.get_mut(world);
        let Ok(transform) = players.get_single() else { return };

        for &construct_transform in &constructs {
            if transform
                .translation
                .truncate()
                .distance(construct_transform.translation.truncate())
                < CONSTRUCT_SPACING
            {
                return;
            }
        }

        let construct_bundle = (
            SpriteBundle {
                texture: assets.constructs[construct].clone(),
                transform: Transform::from_translation(transform.translation)
                    .with_scale(Vec2::splat(CONSTRUCT_SCALE).extend(1.)),
                ..default()
            },
            construct,
        );

        remove_item_at(slot, &mut slots, inventory.single());

        let mut entity = world.spawn(construct_bundle);
        if let Construct::Generator = construct {
            entity.insert(Generator { fuel: 0. });
        }
    }
}

const MAX_FUEL: f32 = 30.;

pub fn fuel_generator(slot: usize) -> impl Fn(&mut World) {
    move |world: &mut World| {
        let mut system_state = SystemState::<(
            Query<&Transform, With<Player>>,
            Query<(&mut Generator, &Transform)>,
            Query<&mut InventorySlot>,
            Query<&Inventory>,
        )>::new(world);
        let (players, mut generators, mut slots, inventory) = system_state.get_mut(world);
        let Ok(transform) = players.get_single() else { return };
        let mut generator = None;

        for (curr_generator, generator_transform) in &mut generators {
            if transform
                .translation
                .truncate()
                .distance(generator_transform.translation.truncate())
                < INTERACT_RADIUS
            {
                generator = Some(curr_generator);
                break;
            }
        }

        let Some(mut generator) = generator else { return };
        generator.fuel = MAX_FUEL;

        remove_item_at(slot, &mut slots, inventory.single());
    }
}

fn update_generators(mut generators: Query<&mut Generator>, time: Res<Time>) {
    for mut generator in &mut generators {
        if generator.fuel > 0. {
            generator.fuel -= time.delta_seconds();
        }
    }
}

fn update_generator_sprites(
    mut generators: Query<(&mut Handle<Image>, &Generator), Changed<Generator>>,
    assets: Res<GameAssets>,
) {
    for (mut sprite, generator) in &mut generators {
        *sprite = assets.generators[(generator.fuel.clamp(0., MAX_FUEL) / MAX_FUEL
            * (assets.generators.len() as f32 - 1.))
            .ceil() as usize]
            .clone();
    }
}
