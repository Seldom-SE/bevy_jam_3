use std::ops::RangeInclusive;

use bevy::prelude::{warn, IVec2, Vec2};

use crate::{entities::Enemy, item::Item};

use super::CHUNK_SIZE;

const ITEM_CHANCE: f32 = 0.3;

#[derive(Clone, Copy)]
pub struct RandomField(pub u32);

impl RandomField {
    pub fn chance(&self, pos: IVec2, chance: f32) -> bool {
        self.gen_f32(pos) < chance
    }

    pub fn gen_f32(&self, pos: IVec2) -> f32 {
        (self.gen(pos) % (1 << 16)) as f32 / ((1 << 16) as f32)
    }

    pub fn gen_range(&self, pos: IVec2, range: RangeInclusive<u32>) -> u32 {
        self.gen(pos) % (range.end() - range.start() + 1) + range.start()
    }

    pub fn gen(&self, pos: IVec2) -> u32 {
        let pos = pos.as_uvec2();

        let mut a = self.0;
        a = a.wrapping_sub(a << 3);
        a ^= pos.x;
        a ^= a >> 4;
        a = a.wrapping_mul(0xad8213d);
        a ^= a >> 13;
        a ^= pos.y;
        a = (a ^ 213) ^ a.rotate_left(15);
        a = a.wrapping_add(a >> 16);
        a ^= a >> 4;
        a
    }
}

