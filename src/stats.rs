use std::f32::EPSILON;

use bevy::prelude::*;
use enum_map::{Enum, EnumMap};

use crate::player::Player;

#[derive(Enum, Clone, Copy, PartialEq, Eq)]
pub enum Stat {
    Speed,
    Health,
    Sight,
    RadiationResistence,
}

#[derive(Component, Default)]
pub struct Stats {
    base_stats: EnumMap<Stat, f32>,
    current_stats: EnumMap<Stat, f32>,
    buffs: Vec<Buff>,
}

impl Stats {
    pub fn new(stats: EnumMap<Stat, f32>) -> Self {
        Stats {
            base_stats: stats,
            current_stats: stats,
            buffs: vec![],
        }
    }
    pub fn get(&self, stat: Stat) -> f32 {
        self.current_stats[stat]
    }

    pub fn calc_radiation_damage(&self, f: f32) -> f32 {
        f / (1.0 + self.get(Stat::RadiationResistence))
    }

    pub fn calc_damage(&self, f: f32) -> f32 {
        f / self.get(Stat::Health)
    }
}

#[derive(Component, Deref, DerefMut, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Health(f32);

impl Default for Health {
    fn default() -> Self {
        Health(1.0)
    }
}

#[derive(Component, Deref, DerefMut, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Hunger(f32);

impl Default for Hunger {
    fn default() -> Self {
        Hunger(1.0)
    }
}

#[derive(Component, Deref, DerefMut, Reflect, FromReflect)]
#[reflect(Component)]
pub struct Radiation(f32);

impl Default for Radiation {
    fn default() -> Self {
        Radiation(0.)
    }
}

#[derive(Component)]
pub struct RadiationSource {
    pub strength: f32,
    pub radius: f32,
    pub active: bool,
}

pub fn stat_propegation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Stats, &mut Health, &Hunger, &mut Radiation)>,
    time: Res<Time>,
) {
    // Calculate stats for the frame
    for (_, mut stats, mut health, _, mut radiation) in query.iter_mut() {
        let mut new_stats = stats.base_stats;

        let buffs = stats.buffs.clone();
        stats.buffs.clear();
        for mut buff in buffs.into_iter() {
            let t = time.delta_seconds().min(buff.total_time - buff.time);
            buff.time += time.delta_seconds();

            let tick_part = t / buff.total_time;
            buff.effect.apply(
                &mut new_stats,
                &stats,
                &mut health,
                &mut radiation,
                tick_part,
            );

            if buff.time <= buff.total_time {
                stats.buffs.push(buff)
            }
        }
        stats.current_stats = new_stats;
    }
    // Do radiation damage
    for (_, stats, mut health, hunger, mut radiation) in query.iter_mut() {
        if radiation.0 > 0.8 {
            health.0 -= stats
                .calc_damage(
                    stats.calc_radiation_damage((radiation.0 - 0.8) * time.delta_seconds()),
                )
                .max(0.);
        }

        if hunger.0 <= EPSILON {
            health.0 -= stats.calc_damage(time.delta_seconds());
        }

        if radiation.0 > 0.0 {
            radiation.0 -= time.delta_seconds() * 0.003;
        }
        radiation.0 = radiation.0.clamp(0.0, 1.0);
    }

    // Kill and adjust health.
    for (entity, _, mut health, _, _) in query.iter_mut() {
        if health.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
        health.0 = health.0.clamp(0.0, 1.0);
    }
}

#[derive(Clone, Copy)]
pub enum EffectKind {
    Mul(Stat),
    Add(Stat),
    Health,
    Radiation,
}

#[derive(Clone)]
pub struct Effect {
    kind: EffectKind,
    strength: f32,
}

