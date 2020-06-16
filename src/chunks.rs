use crate::{blocks::{NaturalBackground, BlockType, Block}, coords::x_to_biome};
use arr_macro::arr;
use std::hash::Hasher;
use twox_hash::XxHash32;

#[derive(Debug)]
pub enum Biome {
    Hills,
    Grassland,
    TemperateBroadleafForest,
}

impl Biome {
    fn get_frequency(&self) -> f64 {
        match self {
            Biome::Hills => 0.2,
            Biome::Grassland => 0.06,
            Biome::TemperateBroadleafForest => 0.08,
        }
    }

    fn get_max_slope(&self) -> f64 {
        match self {
            Biome::Hills => 0.9,
            Biome::Grassland => 0.5,
            Biome::TemperateBroadleafForest => 0.7,
        }
    }

    fn get_height(&self) -> std::ops::Range<f64> {
        match self {
            Biome::Hills => (30.0..40.0),
            Biome::Grassland => (30.0..40.0),
            Biome::TemperateBroadleafForest => (30.0..40.0),
        }
    }

    fn get_tree_prob(&self) -> u16 {
        match self {
            Biome::Hills => 50,
            Biome::Grassland => 32,
            Biome::TemperateBroadleafForest => 10,
        }
    }
}

pub struct Chunk {
    pub blocks: Vec<[Block; 2048]>,   // 32
    pub left_config: (f64, f64),  // (height, slope)
    pub right_config: (f64, f64), // idem
}

impl Chunk {
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::eval_order_dependence)]
    pub fn generate(height: &mut f64, slope: &mut f64, left_to_right: bool, mut x: isize) -> Chunk {
        let begin_config: (f64, f64) = (*height, *slope);
        let biome = x_to_biome(x);

        let mut blocks = Vec::new();
        for _idx in 0..32 {
            let mut hasher = XxHash32::with_seed(42);
            hasher.write_isize(x);
            let hash = hasher.finish();

            let mut random: f64 = hash as f64 - 2_147_483_647.0;
            random /= 2_147_483_647.0;
            *slope += random * biome.get_frequency();

            if *slope > biome.get_max_slope() {
                *slope = biome.get_max_slope();
            } else if *slope < -biome.get_max_slope() {
                *slope = -biome.get_max_slope();
            }

            if (*height < biome.get_height().start - 10.0 && *slope < -0.4)
                || *height < biome.get_height().start
            {
                *slope += biome.get_frequency() / 3.0;
            }
            if (*height > biome.get_height().end + 10.0 && *slope > 0.4)
                || *height > biome.get_height().end
            {
                *slope -= biome.get_frequency() / 3.0;
            }

            let hasher2 = XxHash32::with_seed(42); // to avoid generating a tree if there is a tree at the left // does not work
            hasher.write_isize(x - 1);
            let tree = hash % biome.get_tree_prob() as u64 == 0
                && hasher2.finish() % biome.get_tree_prob() as u64 != 0;

            *height += *slope;

            if left_to_right {
                x += 1;
            } else {
                x -= 1;
            }

            let mut column = arr!(Block{block_type: BlockType::Dirt, natural_background: NaturalBackground::Dirt, light: 0, water: 0}; 2048);
            for block in column.iter_mut().take(height.floor() as usize) {
                *block = Block {
                    block_type: BlockType::Air,
                    natural_background: NaturalBackground::Sky,
                    light: 0,
                    water: 0,
                };
            }
            column[height.floor() as usize] = Block {
                block_type: BlockType::Grass,
                natural_background: NaturalBackground::Dirt,
                light: 0,
                water: 0,
            };
            if tree && height.floor() as usize > 0 {
                column[height.floor() as usize - 1] = Block {
                    block_type: BlockType::Tree,
                    natural_background: NaturalBackground::Dirt,
                    light: 0,
                    water: 0,
                };
            }

            x -= 1;
            if x == 8 {
                column[9].water = 5;
                column[10].water = 16;
            }
            if x == 9 {
                column[9].water = 10;
                column[10].water = 16;
            }
            if x == 10 {
                wasm_game_lib::log!("set x 10 y 9 to w 4");
                column[9].water = 4;
                column[10].water = 10;
            }
            x += 1;

            blocks.push(column)
        }

        if !left_to_right {
            blocks.reverse();
        }

        Chunk {
            blocks,
            left_config: if left_to_right {
                begin_config
            } else {
                (*height, *slope)
            },
            right_config: if !left_to_right {
                begin_config
            } else {
                (*height, *slope)
            },
        }
    }
}
