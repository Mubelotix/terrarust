use crate::{
    blocks::{Block, BlockType, NaturalBackground},
    coords::{map_to_screen, x_to_chunk, x_to_chunk_and_column},
    player::Player,
    textures::{get_texture_idx, Textures},
    chunks::Chunk,
};
use std::rc::Rc;
use wasm_game_lib::{graphics::canvas::Canvas, log};

pub struct Map {
    #[cfg(target_arch = "wasm32")]
    chunks: Vec<(Chunk, Canvas, Canvas)>,
    #[cfg(not(target_arch = "wasm32"))]
    chunks: Vec<(Chunk, (), ())>,
    #[cfg(target_arch = "wasm32")]
    canvas: Canvas,
    first_chunk_number: isize,
    first_block: usize,
    #[cfg(target_arch = "wasm32")]
    textures: Rc<Textures>,
    air: Block,
    to_update_chunks: Vec<usize>,
    pub light_update: Vec<(isize, isize, bool)>,
}

impl Map {
    #[cfg(target_arch = "wasm32")]
    pub fn new(textures: Rc<Textures>) -> Map {
        let mut map = Map {
            chunks: Vec::new(),
            textures,
            first_chunk_number: -5,
            first_block: 0,
            air: Block {
                block_type: BlockType::Air,
                natural_background: NaturalBackground::Sky,
                light: 0,
            },
            to_update_chunks: Vec::new(),
            canvas: Canvas::new(),
            light_update: Vec::new(),
        };
        map.canvas.set_width(32 * 16 * 9);
        map.canvas.set_height(100 * 16);
        let mut height: f64 = 20.0;
        let mut slope: f64 = 0.2;
        for i in -5..5 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(42 * 16);
            chunk_canvas.set_height(100 * 16);

            let mut light_chunk_canvas = Canvas::new();
            light_chunk_canvas.set_width(42 * 16);
            light_chunk_canvas.set_height(100 * 16);

            map.chunks.push((
                Chunk::generate(&mut height, &mut slope, true, i * 32),
                chunk_canvas,
                light_chunk_canvas,
            ));
        }

        for i in 0..10 {
            map.update_chunk(i);
        }

        map.init_lights();

