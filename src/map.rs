use crate::{coords::map_to_screen, player::Player, textures::Textures, random::get_random_u32};
use wasm_game_lib::graphics::canvas::Canvas;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Grass,
    Air,
    Dirt,
}

pub struct Map<'a> {
    dirt_height: [isize; 200],
    textures: &'a Textures,
}

impl<'a> Map<'a> {
    pub fn new(textures: &Textures) -> Map {
        let mut map = Map {
            dirt_height: [20; 200],
            textures,
        };
        let mut height: f64 = 20.0;
        let mut slope: f64 = 0.2;
        for x in 0..200 {
            let mut random: f64 = get_random_u32() as f64 - 2_147_483_647.0;
            random /= 2_147_483_647.0;
            slope += random / 5.0;
            if slope > 1.5 {
                slope = 1.5;
            }
            if height > 40.0 && slope > -0.4 {
                log!("redressing!");
                slope -= 0.08;
            }
            if height < 10.0 && slope < 0.4 {
                log!("slowing down!");
                slope += 0.08;
            }
            height += slope;
            map.dirt_height[x] = height.floor() as isize;
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
        for x in 0..200 {
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
    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        if x > 0 {
            if let Some(height) = self.dirt_height.get(x as usize) {
                return match height {
                    height if height == &y => &Block::Grass,
                    height if height > &y => &Block::Air,
                    height if height < &y => &Block::Dirt,
                    _ => &Block::Air,
                };
            }
        }
        &Block::Air
    }
}
