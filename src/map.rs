use wasm_game_lib::{graphics::{drawable::Drawable, canvas::Canvas}};
use crate::textures::Textures;

#[derive(Debug, Clone, Copy)]
pub enum Block {
    Grass,
    Air,
    Dirt,
}

pub struct Map<'a> {
    blocks: [[Block; 8]; 8],
    textures: &'a Textures,
}

impl<'a> Map<'a> {
    pub fn new(textures: &Textures) -> Map {
        let mut map = Map {
            blocks: [[Block::Air; 8]; 8],
            textures
        };
        for x in 0..8 {
            for y in 7..8 {
                map.blocks[x][y] = Block::Grass;
            }
        }
        map
    }
}

impl<'a> Drawable for Map<'a> {
    fn draw_on_canvas(&self, canvas: &mut Canvas) {
        for x in 0..30 {
            for y in 0..20 {
                match self[(x, y)] {
                    Block::Air => (),
                    Block::Grass => canvas.draw_image((x as f64 * 16.0, y as f64 * 16.0), &self.textures.grass[x % 4]),
                    Block::Dirt => canvas.draw_image((x as f64 * 16.0, y as f64 * 16.0), &self.textures.dirt),
                }
            }
        }
    }
}

impl<'a> std::ops::Index<(usize, usize)> for Map<'a> {
    type Output = Block;
    
    fn index(&self, (x, y): (usize, usize)) -> &<Self as std::ops::Index<(usize, usize)>>::Output {
        if y == 10 {
            &Block::Grass
        } else if y > 10 {
            &Block::Dirt
        } else {
            &Block::Air
        }
        /*if let Some(column) = self.blocks.get(x) {
            if let Some(block) = column.get(y) {
                return &block;
            }
        }
        &Block::Air*/
    }
}