        map
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update_chunk(&mut self, chunk_index: usize) {
        if chunk_index >= 10 {
            return;
        }
        self.chunks[chunk_index].1.clear();
        use wasm_bindgen::JsValue;
        self.chunks[chunk_index]
            .1
            .context
            .set_fill_style(&JsValue::from_str("rgb(135,206,235)"));
        self.chunks[chunk_index]
            .1
            .context
            .fill_rect(5.0 * 16.0, 0.0, 42.0 * 16.0, 100.0 * 16.0);

        self.chunks[chunk_index].2.clear();

        for x_idx in 0..32 {
            for y_idx in 0..100 {
                /*if self.chunks[chunk_index].0.blocks[x_idx][y_idx].light > 50 {
                    let gradient = self.chunks[chunk_index].2.context.create_radial_gradient((x_idx + 5) as f64 * 16.0 + 8.0, y_idx as f64 * 16.0 + 8.0, 0.0, (x_idx + 5) as f64 * 16.0 + 8.0, y_idx as f64 * 16.0 + 8.0, 48.0).unwrap();
                    gradient.add_color_stop(0.0, "rgba(255,255,255,1.0)").unwrap();
                    gradient.add_color_stop(0.5, "rgba(255,255,255,0.1)").unwrap();
                    gradient.add_color_stop(1.0, "rgba(255,255,255,0.0)").unwrap();
                    self.chunks[chunk_index].2.context.set_fill_style(&gradient);
                    self.chunks[chunk_index].2.context.fill_rect(0.0,0.0,42.0*16.0,100.0*16.0);
                }*/
                self.chunks[chunk_index]
                    .2
                    .context
                    .set_fill_style(&JsValue::from(format!(
                        "rgba(255,255,255,0.{:02})",
                        self.chunks[chunk_index].0.blocks[x_idx][y_idx].light
                    )));
                self.chunks[chunk_index].2.context.fill_rect(
                    (x_idx + 5) as f64 * 16.0,
                    y_idx as f64 * 16.0,
                    16.0,
                    16.0,
                );
            }
        }

        for x_idx in 0..32 {
            for y_idx in 0..100 {
                let x = x_idx as isize + (chunk_index as isize + self.first_chunk_number) * 32;
                let y = y_idx + self.first_block as isize;
                let block = &self[(x, y)];

                let block_texture_idx = get_texture_idx((
                    self[(x, y - 1)].can_pass_through(),
                    self[(x + 1, y)].can_pass_through(),
                    self[(x, y + 1)].can_pass_through(),
                    self[(x - 1, y)].can_pass_through(),
                ));

                if block.natural_background == NaturalBackground::Dirt
                    && (block.block_type == BlockType::Air || block_texture_idx != 0)
                {
                    let texture_idx = get_texture_idx((
                        self[(x, y - 1)].natural_background == NaturalBackground::Sky,
                        self[(x + 1, y)].natural_background == NaturalBackground::Sky,
                        self[(x, y + 1)].natural_background == NaturalBackground::Sky,
                        self[(x - 1, y)].natural_background == NaturalBackground::Sky,
                    ));
                    self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.background_dirt.get_html_element(), texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, (x_idx + 5) as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                }

                match block.block_type {
                    BlockType::Air => (),
                    BlockType::Grass => {
                        self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.grass.get_html_element(), block_texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, (x_idx + 5) as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                    }
                    BlockType::Dirt => {
                        self.chunks[chunk_index].1.context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(&self.textures.dirt.get_html_element(), block_texture_idx as f64 * 16.0, 0.0, 16.0, 16.0, (x_idx + 5) as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0).unwrap();
                    }
                    BlockType::Tree => self.chunks[chunk_index].1.draw_image(
                        (
                            (x_idx + 5) as f64 * 16.0 - 80.0,
                            y_idx as f64 * 16.0 - 240.0,
                        ),
                        &self.textures.tree,
                    ),
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn update_chunks(&mut self, player: &Player) {
        let chunk_number = x_to_chunk(player.x.floor() as isize);

        self.to_update_chunks.sort();
        self.to_update_chunks.dedup();
        for idx in 0..self.to_update_chunks.len() {
            self.update_chunk(self.to_update_chunks[idx]);
        }
        self.to_update_chunks.clear();

        let mut diff = self.first_chunk_number - chunk_number;
        let mut need_init_lights = false;
        while diff > -4 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(42 * 16);
            chunk_canvas.set_height(100 * 16);

            let mut light_chunk_canvas = Canvas::new();
            light_chunk_canvas.set_width(42 * 16);
            light_chunk_canvas.set_height(100 * 16);

            self.chunks.remove(self.chunks.len() - 1);
            let mut config = self.chunks[0].0.left_config;
            self.first_chunk_number -= 1;
            self.chunks.insert(
                0,
                (
                    Chunk::generate(
                        &mut config.0,
                        &mut config.1,
                        false,
                        self.first_chunk_number * 32,
                    ),
                    chunk_canvas,
                    light_chunk_canvas,
                ),
            );
            self.update_chunk(2);

            need_init_lights = true;
            diff = self.first_chunk_number - chunk_number;
        }
        while diff < -4 {
            let mut chunk_canvas = Canvas::new();
            chunk_canvas.set_width(42 * 16);
            chunk_canvas.set_height(100 * 16);

            let mut light_chunk_canvas = Canvas::new();
            light_chunk_canvas.set_width(42 * 16);
            light_chunk_canvas.set_height(100 * 16);

            self.chunks.remove(0);
            let mut config = self.chunks[self.chunks.len() - 1].0.right_config;
            self.first_chunk_number += 1;
            self.chunks.push((
                Chunk::generate(
                    &mut config.0,
                    &mut config.1,
                    true,
                    self.first_chunk_number * 32,
                ),
                chunk_canvas,
                light_chunk_canvas,
            ));
            self.update_chunk(self.chunks.len() - 2);

            need_init_lights = true;
            diff = self.first_chunk_number - chunk_number;
        }

        if need_init_lights {
            self.init_lights();
        }
    }

    pub fn init_lights(&mut self) {
        self.light_update.clear();

        for x in self.first_chunk_number * 32
            ..(self.first_chunk_number + self.chunks.len() as isize) * 32
        {
            let mut need_spreading = false;
            for y in 0..2 {
                if y == 0 && self[(x, y)].light == 0 {
                    self[(x, y)].light = 100;
                    need_spreading = true;
                }

                if need_spreading {
                    self.light_update.push((x, y, false));
                }
            }
        }

        self.spread_lights();
    }

    pub fn spread_lights(&mut self) {
        let mut n = 0;
        while !self.light_update.is_empty() {
            self.update_light();
            n += 1;

            if n > 100000 {
                panic!("100000 lights spread. Program hanging");
            }
        }
    }

    fn update_light(&mut self) {
        use std::cmp::max;

        if self.light_update.is_empty() {
            return;
        }

        let (x, y, cancellation) = self.light_update.remove(0);
        if x <= self.first_chunk_number * 32
            || x >= (self.first_chunk_number + self.chunks.len() as isize) * 32
            || y < 0
            || y > 100
        {
            return;
        }

        if cancellation {
            let updates;

            {
                let light = &self[(x, y)].light;
                let left_block = &self[(x - 1, y)];
                let right_block = &self[(x + 1, y)];
                let top_block = &self[(x, y - 1)];
                let bottom_block = &self[(x, y + 1)];
                updates = (
                    right_block.light + right_block.block_type.get_light_loss() == *light,
                    left_block.light + left_block.block_type.get_light_loss() == *light,
                    top_block.light + top_block.block_type.get_light_loss() == *light,
                    bottom_block.light + bottom_block.block_type.get_light_loss() == *light,
                );
            }

            self[(x, y)].light = 0;

            if updates.0 && !self.light_update.contains(&(x + 1, y, true)) {
                self.light_update.push((x + 1, y, true))
            }
            if updates.1 && !self.light_update.contains(&(x - 1, y, true)) {
                self.light_update.push((x - 1, y, true))
            }
            if updates.2 && !self.light_update.contains(&(x, y - 1, true)) {
                self.light_update.push((x, y - 1, true))
            }
            if updates.3 && !self.light_update.contains(&(x, y + 1, true)) {
                self.light_update.push((x, y + 1, true))
            }
            return;
        }

        let light;
        let updates;
        {
            let block = &self[(x, y)];
            let left_block = &self[(x - 1, y)];
            let right_block = &self[(x + 1, y)];
            let top_block = &self[(x, y - 1)];
            let bottom_block = &self[(x, y + 1)];
            light = max(
                max(left_block.light, right_block.light),
                max(top_block.light, bottom_block.light),
            )
            .saturating_sub(block.block_type.get_light_loss());
            updates = (
                right_block.light + right_block.block_type.get_light_loss() < light,
                left_block.light + left_block.block_type.get_light_loss() < light,
                top_block.light + top_block.block_type.get_light_loss() < light,
                bottom_block.light + bottom_block.block_type.get_light_loss() < light,
            );
        }
        if updates.0 && !self.light_update.contains(&(x + 1, y, false)) {
            self.light_update.push((x + 1, y, false))
        }
        if updates.1 && !self.light_update.contains(&(x - 1, y, false)) {
            self.light_update.push((x - 1, y, false))
        }
        if updates.2 && !self.light_update.contains(&(x, y - 1, false)) {
            self.light_update.push((x, y - 1, false))
        }
        if updates.3 && !self.light_update.contains(&(x, y + 1, false)) {
            self.light_update.push((x, y + 1, false))
        }

        self[(x, y)].light = light;
    }

    #[cfg(target_arch = "wasm32")]
    pub fn draw_on_canvas<'a>(
        &'a mut self,
        canvas: &'a mut Canvas,
        player: &Player,
        screen_center: (isize, isize),
    ) {
        canvas.clear();
        let gradient = canvas
            .context
            .create_radial_gradient(
                screen_center.0 as f64,
                screen_center.1 as f64 - 50.0,
                50.0,
                screen_center.0 as f64,
                screen_center.1 as f64 - 50.0,
                500.0,
            )
            .unwrap();
        gradient.add_color_stop(0.0, "rgba(0, 0, 0, 1.0)").unwrap();
        gradient.add_color_stop(0.5, "rgba(0, 0, 0, 0.2)").unwrap();
        gradient.add_color_stop(1.0, "rgba(0, 0, 0, 0.0)").unwrap();
        canvas.context.set_fill_style(&gradient);
        canvas.context.fill_rect(
            0.0,
            0.0,
            canvas.get_width() as f64,
            canvas.get_height() as f64,
        );

        for (chunk_idx, light_canvas) in self.chunks.iter().map(|(_a, _b, c)| c).enumerate() {
            let (mut screen_x, mut screen_y) = map_to_screen(
                (self.first_chunk_number + chunk_idx as isize) * 32,
                self.first_block as isize,
                &player,
                screen_center,
            );
            screen_x = screen_x.floor();
            screen_y = screen_y.floor();

            canvas.draw_canvas((screen_x - 5.0 * 16.0, screen_y), &light_canvas);
        }

        self.canvas.clear();
        for (chunk_idx, chunk_canvas) in self.chunks.iter().map(|(_a, b, _c)| b).enumerate() {
            self.canvas.draw_canvas(
                (
                    (chunk_idx as f64 * 32.0 * 16.0) - 5.0 * 16.0,
                    self.first_block as f64 * 16.0,
                ),
                &chunk_canvas,
            );
        }

        let (mut screen_x, mut screen_y) = map_to_screen(
            (self.first_chunk_number) * 32,
            self.first_block as isize,
            &player,
            screen_center,
        );
        screen_x = screen_x.floor();
        screen_y = screen_y.floor();

        canvas
            .context
            .set_global_composite_operation("source-in")
            .unwrap();
        canvas.draw_canvas((screen_x, screen_y), &self.canvas);
        canvas
            .context
            .set_global_composite_operation("destination-over")
            .unwrap();
        use wasm_bindgen::JsValue;
        use wasm_game_lib::graphics::color::Color;
        canvas
            .context
            .set_fill_style(&JsValue::from_str(&Color::black().to_string()));
        canvas.context.fill_rect(
            0.0,
            0.0,
            screen_center.0 as f64 * 2.0,
            screen_center.1 as f64 * 2.0,
        );
        canvas
            .context
            .set_global_composite_operation("source-over")
            .unwrap();
    }
}

impl std::ops::Index<(isize, isize)> for Map {
    type Output = Block;

    fn index(&self, (x, y): (isize, isize)) -> &Self::Output {
        let (chunk, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk - self.first_chunk_number;

        if y >= 0 && chunk_index >= 0 {
            if let Some(chunk) = self.chunks.get(chunk_index as usize) {
                if let Some(block) = chunk.0.blocks[column as usize].get(y as usize) {
                    return &block;
                }
            }
        }

        &Block {
            block_type: BlockType::Air,
            natural_background: NaturalBackground::Sky,
            light: 0,
        }
    }
}

impl std::ops::IndexMut<(isize, isize)> for Map {
    fn index_mut(&mut self, (x, y): (isize, isize)) -> &mut Self::Output {
        let (chunk_number, column) = x_to_chunk_and_column(x);
        let chunk_index = chunk_number - self.first_chunk_number;

        if y >= 0 && chunk_index >= 0 {
            if let Some(chunk) = self.chunks.get_mut(chunk_index as usize) {
                if let Some(block) = chunk.0.blocks[column as usize].get_mut(y as usize) {
                    self.to_update_chunks.push(chunk_index as usize);
                    if column == 0 && chunk_index > 0 {
                        self.to_update_chunks.push((chunk_index - 1) as usize);
                    }
                    if column == 31 {
                        self.to_update_chunks.push((chunk_index + 1) as usize);
                    }
                    return block;
                }
            }
        }

        if self.air
            != (Block {
                block_type: BlockType::Air,
                natural_background: NaturalBackground::Sky,
                light: 0,
            })
        {
            self.air = Block {
                block_type: BlockType::Air,
                natural_background: NaturalBackground::Sky,
                light: 0,
            };
        }

        &mut self.air
    }
}

#[test]
fn test() {
    use std::mem::MaybeUninit;

    let mut map = Map {
        chunks: Vec::new(),
        first_chunk_number: -5,
        first_block: 0,
        air: Block {
            block_type: BlockType::Air,
            natural_background: NaturalBackground::Sky,
            light: 0,
        },
        to_update_chunks: Vec::new(),
        light_update: Vec::new(),
    };

    let mut height: f64 = 20.0;
    let mut slope: f64 = 0.2;
    for i in -5..5 {
        let chunk_canvas = unsafe { MaybeUninit::uninit().assume_init() };

        let light_chunk_canvas = unsafe { MaybeUninit::uninit().assume_init() };

        map.chunks.push((
            Chunk::generate(&mut height, &mut slope, true, i * 32),
            chunk_canvas,
            light_chunk_canvas,
        ));
    }

    map.init_lights();

    println!("{:?}", map[(10, 10)]);

    println!("\x1B[1;32mSUCCESS\x1B[0m");
}
