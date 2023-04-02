use bevy::prelude::*;
use enum_map::{Enum, EnumMap};

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

#[derive(Component)]
pub struct Health(f32);

impl Default for Health {
    fn default() -> Self {
        Health(1.0)
    }
}

#[derive(Component)]
pub struct Radiation(f32);

impl Default for Radiation {
    fn default() -> Self {
        Radiation(1.0)
    }
}

fn stat_propegation(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Stats, &mut Health, &mut Radiation)>,
    time: Res<Time>,
) {
    // Calculate stats for the frame
    for (_, mut stats, mut health, mut radiation) in query.iter_mut() {
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
    for (_, stats, mut health, mut radiation) in query.iter_mut() {
        if radiation.0 > 0.8 {
            health.0 -= stats.calc_damage((radiation.0 - 0.8) * time.delta_seconds());
        } else if radiation.0 > 0.0 {
            radiation.0 -= time.delta_seconds() * 0.1;
            radiation.0 = radiation.0.max(0.0);
        }
        radiation.0 = radiation.0.clamp(0.0, 1.0);
    }

    // Kill and adjust health.
    for (entity, _, mut health, _) in query.iter_mut() {
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
    pub radiation: Radiation,
}

pub fn stat_plugin(app: &mut App) {
    app.add_system(stat_propegation);
}