impl Effect {
    fn apply(
        &self,
        new_stats: &mut EnumMap<Stat, f32>,
        stats: &Stats,
        health: &mut Health,
        radiation: &mut Radiation,
        tick_part: f32,
    ) {
        match self.kind {
            EffectKind::Mul(stat) => new_stats[stat] *= self.strength,
            EffectKind::Add(stat) => new_stats[stat] += self.strength,
            EffectKind::Health => health.0 += stats.calc_damage(self.strength * tick_part),
            EffectKind::Radiation => {
                radiation.0 += stats.calc_radiation_damage(self.strength * tick_part)
            }
        }
    }
}
#[derive(Clone)]
pub struct Buff {
    effect: Effect,
    total_time: f32,
    time: f32,
}

#[derive(Bundle, Default)]
pub struct StatBundle {
    pub stats: Stats,
    pub health: Health,
    pub hunger: Hunger,
    pub radiation: Radiation,
}

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct HungerBar;

#[derive(Component)]
struct RadiationBar;

pub fn stat_plugin(app: &mut App) {
    app.add_startup_system(init_ui)
        .add_system(stat_propegation)
        .add_system(update_ui)
        .add_system(absorb_radiation)
        .add_system(get_hungry);

    app.register_type::<Health>().register_type::<Radiation>();
}

fn init_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("font/FiraSans-Bold.ttf");

    // TODO Health/radiation bars
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Percent(100.), Val::Auto),
                justify_content: JustifyContent::End,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::rgba(0., 0., 0., 0.5).into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        TextBundle::from_section(
                            "Health: 100",
                            TextStyle {
                                font: font.clone(),
                                font_size: 40.0,
                                color: Color::RED,
                            },
                        ),
                        HealthBar,
                    ));

                    parent.spawn((
                        TextBundle::from_section(
                            "Hunger: 100",
                            TextStyle {
                                font: font.clone(),
                                font_size: 40.0,
                                color: Color::ORANGE,
                            },
                        ),
                        HungerBar,
                    ));

                    parent.spawn((
                        TextBundle::from_section(
                            "Radiation: 0",
                            TextStyle {
                                font,
                                font_size: 40.0,
                                color: Color::GREEN,
                            },
                        ),
                        RadiationBar,
                    ));
                });
        });
}

fn update_ui(
    mut healths: Query<&mut Text, (With<HealthBar>, Without<HungerBar>, Without<RadiationBar>)>,
    mut hungers: Query<&mut Text, (With<HungerBar>, Without<HealthBar>, Without<RadiationBar>)>,
    mut radiations: Query<&mut Text, (With<RadiationBar>, Without<HealthBar>, Without<HungerBar>)>,
    players: Query<(&Health, &Hunger, &Radiation), With<Player>>,
) {
    let Ok((health, hunger, radiation)) = players.get_single() else { return };

    let mut health_text = healths.single_mut();
    health_text.sections[0].value = format!("Health: {}", (health.0 * 100.).ceil());

    let mut hunger_text = hungers.single_mut();
    hunger_text.sections[0].value = format!("Food: {}", (hunger.0 * 100.).ceil());

    let mut radiation_text = radiations.single_mut();
    radiation_text.sections[0].value = format!("Radiation: {}", (radiation.0 * 100.).ceil());
}

fn absorb_radiation(
    mut consumers: Query<(&mut Radiation, &Stats, &Transform)>,
    sources: Query<(&RadiationSource, &Transform)>,
    time: Res<Time>,
) {
    for (mut radiation, stats, consumer_transform) in consumers.iter_mut() {
        for (source, source_transform) in sources.iter() {
            if source.active
                && source_transform
                    .translation
                    .distance_squared(consumer_transform.translation)
                    < source.radius.powi(2)
            {
                **radiation +=
                    source.strength / stats.get(Stat::RadiationResistence) * time.delta_seconds();
                **radiation = radiation.clamp(0., 1.);
            }
        }
    }
}

const HUNGER_RATE: f32 = 0.005;

fn get_hungry(mut hungers: Query<&mut Hunger>, time: Res<Time>) {
    for mut hunger in hungers.iter_mut() {
        **hunger -= HUNGER_RATE * time.delta_seconds();
        **hunger = hunger.clamp(0., 1.);
    }
}
