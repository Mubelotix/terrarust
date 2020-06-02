use crate::{coords::map_to_screen, player::Player, textures::Textures, random::get_random_u32};
use wasm_game_lib::graphics::canvas::Canvas;
use std::hash::Hasher;
use arr_macro::arr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Grass,
    Air,
    Dirt,
    Tree,
}

pub struct Chunk {
    blocks: [[Block; 2048]; 32],
    pub left_config: (f64, f64),  // (height, slope)
    pub right_config: (f64, f64), // idem
}

impl Chunk {
    #[allow(clippy::cognitive_complexity)]
    pub fn generate(height: &mut f64, slope: &mut f64, left_to_right: bool) -> Chunk {
        let begin_config: (f64, f64) = (*height, *slope);

        let mut x: isize = 0;
        let mut blocks = arr!({
            let mut random: f64 = get_random_u32() as f64 - 2_147_483_647.0;
            random /= 2_147_483_647.0;
            *slope += random / 5.0;
            if *slope > 1.5 {
                *slope = 1.5;
            }
            if *height > 40.0 && *slope > -0.4 {
                *slope -= 0.08;
            }
            if *height < 10.0 && *slope < 0.4 {
                *slope += 0.08;
            }
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
            column
        };32);

        if !left_to_right {
            blocks.reverse();
        }

        Chunk {
            blocks,
            left_config: if left_to_right {begin_config} else {(*height, *slope)},
            right_config: if !left_to_right {begin_config} else {(*height, *slope)},
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
        for _i in -5..5 {
            map.chunks.push(Chunk::generate(&mut height, &mut slope, true))
        }
        
        map
    }

    pub fn update_chunks(&mut self, player: &Player) {
        let mut chunk_number = (player.x - player.x % 32.0) as isize;
        chunk_number /= 32;

        let mut diff = self.first_chunk_number - chunk_number;
        while diff > -4 {
            log!("move left");
            self.chunks.remove(self.chunks.len() - 1);
            let mut config = self.chunks[0].left_config;
            self.chunks.insert(0, Chunk::generate(&mut config.0, &mut config.1, false));
            self.first_chunk_number -= 1;

            diff = self.first_chunk_number - chunk_number;
        }
        while diff < -4 {
            log!("move right");
            self.chunks.remove(0);
            let mut config = self.chunks[self.chunks.len() - 1].right_config;
            self.chunks.push(Chunk::generate(&mut config.0, &mut config.1, true));
            self.first_chunk_number += 1;

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
                        canvas.draw_image((xisize, yisize), match (self[(x-1, y)] != Block::Air, self[(x+1, y)] != Block::Air) {
                            (true, false) => &self.textures.grass.2,
                            (false, true) => &self.textures.grass.1,
                            _ => &self.textures.grass.0[x as usize % 4],
                        });
                    }
                    Block::Dirt => canvas.draw_image((xisize, yisize), &self.textures.dirt),
                    Block::Tree => canvas.draw_image((xisize - 80.0, yisize - 240.0), &self.textures.tree),
                }
            }
        }
    }
}

impl<'a> std::ops::Index<(isize, isize)> for Map<'a> {
    type Output = Block;

    #[allow(clippy::comparison_chain)]
    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        let mut column_index = x % 32;
    
        let mut chunk_number = (x - column_index) / 32;
        if x < 0 && column_index < 0 {
            chunk_number -= 1;
            column_index += 32;
        }
        let chunk_index = chunk_number - self.first_chunk_number;

        if y > 0 && chunk_index > 0 {
            if let Some(chunk) = self.chunks.get(chunk_index as usize) {
                if let Some(block) = chunk.blocks[column_index as usize].get(y as usize) {
                    return block;
                }
            }
        }
        
        &Block::Air
    }
}