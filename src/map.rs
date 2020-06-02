use crate::{
    coords::{map_to_screen, x_to_biome, x_to_chunk, x_to_chunk_and_column},
    player::Player,
    random::get_random_u32,
    textures::Textures,
};
use arr_macro::arr;
use std::hash::Hasher;
use twox_hash::XxHash32;
use wasm_game_lib::graphics::canvas::Canvas;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Grass,
    Air,
    Dirt,
    Tree,
}

impl Block {
    pub fn can_pass_through(&self) -> bool {
        match self {
            Block::Grass => false,
            Block::Air => true,
            Block::Dirt => false,
            Block::Tree => true,
        }
    }
}

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
    blocks: [[Block; 2048]; 32],
    pub left_config: (f64, f64),  // (height, slope)
    pub right_config: (f64, f64), // idem
}

impl Chunk {
    #[allow(clippy::cognitive_complexity)]
    pub fn generate(height: &mut f64, slope: &mut f64, left_to_right: bool, mut x: isize) -> Chunk {
        let begin_config: (f64, f64) = (*height, *slope);
        let biome = x_to_biome(x);
        log!("generating {:?}", biome);

        let mut blocks = arr!({
            let mut random: f64 = get_random_u32() as f64 - 2_147_483_647.0;
            random /= 2_147_483_647.0;
            *slope += random * biome.get_frequency();

            if *slope > biome.get_max_slope() {
                *slope = biome.get_max_slope();
            } else if *slope < -biome.get_max_slope() {
                *slope = -biome.get_max_slope();
            }

            if (*height < biome.get_height().start - 10.0 && *slope < -0.4) || *height < biome.get_height().start {
                *slope += biome.get_frequency() / 3.0;
            }
            if (*height > biome.get_height().end + 10.0 && *slope > 0.4) || *height > biome.get_height().end {
                *slope -= biome.get_frequency() / 3.0;
            }

            let mut hasher = XxHash32::with_seed(42);
            hasher.write_isize(x);
            let mut hasher2 = XxHash32::with_seed(42); // to avoid generating a tree if there is a tree at the left
            hasher.write_isize(x-1);
            let tree = hasher.finish() % biome.get_tree_prob() as u64 == 0 && hasher2.finish() % biome.get_tree_prob() as u64 != 0;

            *height += *slope;
            
            if left_to_right {
                x += 1;
            } else {
                x -= 1;
            }
            
            let mut column = [Block::Dirt; 2048];
            for y in 0..height.floor() as usize {
                column[y] = Block::Air;
            }
            column[height.floor() as usize] = Block::Grass;
            if tree && height.floor() as usize > 0 {
                column[height.floor() as usize - 1] = Block::Tree;
            }
            column
        };32);

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

pub struct Map<'a> {
    chunks: Vec<Chunk>,
    first_chunk_number: isize,
    textures: &'a Textures,
}

impl<'a> Map<'a> {
    pub fn new(textures: &Textures) -> Map {
        let mut map = Map {
            chunks: Vec::new(),
            textures,
            first_chunk_number: -5,
        };
        let mut height: f64 = 20.0;
        let mut slope: f64 = 0.2;
        for i in -5..5 {
            map.chunks
                .push(Chunk::generate(&mut height, &mut slope, true, i * 32))
        }

        map
    }

    pub fn update_chunks(&mut self, player: &Player) {
        let chunk_number = x_to_chunk(player.x.floor() as isize);

        let mut diff = self.first_chunk_number - chunk_number;
        while diff > -4 {
            log!("move left");
            self.chunks.remove(self.chunks.len() - 1);
            let mut config = self.chunks[0].left_config;
            self.first_chunk_number -= 1;
            self.chunks.insert(
                0,
                Chunk::generate(
                    &mut config.0,
                    &mut config.1,
                    false,
                    self.first_chunk_number * 32,
                ),
            );

            diff = self.first_chunk_number - chunk_number;
        }
        while diff < -4 {
            log!("move right");
            self.chunks.remove(0);
            let mut config = self.chunks[self.chunks.len() - 1].right_config;
            self.first_chunk_number += 1;
            self.chunks.push(Chunk::generate(
                &mut config.0,
                &mut config.1,
                true,
                self.first_chunk_number * 32,
            ));

            diff = self.first_chunk_number - chunk_number;
        }
    }
}

impl<'a> Map<'a> {
    pub fn draw_on_canvas(
        &self,
        canvas: &mut Canvas,
        player: &Player,
        screen_center: (isize, isize),
    ) {
        for x in player.x.floor() as isize - 60..player.x.floor() as isize + 60 {
            for y in player.y.floor() as isize - 35..player.y.floor() as isize + 35 {
                let (xisize, yisize) =
                    map_to_screen(x as isize, y as isize, &player, screen_center);
                match self[(x, y)] {
                    Block::Air => (),
                    Block::Grass => {
                        canvas.draw_image(
                            (xisize, yisize),
                            match (
                                self[(x - 1, y)].can_pass_through(),
                                self[(x + 1, y)].can_pass_through(),
                            ) {
                                (false, true) => &self.textures.grass.2,
                                (true, false) => &self.textures.grass.1,
                                _ => &self.textures.grass.0[x as usize % 4],
                            },
                        );
                    }
                    Block::Dirt => canvas.draw_image((xisize, yisize), &self.textures.dirt),
                    Block::Tree => {
                        canvas.draw_image((xisize - 80.0, yisize - 240.0), &self.textures.tree)
                    }
                }
            }
        }
    }
}

impl<'a> std::ops::Index<(isize, isize)> for Map<'a> {
    type Output = Block;

    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        let (chunk, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk - self.first_chunk_number;

        if y > 0 && chunk_index > 0 {
            if let Some(chunk) = self.chunks.get(chunk_index as usize) {
                if let Some(block) = chunk.blocks[column as usize].get(y as usize) {
                    return block;
                }
            }
        }

        &Block::Air
    }
}
