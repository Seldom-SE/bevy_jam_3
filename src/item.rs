use bevy::ecs::system::SystemState;
use enum_map::{enum_map, Enum, EnumMap};
use rand::distributions::Standard;

use crate::{
    asset::GameAssets,
    construct::{fuel_generator, spawn_construct, Assembler, PowerConsumer},
    player::{Action, Player},
    prelude::*,
    stats::Hunger,
};

pub fn item_plugin(app: &mut App) {
    app.init_resource::<Recipes>()
        .add_startup_system(init_inventory)
        .add_startup_system(init_recipe_menu)
        .add_system(collect_item)
        .add_system(update_item_image)
        .add_system(use_item)
        .add_system(drop_item)
        .add_system(update_recipe_menu)
        .add_system(craft_item);
}

#[derive(Clone, Component, Copy, Enum, Eq, PartialEq)]
pub enum Item {
    Circuit,
    Metal,
    CannedFood,
    Plant,
    FuelTank,
    Generator,
    Assembler,
    Turret,
}

impl Distribution<Item> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Item {
        *[
            Item::Circuit,
            Item::Metal,
            Item::CannedFood,
            Item::Plant,
            Item::FuelTank,
        ]
        .choose(rng)
        .unwrap()
    }
}

const INVENTORY_SIZE: usize = 10;

#[derive(Component, Deref, DerefMut)]
pub struct Inventory([Entity; INVENTORY_SIZE]);

#[derive(Component, Deref, DerefMut)]
pub struct InventorySlot(Option<Item>);

#[derive(Deref, DerefMut, Resource)]
struct Recipes(EnumMap<Item, Option<Vec<(Item, u8)>>>);

#[derive(Component)]
struct RecipeMenu;

#[derive(Component, Deref, DerefMut)]
struct Recipe(Item);

impl Default for Recipes {
    fn default() -> Self {
        Self(enum_map! {
            Item::Circuit | Item::Metal | Item::CannedFood | Item::Plant | Item::FuelTank => None,
            Item::Generator => Some(vec![(Item::Metal, 1)]),
            Item::Assembler => Some(vec![(Item::Circuit, 1)]),
            Item::Turret => Some(vec![(Item::Circuit, 1), (Item::Metal, 1)]),
        })
    }
}

