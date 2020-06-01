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
        for x in 0..30 {
            for y in 0..20 {
                let (xisize, yisize) =
                    map_to_screen(x as isize, y as isize, &player, screen_center);
                match self[(x, y)] {
                    Block::Air => (),
                    Block::Grass => canvas
                        .draw_image((xisize as f64, yisize as f64), &self.textures.grass[x as usize % 4]),
                    Block::Dirt => {
                        canvas.draw_image((xisize as f64, yisize as f64), &self.textures.dirt)
                    }
                }
            }
        }
    }
}

impl<'a> std::ops::Index<(isize, isize)> for Map<'a> {
    type Output = Block;

    #[allow(clippy::comparison_chain)]
    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        if x == 25 || x == 0 {
            return &Block::Dirt;
        }
        if x == 24 && y >= 2 {
            return &Block::Dirt;
        }
        if x == 23 && y >= 4 {
            return &Block::Dirt;
        }
        if x == 22 && y >= 6 {
            return &Block::Dirt;
        }
        if x > 0 && x < 4 && y == 0 {
            return &Block::Dirt;
        }
        if x > 6 && x < 9 && y == 0 {
            return &Block::Dirt;
        }
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
