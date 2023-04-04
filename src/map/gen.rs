use std::ops::RangeInclusive;

use crate::{entities::Enemy, item::Item};

use super::CHUNK_SIZE;

use vek::*;

const ITEM_CHANCE: f32 = 0.3;

#[derive(Clone, Copy)]
pub struct RandomField(pub u32);

impl RandomField {
    pub fn chance(&self, pos: Vec2<i32>, chance: f32) -> bool {
        self.gen_f32(pos) < chance
    }

    pub fn gen_f32(&self, pos: Vec2<i32>) -> f32 {
        (self.gen(pos) % (1 << 16)) as f32 / ((1 << 16) as f32)
    }

    pub fn gen_range(&self, pos: Vec2<i32>, range: RangeInclusive<u32>) -> u32 {
        self.gen(pos) % (range.end() - range.start() + 1) + range.start()
    }

    pub fn gen_bounds(&self, center: Vec2<i32>, size_range: RangeInclusive<u32>) -> Aabr<i32> {
        let width = self.gen_range(center, size_range.clone());
        let height = self.gen_range(center.map(|e| e ^ e << 5), size_range);
        let size = Vec2::new(width as i32, height as i32);
        Aabr {
            min: center - size / 2,
            max: center + size / 2 + size % 2,
        }
    }

    pub fn gen(&self, pos: Vec2<i32>) -> u32 {
        let pos = pos.as_::<u32>();

        let mut a = self.0;
        a = a.wrapping_sub(a << 3);
        a ^= pos.x;
        a ^= a.rotate_right(5);
        a = a.wrapping_mul(0xad8213d);
        a ^= a >> 13;
        a ^= pos.y;
        a = (a ^ 213) ^ a.rotate_left(15);
        a = a.wrapping_add(a >> 16);
        a ^= a.rotate_right(5);
        a
    }
}

#[derive(Default, Clone, Copy)]
pub struct StructureField {
    pos: Vec2<i32>,
    seed: u32,
}

#[derive(Clone)]
pub struct StructureGen {
    freq: u32,
    spread: u32,
    x_field: RandomField,
    y_field: RandomField,
    seed_field: RandomField,
}

impl StructureGen {
    pub fn new(seed: u32, freq: u32, spread: u32) -> Self {
        Self {
            freq,
            spread,
            x_field: RandomField(seed),
            y_field: RandomField(seed + 1),
            seed_field: RandomField(seed + 2),
        }
    }

    fn index_to_sample_internal(
        freq: i32,
        freq_offset: i32,
        spread: i32,
        spread_mul: u32,
        x_field: RandomField,
        y_field: RandomField,
        seed_field: RandomField,
        index: Vec2<i32>,
    ) -> StructureField {
        let center = index * freq + freq_offset;
        StructureField {
            pos: center
                + if spread_mul > 0 {
                    Vec2::new(
                        (x_field.gen(center) % spread_mul) as i32 - spread,
                        (y_field.gen(center) % spread_mul) as i32 - spread,
                    )
                } else {
                    Vec2::zero()
                },
            seed: seed_field.gen(center),
        }
    }

    #[inline]
    fn sample_to_index_internal(freq: i32, pos: Vec2<i32>) -> Vec2<i32> {
        pos.map(|e| e.div_euclid(freq))
    }

    #[inline]
    fn freq_offset(freq: i32) -> i32 {
        freq / 2
    }

    #[inline]
    fn spread_mul(spread: u32) -> u32 {
        spread * 2
    }