fn init_inventory(mut commands: Commands, assets: Res<GameAssets>) {
    let inventory = Inventory([None; 10].map(|item| {
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
                InventorySlot(item),
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

fn add_item(item: Item, slots: &mut Query<&mut InventorySlot>, inventory: &Inventory) -> bool {
    for &slot_entity in &**inventory {
        let mut slot = slots.get_mut(slot_entity).unwrap();
        if slot.is_some() {
            continue;
        }

        **slot = Some(item);
        return true;
    }
    false
}

pub fn remove_item_at(slot: usize, slots: &mut Query<&mut InventorySlot>, inventory: &Inventory) {
    for curr_slot in slot..INVENTORY_SIZE - 1 {
        let [mut curr_slot, mut next_slot] = slots
            .get_many_mut([inventory[curr_slot], inventory[curr_slot + 1]])
            .unwrap();

        **curr_slot = **next_slot;
        **next_slot = None;
    }

    let mut last_slot = slots.get_mut(inventory[INVENTORY_SIZE - 1]).unwrap();
    **last_slot = None;
}

fn remove_item(item: Item, slots: &mut Query<&mut InventorySlot>, inventory: &Inventory) {
    for (i, &slot) in inventory.iter().enumerate() {
        let slot = slots.get(slot).unwrap();
        if **slot == Some(item) {
            remove_item_at(i, slots, inventory);
            break;
        }
    }
}

pub const INTERACT_RADIUS: f32 = 32.;

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
        if player_pos.distance_squared(item_transform.translation.truncate())
            >= INTERACT_RADIUS * INTERACT_RADIUS
        {
            continue;
        }

        let inventory = inventory.single();
        if add_item(*item_type, &mut slots, inventory) {
            commands.entity(item).despawn();
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

const CANNED_FOOD_VALUE: f32 = 0.4;

fn use_item(
    mut commands: Commands,
    interactions: Query<(Entity, &Interaction), (With<InventorySlot>, Changed<Interaction>)>,
    slots: Query<&InventorySlot>,
    inventory: Query<&Inventory>,
) {
    use Item::*;

    let Some((slot_entity, _)) = interactions.iter().find(|(_, &interaction)| {
        interaction == Interaction::Clicked
    }) else { return };

    let slot = inventory
        .single()
        .iter()
        .position(|&slot| slot == slot_entity)
        .unwrap();

    let Some(item) = **slots.get(slot_entity).unwrap() else { return };
    match item {
        FuelTank => commands.add(fuel_generator(slot)),
        Generator | Assembler | Turret => {
            commands.add(spawn_construct(slot, item.try_into().unwrap()))
        }
        CannedFood => commands.add(eat_food(slot, CANNED_FOOD_VALUE)),
        Circuit | Metal | Plant => (),
    };
}

fn drop_item(
    mut commands: Commands,
    interactions: Query<(Entity, &Interaction), With<InventorySlot>>,
    mut slots: Query<&mut InventorySlot>,
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

    for (slot_entity, &interaction) in &interactions {
        if interaction != Interaction::Hovered {
            continue;
        }

        let Some(item) = **slots.get(slot_entity).unwrap() else { return };

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
        remove_item_at(slot, &mut slots, inventory);
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

const ASSEMBLER_RANGE: f32 = 32.;

fn update_recipe_menu(
    mut commands: Commands,
    recipe_menu: Query<Entity, With<RecipeMenu>>,
    slots: Query<&InventorySlot>,
    players: Query<&Transform, With<Player>>,
    assemblers: Query<(&PowerConsumer, &Transform), With<Assembler>>,
    recipes: Res<Recipes>,
    assets: Res<GameAssets>,
) {
    let Ok(player_transform) = players.get_single() else { return };

    let recipe_menu = recipe_menu.single();
    commands.entity(recipe_menu).despawn_descendants();

    'outer: {
        for (power, assembler_transform) in &assemblers {
            if power.source.is_some()
                && player_transform
                    .translation
                    .truncate()
                    .distance_squared(assembler_transform.translation.truncate())
                    < ASSEMBLER_RANGE * ASSEMBLER_RANGE
            {
                break 'outer;
            }
        }

        return;
    }

    let recipes = recipes
        .iter()
        .filter_map(|(item, recipe)| {
            recipe.as_ref().and_then(|recipe| {
                for (ingredient, count) in recipe {
                    if slots
                        .iter()
                        .filter(|&slot| **slot == Some(*ingredient))
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

fn craft_item(
    mut slots: Query<&mut InventorySlot>,
    recipe_slots: Query<(&Recipe, &Interaction), Changed<Interaction>>,
    inventory: Query<&Inventory>,
    recipes: Res<Recipes>,
) {
    for (recipe, &interaction) in recipe_slots.iter() {
        if interaction != Interaction::Clicked {
            continue;
        }

        let inventory = inventory.single();
        let ingredients = recipes[**recipe].as_ref().unwrap();

        for (ingredient, count) in ingredients {
            for _ in 0..*count {
                remove_item(*ingredient, &mut slots, inventory);
            }
        }

        add_item(**recipe, &mut slots, inventory);
        return;
    }
}

fn eat_food(slot: usize, value: f32) -> impl Fn(&mut World) {
    move |world| {
        let mut system_state = SystemState::<(
            Query<&mut Hunger, With<Player>>,
            Query<&mut InventorySlot>,
            Query<&Inventory>,
        )>::new(world);
        let (mut players, mut slots, inventory) = system_state.get_mut(world);
        let Ok(mut hunger) = players.get_single_mut() else { return };
        **hunger += value;
        **hunger = hunger.clamp(0., 1.);

        remove_item_at(slot, &mut slots, inventory.single());
    }
}
