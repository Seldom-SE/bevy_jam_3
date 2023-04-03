use bevy::ecs::system::SystemState;
use enum_map::Enum;

use crate::{
    asset::GameAssets,
    item::{remove_item_at, Inventory, InventorySlot, Item},
    player::Player,
    prelude::*,
};

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

        let construct = (
            SpriteBundle {
                texture: assets.constructs[construct].clone(),
                transform: Transform::from_translation(transform.translation)
                    .with_scale(Vec2::splat(CONSTRUCT_SCALE).extend(1.)),
                ..default()
            },
            construct,
        );

        remove_item_at(slot, &mut slots, inventory.single());

        world.spawn(construct);
    }
}
