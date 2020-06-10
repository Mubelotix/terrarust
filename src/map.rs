use crate::{
    coords::{map_to_screen, x_to_biome, x_to_chunk, x_to_chunk_and_column},
    player::Player,
    textures::Textures,
    blocks::Block,
};
use arr_macro::arr;
use std::hash::Hasher;
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
    air: Block,
}

impl<'a> Map<'a> {
    pub fn new(textures: &Textures) -> Map {
        let mut map = Map {
            chunks: Vec::new(),
            textures,
            first_chunk_number: -5,
            air: Block::Air,
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
        let coords = (player.x.floor() as isize - 60, player.y.floor() as isize - 35);
        let (mut screen_x, mut screen_y) =
            map_to_screen(coords.0, coords.1, &player, screen_center);
        screen_x = screen_x.floor();
        screen_y = screen_y.floor();

        for x in coords.0..coords.0 + 120 {
            for y in coords.1..coords.1 + 70 {
                let screen_x = screen_x + (x - coords.0) as f64 * 16.0;
                let screen_y = screen_y + (y - coords.1) as f64 * 16.0;

                match self[(x, y)] {
                    Block::Air => (),
                    Block::Grass => {
                        let mut texture_idx = 0b0000_0000;
                        if self[(x, y - 1)].can_pass_through() {
                            texture_idx |= 0b0000_1000;
                        }
                        if self[(x + 1, y)].can_pass_through() {
                            texture_idx |= 0b0000_0100;
                        }
                        if self[(x, y + 1)].can_pass_through() {
                            texture_idx |= 0b0000_0010;
                        }
                        if self[(x - 1, y)].can_pass_through() {
                            texture_idx |= 0b0000_0001;
                        }

                        canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.grass.get_html_element(), texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, screen_x, screen_y, 16.0, 16.0).unwrap();
                    }
                    Block::Dirt => {
                        let mut texture_idx = 0b0000_0000;
                        if self[(x, y - 1)].can_pass_through() {
                            texture_idx |= 0b0000_1000;
                        }
                        if self[(x + 1, y)].can_pass_through() {
                            texture_idx |= 0b0000_0100;
                        }
                        if self[(x, y + 1)].can_pass_through() {
                            texture_idx |= 0b0000_0010;
                        }
                        if self[(x - 1, y)].can_pass_through() {
                            texture_idx |= 0b0000_0001;
                        }

                        canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.dirt.get_html_element(), texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, screen_x, screen_y, 16.0, 16.0).unwrap();
                    },
                    Block::Tree => {
                        canvas.draw_image((screen_x - 80.0, screen_y - 240.0), &self.textures.tree)
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

impl<'a> std::ops::IndexMut<(isize, isize)> for Map<'a> {
    fn index_mut(&mut self, (x, y): (isize, isize)) -> &mut Self::Output {
        let (chunk, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk - self.first_chunk_number;

        if y > 0 && chunk_index > 0 {
            if let Some(chunk) = self.chunks.get_mut(chunk_index as usize) {
                if let Some(block) = chunk.blocks[column as usize].get_mut(y as usize) {
                    return block;
                }
            }
        }

        if self.air != Block::Air {
            self.air = Block::Air;
        }

        &mut self.air
    }
}