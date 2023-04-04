use std::time::Duration;

use bevy::ecs::system::SystemState;
use bevy_kira_audio::{
    prelude::AudioEmitter, Audio, AudioControl, AudioEasing, AudioInstance, AudioTween,
};
use enum_map::Enum;

use crate::{
    asset::GameAssets,
    ecs::DynBundle,
    item::{remove_item_at, Inventory, InventorySlot, Item, INTERACT_RADIUS},
    map::as_object_vec3,
    player::Player,
    prelude::*,
    stats::RadiationSource,
};

pub fn construct_plugin(app: &mut App) {
    app.add_system(update_generators)
        .add_system(update_generator_sprites)
        .add_system(set_power)
        .add_system(update_assemblers);
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

impl Construct {
    pub fn bundle(self, pos: Vec2, assets: &GameAssets) -> Box<dyn DynBundle> {
        let common = (
            SpriteBundle {
                texture: assets.constructs[self].clone(),
                transform: Transform::from_translation(as_object_vec3(pos))
                    .with_scale(Vec2::splat(CONSTRUCT_SCALE).extend(1.)),
                ..default()
            },
            self,
            AudioEmitter { instances: vec![] },
        );

        match self {
            Construct::Generator => Box::new((
                common,
                Generator::default(),
                PowerSource::default(),
                RadiationSource {
                    strength: GENERATOR_RADIATION,
                    active: false,
                },
            )) as Box<dyn DynBundle>,
            Construct::Assembler => Box::new((common, Assembler, PowerConsumer::default())),
        }
    }
}

#[derive(Component, Default)]
struct Generator {
    fuel: f32,
}

#[derive(Component)]
pub struct Assembler;

#[derive(Component, Default)]
pub struct PowerConsumer {
    pub source: Option<Entity>,
}

#[derive(Component, Default, Deref, DerefMut)]
pub struct PowerSource(bool);

const CONSTRUCT_SPACING: f32 = 32.;
const CONSTRUCT_SCALE: f32 = 2.;
const GENERATOR_RADIATION: f32 = 0.05;

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
                .distance_squared(construct_transform.translation.truncate())
                < CONSTRUCT_SPACING * CONSTRUCT_SPACING
            {
                return;
            }
        }

        let construct = construct.bundle(transform.translation.truncate(), &assets);

        remove_item_at(slot, &mut slots, inventory.single());

        construct.world_spawn(world);
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
                .distance_squared(generator_transform.translation.truncate())
                < INTERACT_RADIUS * INTERACT_RADIUS
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

fn update_generators(
    mut generators: Query<(&mut Generator, &mut PowerSource, &mut RadiationSource)>,
    time: Res<Time>,
) {
    for (mut generator, mut source, mut radiation) in &mut generators {
        if generator.fuel > 0. {
            generator.fuel -= time.delta_seconds();

            if generator.fuel <= 0. {
                **source = false;
                radiation.active = false;
            } else if !**source {
                **source = true;
                radiation.active = true;
            }
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

const POWER_RANGE: f32 = 128.;

// Optimize this if it gets laggy
fn set_power(
    sources: Query<(Entity, &Transform, &PowerSource)>,
    mut consumers: Query<(&Transform, &mut PowerConsumer)>,
) {
    'outer: for (consumer_transform, mut consumer) in &mut consumers {
        for (source, source_transform, source_power) in &sources {
            if source_transform
                .translation
                .truncate()
                .distance_squared(consumer_transform.translation.truncate())
                < POWER_RANGE * POWER_RANGE
                && **source_power
            {
                consumer.source = Some(source);
                continue 'outer;
            }
        }

        consumer.source = None;
    }
}

fn update_assemblers(
    mut consumers: Query<
        (&PowerConsumer, &mut Handle<Image>, &mut AudioEmitter),
        Changed<PowerConsumer>,
    >,
    assets: Res<GameAssets>,
    audio: Res<Audio>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    for (consumer, mut image, mut audio_emitter) in &mut consumers {
        match consumer.source {
            Some(_) if *image != assets.assemblers[1] => {
                audio_emitter
                    .instances
                    .push(audio.play(assets.assembler_sound.clone()).looped().handle());
                *image = assets.assemblers[1].clone();
            }
            None if *image != assets.assemblers[0] => {
                for instance in audio_emitter.instances.drain(..) {
                    if let Some(instance) = audio_instances.get_mut(&instance) {
                        instance.stop(AudioTween::new(
                            Duration::from_secs_f32(1.0),
                            AudioEasing::OutPowi(2),
                        ));
                    }
                }
                *image = assets.assemblers[0].clone();
            }
            _ => {}
        };
    }
}
