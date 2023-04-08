// TODO Reintegrate day/night cycle

use std::f32::consts::TAU;

use crate::{player::Player, prelude::*, stats::Radiation};

pub fn day_night_plugin(app: &mut App) {
    app.add_startup_system(init).add_system(update);
}

const MEAN_INTENSITY: f32 = 0.5;
const INTENSITY_RANGE: f32 = 0.5;
const DAY_COLOR: Vec3 = Vec3::new(0.85, 0.85, 0.6);
const NIGHT_COLOR: Vec3 = Vec3::new(0.3, 0.45, 0.8);

fn sky_light(time: f32) -> Skylight2d {
    let strength = (MEAN_INTENSITY + INTENSITY_RANGE * time.sin()).powi(2) - 0.1;
    let color = NIGHT_COLOR.lerp(DAY_COLOR, time.sin() * 0.5 + 0.5);
    let color = Color::rgb(color.x, color.y, color.z);
    Skylight2d { color, strength }
}

fn init(mut commands: Commands) {
    commands.spawn(Skylight2d {
        color: Color::WHITE,
        strength: 1.0,
    });
}

const DAY_LENGTH: f32 = 50.;

fn update(
    time: Res<Time>,
    mut skylights: Query<&mut Skylight2d>,
    players: Query<&Radiation, With<Player>>,
) {
    let mut sky_light = sky_light((time.elapsed_seconds() / DAY_LENGTH).fract() * TAU);
    if let Ok(radiation) = players.get_single() {
        let radiation = ((**radiation * 1.3) - 0.3).max(0.);
        sky_light.strength *= 1. + radiation;
        sky_light.color.set_r(sky_light.color.r() * 1. - radiation);
        sky_light.color.set_b(sky_light.color.b() * 1. - radiation);
    }

    *skylights.single_mut() = sky_light;
}
