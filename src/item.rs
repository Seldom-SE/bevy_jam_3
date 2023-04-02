use enum_map::{enum_map, Enum, EnumMap};
use rand::distributions::Standard;

use crate::{
    asset::GameAssets,
    player::{Action, Player},
    prelude::*,
};

pub fn item_plugin(app: &mut App) {
    app.init_resource::<Recipes>()
        .add_startup_system(init_inventory)
        .add_startup_system(init_recipe_menu)
        .add_system(collect_item)
        .add_system(update_item_image)
        .add_system(drop_item)
        .add_system(update_recipe_menu);
}

#[derive(Clone, Component, Copy, Enum, Eq, PartialEq)]
pub enum Item {
    Circuit,
    Metal,
    CannedFood,
    Plant,
    Generator,
    Assembler,
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

const INVENTORY_SIZE: usize = 10;

#[derive(Component, Deref, DerefMut)]
struct Inventory([Entity; INVENTORY_SIZE]);

#[derive(Component, Deref, DerefMut)]
struct InventorySlot(Option<Item>);

#[derive(Deref, DerefMut, Resource)]
struct Recipes(EnumMap<Item, Option<Vec<(Item, u8)>>>);

#[derive(Component)]
struct RecipeMenu;

#[derive(Component)]
struct Recipe(Item);

impl Default for Recipes {
    fn default() -> Self {
        Self(enum_map! {
            Item::Circuit | Item::Metal | Item::CannedFood | Item::Plant => None,
            Item::Generator => Some(vec![(Item::Circuit, 1), (Item::Metal, 2)]),
            Item::Assembler => Some(vec![(Item::Circuit, 2), (Item::Metal, 1)]),
        })
    }
}

fn init_inventory(mut commands: Commands, assets: Res<GameAssets>) {
    let inventory = Inventory([(); INVENTORY_SIZE].map(|_| {
        commands
            .spawn((
                ButtonBundle {
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
                position_type: PositionType::Absolute,
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
    if !action.just_pressed(Action::Collect) {
        return;
    }

    let player_pos = player_transform.translation.truncate();
    for (item, item_transform, item_type) in &items {
        if player_pos.distance(item_transform.translation.truncate()) >= COLLECTION_RADIUS {
            continue;
        }

        let inventory = inventory.single();
        for &slot_entity in &**inventory {
            let mut slot = slots.get_mut(slot_entity).unwrap();
            if slot.is_some() {
                continue;
            }

            **slot = Some(*item_type);
            commands.entity(item).despawn();
            break;
        }
        break;
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

fn drop_item(
    mut commands: Commands,
    mut slots: Query<(Entity, &mut InventorySlot, &Interaction)>,
    inventory: Query<&Inventory>,
    players: Query<&Transform, With<Player>>,
    mouse: Res<Input<MouseButton>>,
    assets: Res<GameAssets>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let Ok(transform) = players.get_single() else { return };

    let inventory = inventory.single();
    let mut clicked_slot = None;

    for (slot_entity, slot, &interaction) in slots.iter() {
        if interaction != Interaction::Hovered {
            continue;
        }

        let Some(item) = **slot else { return };

        commands.spawn((
            SpriteBundle {
                texture: assets.items[item].clone(),
                transform: Transform::from_translation(transform.translation),
                ..default()
            },
            item,
        ));

        clicked_slot = Some(
            inventory
                .iter()
                .position(|&slot| slot == slot_entity)
                .unwrap(),
        );
    }

    if let Some(slot) = clicked_slot {
        for curr_slot in slot..INVENTORY_SIZE - 1 {
            let [(_, mut curr_slot, _), (_, mut next_slot, _)] = slots
                .get_many_mut([inventory[curr_slot], inventory[curr_slot + 1]])
                .unwrap();

            **curr_slot = **next_slot;
            **next_slot = None;
        }
    }
}

fn init_recipe_menu(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::all(Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        },
        RecipeMenu,
    ));
}

fn update_recipe_menu(
    mut commands: Commands,
    recipe_menu: Query<Entity, With<RecipeMenu>>,
    slots: Query<Ref<InventorySlot>>,
    recipes: Res<Recipes>,
    assets: Res<GameAssets>,
) {
    if !slots.iter().any(|slot| slot.is_changed()) {
        return;
    }

    let recipe_menu = recipe_menu.single();
    commands.entity(recipe_menu).despawn_descendants();

    let recipes = recipes
        .iter()
        .filter_map(|(item, recipe)| {
            recipe.as_ref().and_then(|recipe| {
                for (ingredient, count) in recipe {
                    if slots
                        .iter()
                        .filter(|slot| ***slot == Some(*ingredient))
                        .count()
                        < *count as usize
                    {
                        return None;
                    }
                }

                Some(
                    commands
                        .spawn((
                            ButtonBundle {
                                style: Style {
                                    size: Size::all(Val::Px(64.)),
                                    ..default()
                                },
                                image: assets.items[item].clone().into(),
                                ..default()
                            },
                            Recipe(item),
                        ))
                        .id(),
                )
            })
        })
        .collect::<Vec<_>>();

    commands.entity(recipe_menu).push_children(&recipes);
}
