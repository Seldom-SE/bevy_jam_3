use enum_map::Enum;
use rand::distributions::Standard;

use crate::prelude::*;

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