    pub fn get(&self, pos: Vec2<i32>) -> [StructureField; 9] {
        let mut samples = [StructureField::default(); 9];

        let spread = self.spread as i32;
        let spread_mul = Self::spread_mul(self.spread);
        let freq = self.freq as i32;
        let freq_offset = Self::freq_offset(freq);

        let sample_closest = Self::sample_to_index_internal(freq, pos);

        for i in 0..3 {
            for j in 0..3 {
                let index = sample_closest + Vec2::new(i as i32, j as i32) - 1;
                let sample = Self::index_to_sample_internal(
                    freq,
                    freq_offset,
                    spread,
                    spread_mul,
                    self.x_field,
                    self.y_field,
                    self.seed_field,
                    index,
                );
                samples[i * 3 + j] = sample;
            }
        }

        samples
    }

    pub fn iter_area(
        &self,
        min: Vec2<i32>,
        max: Vec2<i32>,
    ) -> impl Iterator<Item = StructureField> {
        let freq = self.freq;
        let spread = self.spread;
        let spread_mul = Self::spread_mul(spread);
        assert!(spread * 2 == spread_mul);
        assert!(spread_mul <= freq);
        let spread = spread as i32;
        let freq = freq as i32;
        let freq_offset = Self::freq_offset(freq);
        assert!(freq_offset * 2 == freq);

        let min_index = Self::sample_to_index_internal(freq, min) - 1;
        let max_index = Self::sample_to_index_internal(freq, max) + 1;
        assert!(min_index.x < max_index.x);
        // NOTE: xlen > 0
        let xlen = (max_index.x - min_index.x) as u32;
        assert!(min_index.y < max_index.y);
        // NOTE: ylen > 0
        let ylen = (max_index.y - min_index.y) as u32;
        // NOTE: Cannot fail, since every product of u32s fits in a u64.
        let len = ylen as u64 * xlen as u64;
        // NOTE: since iteration is *exclusive* for the initial range, it's fine that we
        // don't go up to the maximum value.
        // NOTE: we convert to usize first, and then iterate, because we want to make
        // sure we get a properly indexed parallel iterator that can deal with
        // the whole range at once.
        let x_field = self.x_field;
        let y_field = self.y_field;
        let seed_field = self.seed_field;
        (0..len).map(move |xy| {
            let index = min_index + Vec2::new((xy % xlen as u64) as i32, (xy / xlen as u64) as i32);
            Self::index_to_sample_internal(
                freq,
                freq_offset,
                spread,
                spread_mul,
                x_field,
                y_field,
                seed_field,
                index,
            )
        })
    }
}

#[derive(Default, Clone, Copy)]
pub enum FloorTile {
    #[default]
    Ground,
    Water,
    Concrete,
    Floor,
}

#[derive(Default, Clone, Copy)]
pub enum WallTile {
    #[default]
    None,
    Wall,
}

pub struct ChunkData {
    cpos: Vec2<i32>,
    pub floor: Vec<FloorTile>,
    pub walls: Vec<WallTile>,
    pub items: Vec<(bevy::prelude::Vec2, Item)>,
    pub enemies: Vec<(bevy::prelude::Vec2, Enemy)>,
}

impl ChunkData {
    fn new(cpos: Vec2<i32>) -> Self {
        Self {
            cpos,
            ..Default::default()
        }
    }
    #[inline]
    fn index_rpos(p: Vec2<i32>) -> usize {
        p.y as usize * CHUNK_SIZE as usize + p.x as usize
    }
    #[inline]
    fn index_wpos(&self, p: Vec2<i32>) -> Option<usize> {
        let min = self.cpos * CHUNK_SIZE as i32;
        let max = min + CHUNK_SIZE as i32;
        if (min.x..max.x).contains(&p.x) && (min.y..max.y).contains(&p.y) {
            Some(Self::index_rpos(p - min))
        } else {
            None
        }
    }
    #[inline]
    fn set_floor(&mut self, p: Vec2<i32>, t: FloorTile) {
        if let Some(index) = self.index_wpos(p) {
            self.floor[index] = t;
        }
    }

