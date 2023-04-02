use enum_map::Enum;
use rand::distributions::Standard;

use crate::{
    asset::GameAssets,
    player::{Action, Player},
    prelude::*,
};

pub fn item_plugin(app: &mut App) {
    app.add_startup_system(init_inventory)
        .add_system(collect_item)
        .add_system(update_item_image);
}

#[derive(Clone, Component, Copy, Enum)]
pub enum Item {
    Circuit,
    Metal,
    CannedFood,
    Plant,
}

impl Distribution<Item> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Item {
        match rng.gen_range(0..4) {
            0 => Item::Circuit,
            1 => Item::Metal,
            2 => Item::CannedFood,
            3 => Item::Plant,
            _ => unreachable!(),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Inventory([Entity; 10]);

#[derive(Component, Deref, DerefMut)]
struct InventorySlot(Option<Item>);

fn init_inventory(mut commands: Commands, assets: Res<GameAssets>) {
    let inventory = Inventory([(); 10].map(|_| {
        commands
            .spawn((
                ImageBundle {
                    style: Style {
                        size: Size::all(Val::Px(64.)),
                        ..default()
                    },
                    image: assets.empty_item.clone().into(),
                    ..default()
                },
                InventorySlot(None),
            ))
            .id()
    }));

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::all(Val::Percent(100.)),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .push_children(&*inventory)
        .insert(inventory);
}

const COLLECTION_RADIUS: f32 = 32.;
fn collect_item(
    mut commands: Commands,
    players: Query<(&Transform, &ActionState<Action>), With<Player>>,
    items: Query<(Entity, &Transform, &Item)>,
    inventory: Query<&Inventory>,
    mut slots: Query<&mut InventorySlot>,
) {
    let Ok((player_transform, action)) = players.get_single() else { return };
    if action.just_pressed(Action::Collect) {
        let player_pos = player_transform.translation.truncate();
        for (item, item_transform, item_type) in &items {
            if player_pos.distance(item_transform.translation.truncate()) < COLLECTION_RADIUS {
                let inventory = inventory.single();
                for &slot_entity in &**inventory {
                    let mut slot = slots.get_mut(slot_entity).unwrap();
                    if slot.is_none() {
                        **slot = Some(*item_type);
                        commands.entity(item).despawn();
                        break;
                    }
                }
                break;
            }
        }
    }
}

fn update_item_image(
    mut slots: Query<(&mut UiImage, &InventorySlot), Changed<InventorySlot>>,
    assets: Res<GameAssets>,
) {
    for (mut image, slot) in &mut slots {
        image.texture = match **slot {
            Some(item) => assets.items[item].clone(),
            None => assets.empty_item.clone(),
        };
    }
}
