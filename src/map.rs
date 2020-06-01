use crate::{coords::map_to_screen, player::Player, textures::Textures};
use wasm_game_lib::graphics::canvas::Canvas;

#[derive(Debug, Clone, Copy, PartialEq)]
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
            textures,
        };
        for x in 0..8 {
            for y in 7..8 {
                map.blocks[x][y] = Block::Grass;
            }
        }
        map
    }
}

impl<'a> Map<'a> {
    pub fn draw_on_canvas(
        &self,
        canvas: &mut Canvas,
        player: &Player,
        screen_center: (isize, isize),
    ) {
        for x in -50..50 {
            for y in 0..50 {
                let (xisize, yisize) =
                    map_to_screen(x as isize, y as isize, &player, screen_center);
                match self[(x, y)] {
                    Block::Air => (),
                    Block::Grass => {
                        canvas.draw_image((xisize, yisize), &self.textures.grass[x as usize % 4])
                    }
                    Block::Dirt => canvas.draw_image((xisize, yisize), &self.textures.dirt),
                }
            }
        }
    }
}

impl<'a> std::ops::Index<(isize, isize)> for Map<'a> {
    type Output = Block;

    #[allow(clippy::comparison_chain)]
    fn index(&self, (_x, y): (isize, isize)) -> &Self::Output {
        if y == 8 {
            &Block::Grass
        } else if y > 8 {
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
