use crate::{coords::map_to_screen, player::Player, textures::Textures, random::get_random_u32};
use wasm_game_lib::graphics::canvas::Canvas;
use std::hash::Hasher;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Grass,
    Air,
    Dirt,
    Tree,
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
        for x in 20..200 {
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
        for x in player.x.floor() as isize - 60..player.x.floor() as isize + 60 {
            for y in player.y.floor() as isize - 35..player.y.floor() as isize + 35 {
                let mut tree = false;
                if y > 10 && y < 40 && self[(x, y+1)] == Block::Grass {
                    let mut hasher = twox_hash::XxHash64::with_seed(565);
                    hasher.write_isize(x);
                    if hasher.finish() % 10 == 0 {
                        tree = true;
                    }
                }
                let (xisize, yisize) =
                    map_to_screen(x as isize, y as isize, &player, screen_center);
                if tree {
                    canvas.draw_image((xisize - 80.0, yisize - 240.0), &self.textures.tree)
                } else {
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
}

impl<'a> std::ops::Index<(isize, isize)> for Map<'a> {
    type Output = Block;

    #[allow(clippy::comparison_chain)]
    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        if x == 5 && y == 19 {
            return &Block::Tree;
        }
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