    /// Inclusive
    fn blit_floor(&mut self, min: Vec2<i32>, max: Vec2<i32>, t: FloorTile) {
        let chunk_min = self.cpos * CHUNK_SIZE as i32;
        let min = (min - chunk_min).map(|e| e.max(0));
        let max = (max - chunk_min).map(|e| e.min(CHUNK_SIZE as i32 - 1));

        if min.x <= max.x && min.y <= max.y {
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    self.floor[Self::index_rpos(Vec2::new(x, y))] = t;
                }
            }
        }
    }

    fn draw_line(&self, start: Vec2<i32>, end: Vec2<i32>, mut set: impl FnMut(usize)) {
        let chunk_min = self.cpos * CHUNK_SIZE as i32;
        if start.x == end.x {
            let x = start.x - chunk_min.x;
            if (0..CHUNK_SIZE as i32).contains(&x) {
                let min = (start.y.min(end.y) - chunk_min.y).max(0);
                let max = (start.y.max(end.y) - chunk_min.y).min(CHUNK_SIZE as i32 - 1);
                if min <= max {
                    for y in min..=max {
                        set(Self::index_rpos(Vec2::new(x, y)));
                    }
                }
            }
        } else if start.y == end.y {
            let y = start.y - chunk_min.y;
            if (0..CHUNK_SIZE as i32).contains(&y) {
                let min = (start.x.min(end.x) - chunk_min.x).max(0);
                let max = (start.x.max(end.x) - chunk_min.x).min(CHUNK_SIZE as i32 - 1);
                if min <= max {
                    for x in min..=max {
                        set(Self::index_rpos(Vec2::new(x, y)));
                    }
                }
            }
        } else {
            panic!("Not an axis aligned line");
        }
    }

    // Axis aligned line, Inclusive
    fn floor_line(&mut self, start: Vec2<i32>, end: Vec2<i32>, t: FloorTile) {
        let mut floor = std::mem::take(&mut self.floor);
        self.draw_line(start, end, |i| floor[i] = t);
        self.floor = floor;
    }

    // Axis aligned line, Inclusive
    fn wall_line(&mut self, start: Vec2<i32>, end: Vec2<i32>, t: WallTile) {
        let mut walls = std::mem::take(&mut self.walls);
        self.draw_line(start, end, |i| walls[i] = t);
        self.walls = walls;
    }

    #[inline]
    fn set_wall(&mut self, p: Vec2<i32>, t: WallTile) {
        if let Some(index) = self.index_wpos(p) {
            self.walls[index] = t;
        }
    }

    fn item(&mut self, pos: Vec2<f32>, item: Item) {
        let aabr = self.aabr();
        if aabr.contains_point(pos.as_()) {
            self.items
                .push((bevy::prelude::Vec2::from_array(pos.into_array()), item));
        }
    }

    fn enemy(&mut self, pos: Vec2<f32>, enemy: Enemy) {
        let aabr = self.aabr();
        if aabr.contains_point(pos.as_()) {
            self.enemies
                .push((bevy::prelude::Vec2::from_array(pos.into_array()), enemy));
        }
    }

    fn aabr(&self) -> Aabr<i32> {
        let min = self.cpos * CHUNK_SIZE as i32;
        Aabr {
            min,
            max: min + CHUNK_SIZE as i32 - 1,
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            cpos: Vec2::zero(),
            floor: vec![FloorTile::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize],
            walls: vec![WallTile::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize],
            items: Default::default(),
            enemies: Default::default(),
        }
    }
}

const LAKES_SEED: u32 = 1001;
const STRUCTURES_SEED: u32 = 1002;