#[derive(Default, Clone, Copy)]
pub struct StructureField {
    pos: IVec2,
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
        index: IVec2,
    ) -> StructureField {
        let center = index * freq + freq_offset;
        let pos = IVec2::from(center);
        StructureField {
            pos: center
                + if spread_mul > 0 {
                    IVec2::new(
                        (x_field.gen(pos) % spread_mul) as i32 - spread,
                        (y_field.gen(pos) % spread_mul) as i32 - spread,
                    )
                } else {
                    IVec2::ZERO
                },
            seed: seed_field.gen(pos),
        }
    }

    #[inline]
    fn sample_to_index_internal(freq: i32, pos: IVec2) -> IVec2 {
        IVec2::new(pos.x.div_euclid(freq), pos.y.div_euclid(freq))
    }

    #[inline]
    fn freq_offset(freq: i32) -> i32 {
        freq / 2
    }

    #[inline]
    fn spread_mul(spread: u32) -> u32 {
        spread * 2
    }

    pub fn get(&self, pos: IVec2) -> [StructureField; 9] {
        let mut samples = [StructureField::default(); 9];

        let spread = self.spread as i32;
        let spread_mul = Self::spread_mul(self.spread);
        let freq = self.freq as i32;
        let freq_offset = Self::freq_offset(freq);

        let sample_closest = Self::sample_to_index_internal(freq, pos);

        for i in 0..3 {
            for j in 0..3 {
                let index = sample_closest + IVec2::new(i as i32, j as i32) - 1;
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

    pub fn iter_area(&self, min: IVec2, max: IVec2) -> impl Iterator<Item = StructureField> {
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
            let index =
                min_index + IVec2::new((xy % xlen as u64) as i32, (xy / xlen as u64) as i32);
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
    cpos: IVec2,
    pub floor: Vec<FloorTile>,
    pub walls: Vec<WallTile>,
    pub items: Vec<(Vec2, Item)>,
    pub enemies: Vec<(Vec2, Enemy)>,
}

impl ChunkData {
    fn new(cpos: IVec2) -> Self {
        Self {
            cpos,
            ..Default::default()
        }
    }
    #[inline]
    fn index_rpos(p: IVec2) -> usize {
        p.y as usize * CHUNK_SIZE as usize + p.x as usize
    }
    #[inline]
    fn index_wpos(&self, p: IVec2) -> Option<usize> {
        let min = self.cpos * CHUNK_SIZE as i32;
        let max = min + IVec2::splat(CHUNK_SIZE as i32);
        if (min.x..max.x).contains(&p.x) && (min.y..max.y).contains(&p.y) {
            Some(Self::index_rpos(p - min))
        } else {
            None
        }
    }
    #[inline]
    fn set_floor(&mut self, p: IVec2, t: FloorTile) {
        if let Some(index) = self.index_wpos(p) {
            self.floor[index] = t;
        }
    }

    /// Inclusive
    fn blit_floor(&mut self, min: IVec2, max: IVec2, t: FloorTile) {
        let chunk_min = self.cpos * CHUNK_SIZE as i32;
        let min = (min - chunk_min).max(IVec2::ZERO);
        let max = (max - chunk_min).min(IVec2::splat(CHUNK_SIZE as i32 - 1));

        if min.x <= max.x && min.y <= max.y {
            for y in min.y..=max.y {
                for x in min.x..=max.x {
                    self.floor[Self::index_rpos(IVec2::new(x, y))] = t;
                }
            }
        }
    }

    fn draw_line(&self, start: IVec2, end: IVec2, mut set: impl FnMut(usize)) {
        let chunk_min = self.cpos * CHUNK_SIZE as i32;
        if start.x == end.x {
            let x = start.x - chunk_min.x;
            if (0..CHUNK_SIZE as i32).contains(&x) {
                let min = (start.y.min(end.y) - chunk_min.y).max(0);
                let max = (start.y.max(end.y) - chunk_min.y).min(CHUNK_SIZE as i32 - 1);
                if min <= max {
                    for y in min..=max {
                        set(Self::index_rpos(IVec2::new(x, y)));
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
                        set(Self::index_rpos(IVec2::new(x, y)));
                    }
                }
            }
        } else {
            panic!("Not an axis aligned line");
        }
    }

    // Axis aligned line, Inclusive
    fn floor_line(&mut self, start: IVec2, end: IVec2, t: FloorTile) {
        let mut floor = std::mem::take(&mut self.floor);
        self.draw_line(start, end, |i| floor[i] = t);
        self.floor = floor;
    }

    // Axis aligned line, Inclusive
    fn wall_line(&mut self, start: IVec2, end: IVec2, t: WallTile) {
        let mut walls = std::mem::take(&mut self.walls);
        self.draw_line(start, end, |i| walls[i] = t);
        self.walls = walls;
    }

    #[inline]
    fn set_wall(&mut self, p: IVec2, t: WallTile) {
        if let Some(index) = self.index_wpos(p) {
            self.walls[index] = t;
        }
    }

    fn item(&mut self, pos: Vec2, item: Item) {
        let min = self.cpos * CHUNK_SIZE as i32;
        let max = min + IVec2::splat(CHUNK_SIZE as i32);
        let p = pos.as_ivec2();
        if (min.x..max.x).contains(&p.x) && (min.y..max.y).contains(&p.y) {
            self.items.push((pos, item));
        }
    }

    fn enemy(&mut self, pos: Vec2, enemy: Enemy) {
        let min = self.cpos * CHUNK_SIZE as i32;
        let max = min + IVec2::splat(CHUNK_SIZE as i32);
        let p = pos.as_ivec2();
        if (min.x..max.x).contains(&p.x) && (min.y..max.y).contains(&p.y) {
            self.enemies.push((pos, enemy));
        }
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            cpos: IVec2::ZERO,
            floor: vec![FloorTile::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize],
            walls: vec![WallTile::default(); (CHUNK_SIZE * CHUNK_SIZE) as usize],
            items: Default::default(),
            enemies: Default::default(),
        }
    }
}

const LAKES_SEED: u32 = 1001;
const STRUCTURES_SEED: u32 = 1002;

pub fn gen_chunk(cpos: IVec2, seed: u32) -> ChunkData {
    const CHUNK_EXTENT: IVec2 = IVec2::splat(CHUNK_SIZE as i32);
    let mut chunk = ChunkData::new(cpos);
    let min = cpos * CHUNK_SIZE as i32;
    let max = min + CHUNK_EXTENT;
    let lakes = StructureGen::new(seed.wrapping_add(LAKES_SEED), 50, 24);

    struct Lake {
        pos: IVec2,
        extent: IVec2,
        seed: u32,
    }

    let lakes = lakes
        .iter_area(min, max)
        .map(|structure| {
            let x = RandomField(structure.seed).gen_range(structure.pos, 5..=12);
            let y = RandomField(structure.seed.wrapping_add(1)).gen_range(structure.pos, 5..=12);
            Lake {
                pos: structure.pos,
                seed: structure.seed,
                extent: IVec2::new(x as i32, y as i32),
            }
        })
        .collect::<Vec<_>>();

    for lake in lakes.iter() {
        let field = RandomField(lake.seed);
        for y in -lake.extent.y..=lake.extent.y {
            let t_y = y as f32 / lake.extent.y as f32;
            let t_x = (1.0 - t_y * t_y).sqrt();
            let x = t_x * lake.extent.x as f32;
            let offset0 =
                field.gen_range(lake.pos + IVec2::new(-lake.extent.x, y), 0..=2) as i32 - 1;
            let offset1 =
                field.gen_range(lake.pos + IVec2::new(lake.extent.x, y), 0..=2) as i32 - 1;

            let start = lake.pos + IVec2::new(-x as i32 + offset0, y);
            let end = lake.pos + IVec2::new(x as i32 + offset1, y);

            chunk.floor_line(start, end, FloorTile::Water);
        }

        let mut i = 1;
        while field.chance(IVec2::new(i, 0), ITEM_CHANCE) {
            i += 1;

            let x = (field.gen_f32(IVec2::new(-i, 0)) * 2.0 - 1.0) * (lake.extent.x - 1) as f32;
            let y = (field.gen_f32(IVec2::new(0, -i)) * 2.0 - 1.0) * (lake.extent.y - 1) as f32;
            chunk.enemy(lake.pos.as_vec2() + Vec2::new(x, y), Enemy::Slime);
        }
    }

    let structures = StructureGen::new(seed.wrapping_add(STRUCTURES_SEED), 40, 15);

    for structure in structures.iter_area(min, max) {
        let field = RandomField(structure.seed);
        let x = field.gen_range(IVec2::ZERO, 4..=7);
        let y = field.gen_range(IVec2::ONE, 4..=7);
        let extent = IVec2::new(x as i32, y as i32);
        if lakes.iter().any(|lake| {
            let d = (lake.pos - structure.pos).abs();
            let e = extent + lake.extent + IVec2::ONE;
            d.x <= e.x && d.y <= e.y
        }) {
            continue;
        }

        let min = structure.pos - IVec2::new(extent.x, extent.y);
        let max = structure.pos + IVec2::new(extent.x, extent.y);

        let min_max = structure.pos + IVec2::new(-extent.x, extent.y);
        let max_min = structure.pos + IVec2::new(extent.x, -extent.y);

        chunk.blit_floor(min, max, FloorTile::Concrete);
        chunk.wall_line(min, min_max, WallTile::Wall);
        chunk.wall_line(min, max_min, WallTile::Wall);
        chunk.wall_line(max, min_max, WallTile::Wall);
        chunk.wall_line(max, max_min, WallTile::Wall);

        let door = structure.pos
            + ([
                IVec2::new(extent.x, 0),
                IVec2::new(-extent.x, 0),
                IVec2::new(0, extent.y),
                IVec2::new(0, -extent.y),
            ])[field.gen_range(structure.pos, 0..=3) as usize];
        chunk.set_wall(door, WallTile::None);

        let mut i = 1;
        while field.chance(IVec2::new(i, 0), ITEM_CHANCE) {
            i += 1;

            let item: Item = ([
                Item::Circuit,
                Item::Metal,
                Item::CannedFood,
                Item::Plant,
                Item::FuelTank,
            ])[field.gen_range(IVec2::new(0, i), 0..=4) as usize];

            let x = (field.gen_f32(IVec2::new(-i, 0)) * 2.0 - 1.0) * (extent.x - 1) as f32;
            let y = (field.gen_f32(IVec2::new(0, -i)) * 2.0 - 1.0) * (extent.y - 1) as f32;
            chunk.item(structure.pos.as_vec2() + Vec2::new(x, y), item);
        }
    }

    chunk
}
