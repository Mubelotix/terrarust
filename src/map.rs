use crate::{
    coords::{map_to_screen, x_to_biome, x_to_chunk, x_to_chunk_and_column},
    player::Player,
    textures::{Textures, get_texture_idx},
    blocks::{Block, BlockType, NaturalBackground},
};
use arr_macro::arr;
use std::{hash::Hasher, rc::Rc};
use twox_hash::XxHash32;
use wasm_game_lib::{graphics::canvas::Canvas, log};

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
    #[allow(clippy::eval_order_dependence)]
    pub fn generate(height: &mut f64, slope: &mut f64, left_to_right: bool, mut x: isize) -> Chunk {
        let begin_config: (f64, f64) = (*height, *slope);
        let biome = x_to_biome(x);
        log!("generating {:?}", biome);

        let mut blocks = arr!({
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

            if (*height < biome.get_height().start - 10.0 && *slope < -0.4) || *height < biome.get_height().start {
                *slope += biome.get_frequency() / 3.0;
            }
            if (*height > biome.get_height().end + 10.0 && *slope > 0.4) || *height > biome.get_height().end {
                *slope -= biome.get_frequency() / 3.0;
            }

            let mut hasher2 = XxHash32::with_seed(42); // to avoid generating a tree if there is a tree at the left
            hasher.write_isize(x-1);
            let tree = hash % biome.get_tree_prob() as u64 == 0 && hasher2.finish() % biome.get_tree_prob() as u64 != 0;

            *height += *slope;
            
            if left_to_right {
                x += 1;
            } else {
                x -= 1;
            }
            
            let mut column = [Block::new(BlockType::Dirt, NaturalBackground::Dirt); 2048];
            for y in 0..height.floor() as usize {
                column[y] = Block::new(BlockType::Air, NaturalBackground::Sky);
            }
            column[height.floor() as usize] = Block::new(BlockType::Grass, NaturalBackground::Dirt);
            if tree && height.floor() as usize > 0 {
                column[height.floor() as usize - 1] = Block::new(BlockType::Tree, NaturalBackground::Sky);
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

pub struct Map {
    chunks: Vec<(Chunk, Canvas)>,
    first_chunk_number: isize,
    first_block: usize,
    textures: Rc<Textures>,
    air: Block,
}

impl Map {
    pub fn new(textures: Rc<Textures>) -> Map {
        let mut map = Map {
            chunks: Vec::new(),
            textures,
            first_chunk_number: -5,
            first_block: 0,
            air: Block::new(BlockType::Air, NaturalBackground::Sky),
        };
        let mut height: f64 = 20.0;
        let mut slope: f64 = 0.2;
        for i in -5..5 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(32 * 16);
            chunk_canvas.set_height(100 * 16);

            map.chunks
                .push((Chunk::generate(&mut height, &mut slope, true, i * 32), chunk_canvas));
        }

        for i in 0..10 {
            map.update_chunk(i);
        }

        map
    }

    pub fn update_chunk(&mut self, chunk_index: usize) {
        for x_idx in 0..32 {
            for y_idx in 0..100 {
                let x = x_idx as isize + (chunk_index as isize + self.first_chunk_number) * 32;
                let y = y_idx + self.first_block as isize;
                let block = self[(x, y)];

                let block_texture_idx = get_texture_idx((
                    self[(x, y - 1)].can_pass_through(),
                    self[(x + 1, y)].can_pass_through(),
                    self[(x, y + 1)].can_pass_through(),
                    self[(x - 1, y)].can_pass_through()
                ));

                if block.natural_background == NaturalBackground::Dirt && (block.block_type == BlockType::Air || block_texture_idx != 0) {
                    let texture_idx = get_texture_idx((
                        self[(x, y - 1)].natural_background == NaturalBackground::Sky,
                        self[(x + 1, y)].natural_background == NaturalBackground::Sky,
                        self[(x, y + 1)].natural_background == NaturalBackground::Sky,
                        self[(x - 1, y)].natural_background == NaturalBackground::Sky,
                    ));
                    self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.background_dirt.get_html_element(), texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, x_idx as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                }

                match block.block_type {
                    BlockType::Air => (),
                    BlockType::Grass => {
                        self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.grass.get_html_element(), block_texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, x_idx as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                    }
                    BlockType::Dirt => {
                        self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.dirt.get_html_element(), block_texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, x_idx as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                    },
                    BlockType::Tree => {
                        self.chunks[chunk_index].1.draw_image((x_idx as f64 * 16.0 - 80.0, y_idx as f64 * 16.0 - 240.0), &self.textures.tree)
                    }
                }
            }
        }
    }

    pub fn update_chunks(&mut self, player: &Player) {
        let chunk_number = x_to_chunk(player.x.floor() as isize);

        let mut diff = self.first_chunk_number - chunk_number;
        while diff > -4 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(32 * 16);
            chunk_canvas.set_height(100 * 16);

            self.chunks.remove(self.chunks.len() - 1);
            let mut config = self.chunks[0].0.left_config;
            self.first_chunk_number -= 1;
            self.chunks.insert(
                0,
                (Chunk::generate(
                    &mut config.0,
                    &mut config.1,
                    false,
                    self.first_chunk_number * 32,
                ), chunk_canvas),
            );
            self.update_chunk(2);

            diff = self.first_chunk_number - chunk_number;
        }
        while diff < -4 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(32 * 16);
            chunk_canvas.set_height(100 * 16);

            self.chunks.remove(0);
            let mut config = self.chunks[self.chunks.len() - 1].0.right_config;
            self.first_chunk_number += 1;
            self.chunks.push((Chunk::generate(
                &mut config.0,
                &mut config.1,
                true,
                self.first_chunk_number * 32,
            ), chunk_canvas));
            self.update_chunk(self.chunks.len() - 2);

            diff = self.first_chunk_number - chunk_number;
        }
    }
}

impl Map {
    pub fn draw_on_canvas<'a>(
        &'a mut self,
        canvas: &'a mut Canvas,
        player: &Player,
        screen_center: (isize, isize),
    ) {
        //canvas.draw_canvas((5.0, 200.0), &self.chunks[8].1);
        for (chunk_idx, chunk_canvas) in self.chunks.iter().map(|(_a, b)| b).enumerate() {
            let (mut screen_x, mut screen_y) = map_to_screen((chunk_idx as isize + self.first_chunk_number) * 32, self.first_block as isize, &player, screen_center);
            screen_x = screen_x.floor();
            screen_y = screen_y.floor();
            
            canvas.draw_canvas((screen_x, screen_y), &chunk_canvas);
        }
    }
}

impl std::ops::Index<(isize, isize)> for Map {
    type Output = Block;

    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        let (chunk, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk - self.first_chunk_number;

        if y > 0 && chunk_index > 0 {
            if let Some(chunk) = self.chunks.get(chunk_index as usize) {
                if let Some(block) = chunk.0.blocks[column as usize].get(y as usize) {
                    return &block;
                }
            }
        }

        & Block {
            block_type: BlockType::Air,
            natural_background: NaturalBackground::Sky,
        }
    }
}

impl std::ops::IndexMut<(isize, isize)> for Map {
    fn index_mut(&mut self, (x, y): (isize, isize)) -> &mut Self::Output {
        let (chunk, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk - self.first_chunk_number;

        if y > 0 && chunk_index > 0 {
            if let Some(chunk) = self.chunks.get_mut(chunk_index as usize) {
                if let Some(block) = chunk.0.blocks[column as usize].get_mut(y as usize) {
                    return block;
                }
            }
        }

        if self.air != Block::new(BlockType::Air, NaturalBackground::Sky) {
            self.air = Block::new(BlockType::Air, NaturalBackground::Sky);
        }

        &mut self.air
    }
}