pub fn gen_chunk(cpos: bevy::prelude::IVec2, seed: u32) -> ChunkData {
    let cpos = Vec2::from(cpos.to_array());

    let mut chunk = ChunkData::new(cpos);
    let chunk_aabr = chunk.aabr();
    let min = cpos * CHUNK_SIZE as i32;
    let max = min + CHUNK_SIZE as i32;
    let lakes = StructureGen::new(seed.wrapping_add(LAKES_SEED), 50, 24);

    struct Lake {
        bounds: Aabr<i32>,
        seed: u32,
    }

    let lakes = lakes
        .iter_area(min - CHUNK_SIZE as i32 * 2, max + CHUNK_SIZE as i32 * 2)
        .map(|structure| Lake {
            bounds: RandomField(structure.seed).gen_bounds(structure.pos, 10..=24),
            seed: structure.seed,
        })
        .collect::<Vec<_>>();

    for lake in lakes
        .iter()
        .filter(|lake| lake.bounds.intersection(chunk_aabr).is_valid())
    {
        let b = lake.bounds.as_::<f32>();
        let field = RandomField(lake.seed);
        for y in lake.bounds.min.y..=lake.bounds.max.y {
            let t_y = (y as f32 - b.center().y) / b.half_size().h;
            let t_x = (1.0 - t_y * t_y).sqrt();
            let x = t_x * b.half_size().w;
            let offset0 = field.gen_f32(Vec2::new(0, y)) * 2.0 - 1.0;
            let offset1 = field.gen_f32(Vec2::new(0, -1 - y)) * 2.0 - 1.0;

            let start = Vec2::new((b.center().x - x + offset0) as i32, y);
            let end = Vec2::new((b.center().x + x + offset1) as i32, y);

            chunk.floor_line(start, end, FloorTile::Water);
        }

        let mut i = 1;
        while field.chance(Vec2::new(i, 0), ITEM_CHANCE) {
            i += 1;

            let p = Vec2::new(1, -1).map(|i| field.gen_f32(Vec2::new((1 + i) * i, 0)))
                * (b.max - b.min)
                + b.min;
            chunk.enemy(p, Enemy::Slime);
        }
    }

    let structures = StructureGen::new(seed.wrapping_add(STRUCTURES_SEED), 40, 20);

    for structure in structures.iter_area(min - CHUNK_SIZE as i32 * 2, max + CHUNK_SIZE as i32 * 2)
    {
        let bounds = RandomField(structure.seed).gen_bounds(structure.pos, 8..=15);
        if !bounds.intersection(chunk_aabr).is_valid()
            || lakes
                .iter()
                .any(|lake| lake.bounds.intersection(bounds).is_valid())
        {
            continue;
        }

        let field = RandomField(structure.seed);

        let min_max = Vec2::new(bounds.min.x, bounds.max.y);
        let max_min = Vec2::new(bounds.max.x, bounds.min.y);

        chunk.blit_floor(bounds.min, bounds.max, FloorTile::Concrete);
        chunk.wall_line(bounds.min, min_max, WallTile::Wall);
        chunk.wall_line(bounds.min, max_min, WallTile::Wall);
        chunk.wall_line(bounds.max, min_max, WallTile::Wall);
        chunk.wall_line(bounds.max, max_min, WallTile::Wall);

        let door = ([
            Vec2::new(bounds.min.x, bounds.center().y),
            Vec2::new(bounds.max.x, bounds.center().y),
            Vec2::new(bounds.center().x, bounds.min.y),
            Vec2::new(bounds.center().x, bounds.max.y),
        ])[field.gen_range(structure.pos, 0..=3) as usize];

        chunk.set_wall(door, WallTile::None);

        let mut i = 1;
        let b = bounds.as_::<f32>();
        while field.chance(Vec2::new(i, 0), ITEM_CHANCE) {
            i += 1;

            let item: Item = ([
                Item::Circuit,
                Item::Metal,
                Item::CannedFood,
                Item::Plant,
                Item::FuelTank,
            ])[field.gen_range(Vec2::new(0, i), 0..=4) as usize];

            let p = Vec2::new(1, -1).map(|i| field.gen_f32(Vec2::new((1 + i) * i, 0)))
                * (b.max - b.min - 2.0)
                + b.min
                + 1.0;
            chunk.item(p, item);
        }
    }

    chunk
}
