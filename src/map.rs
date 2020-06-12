use crate::{
    blocks::{Block, BlockType, NaturalBackground},
    coords::{map_to_screen, x_to_biome, x_to_chunk, x_to_chunk_and_column},
    player::Player,
    textures::{get_texture_idx, Textures},
};
use arr_macro::arr;
use std::{hash::Hasher, rc::Rc};
use twox_hash::XxHash32;
use wasm_game_lib::{graphics::{canvas::Canvas}, log};

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
    blocks: Vec<[Block; 2048]>, // 32
    pub left_config: (f64, f64),  // (height, slope)
    pub right_config: (f64, f64), // idem
}

impl Chunk {
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::eval_order_dependence)]
    pub fn generate(height: &mut f64, slope: &mut f64, left_to_right: bool, mut x: isize) -> Chunk {
        let begin_config: (f64, f64) = (*height, *slope);
        let biome = x_to_biome(x);

        let mut blocks = Vec::new();
        for _idx in 0..32 {
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

            let hasher2 = XxHash32::with_seed(42); // to avoid generating a tree if there is a tree at the left
            hasher.write_isize(x-1);
            let tree = hash % biome.get_tree_prob() as u64 == 0 && hasher2.finish() % biome.get_tree_prob() as u64 != 0;

            *height += *slope;
            
            if left_to_right {
                x += 1;
            } else {
                x -= 1;
            }
            
            let mut column = arr!(Block{block_type: BlockType::Dirt, natural_background: NaturalBackground::Dirt, light: 0}; 2048);
            for block in column.iter_mut().take(height.floor() as usize) {
                *block = Block{block_type: BlockType::Air, natural_background: NaturalBackground::Sky, light: 0};
            }
            column[height.floor() as usize] = Block{block_type: BlockType::Grass, natural_background: NaturalBackground::Dirt, light: 0};
            if tree && height.floor() as usize > 0 {
                column[height.floor() as usize - 1] = Block{block_type: BlockType::Tree, natural_background: NaturalBackground::Dirt, light: 0};
            }
            blocks.push(column)
        }

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
    chunks: Vec<(Chunk, Canvas, Canvas)>,
    canvas: Canvas,
    first_chunk_number: isize,
    first_block: usize,
    textures: Rc<Textures>,
    air: Block,
    to_update_chunks: Vec<usize>,
    pub light_update: Vec<(isize, isize, bool)>,
}

impl Map {
    pub fn new(textures: Rc<Textures>) -> Map {
        let mut map = Map {
            chunks: Vec::new(),
            textures,
            first_chunk_number: -5,
            first_block: 0,
            air: Block{block_type: BlockType::Air, natural_background: NaturalBackground::Sky, light: 0},
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
                light_chunk_canvas
            ));
        }

        for i in 0..10 {
            map.update_chunk(i);
        }

        map.init_lights();

        map
    }

    pub fn update_chunk(&mut self, chunk_index: usize) {
        if chunk_index >= 10 {
            return;
        }
        self.chunks[chunk_index].1.clear();
        use wasm_bindgen::JsValue;
        self.chunks[chunk_index].1.context.set_fill_style(&JsValue::from_str("rgb(135,206,235)"));
        self.chunks[chunk_index].1.context.fill_rect(5.0 * 16.0, 0.0, 42.0 * 16.0, 100.0 * 16.0);

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
                self.chunks[chunk_index].2.context.set_fill_style(&JsValue::from(format!("rgba(255,255,255,0.{:02})", self.chunks[chunk_index].0.blocks[x_idx][y_idx].light)));
                self.chunks[chunk_index].2.context.fill_rect((x_idx + 5) as f64 * 16.0, y_idx as f64 * 16.0, 16.0, 16.0);
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

    pub fn update_chunks(&mut self, player: &Player) {
        let chunk_number = x_to_chunk(player.x.floor() as isize);

        self.to_update_chunks.sort();
        self.to_update_chunks.dedup();
        for idx in 0..self.to_update_chunks.len() {
            self.update_chunk(self.to_update_chunks[idx]);
        }
        self.to_update_chunks.clear();

        let mut diff = self.first_chunk_number - chunk_number;
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
                    light_chunk_canvas
                ),
            );
            self.update_chunk(2);

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
                light_chunk_canvas
            ));
            self.update_chunk(self.chunks.len() - 2);

            diff = self.first_chunk_number - chunk_number;
        }
    }

    pub fn init_lights(&mut self) {
        log!("Initializing lights");

        self.light_update.clear();

        for x in self.first_chunk_number * 32..(self.first_chunk_number + self.chunks.len() as isize) * 32 {
            for y in 0..2048 {
                if y == 0 {
                    self[(x,y)].light = 100;
                } else {
                    self[(x,y)].light = 0;
                }
                
                if y == 1 {
                    self.light_update.push((x, y, false));
                }
            }
        }

        self.spread_lights();

        log!("Lights initialized");
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
        if x <= self.first_chunk_number * 32 || x >= (self.first_chunk_number + self.chunks.len() as isize) * 32 || y < 0 || y > 100 {
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
                    bottom_block.light + bottom_block.block_type.get_light_loss() == *light
                );
            }

            self[(x, y)].light = 0;

            if updates.0 && !self.light_update.contains(&(x + 1, y, true))  {
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
            light = max(max(left_block.light, right_block.light), max(top_block.light, bottom_block.light)).saturating_sub(block.block_type.get_light_loss());
            updates = ( 
                right_block.light + right_block.block_type.get_light_loss() < light,
                left_block.light + left_block.block_type.get_light_loss() < light,
                top_block.light + top_block.block_type.get_light_loss() < light,
                bottom_block.light + bottom_block.block_type.get_light_loss() < light
            );
        }
        if updates.0 && !self.light_update.contains(&(x + 1, y, false))  {
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
}

impl Map {
    pub fn draw_on_canvas<'a>(
        &'a mut self,
        canvas: &'a mut Canvas,
        player: &Player,
        screen_center: (isize, isize),
    ) {
        canvas.clear();
        let gradient = canvas.context.create_radial_gradient(screen_center.0 as f64, screen_center.1 as f64 - 50.0, 50.0, screen_center.0 as f64, screen_center.1 as f64 - 50.0, 500.0).unwrap();
        gradient.add_color_stop(0.0, "rgba(0, 0, 0, 1.0)").unwrap();
        gradient.add_color_stop(0.5, "rgba(0, 0, 0, 0.2)").unwrap();
        gradient.add_color_stop(1.0, "rgba(0, 0, 0, 0.0)").unwrap();
        canvas.context.set_fill_style(&gradient);
        canvas.context.fill_rect(0.0, 0.0, canvas.get_width() as f64, canvas.get_height() as f64);

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
            self.canvas.draw_canvas(((chunk_idx as f64 * 32.0 * 16.0) - 5.0 * 16.0, self.first_block as f64 * 16.0), &chunk_canvas);
        }

        let (mut screen_x, mut screen_y) = map_to_screen(
            (self.first_chunk_number) * 32,
            self.first_block as isize,
            &player,
            screen_center,
        );
        screen_x = screen_x.floor();
        screen_y = screen_y.floor();

        canvas.context.set_global_composite_operation("source-in").unwrap();
        canvas.draw_canvas((screen_x, screen_y), &self.canvas);
        canvas.context.set_global_composite_operation("destination-over").unwrap();
        use wasm_bindgen::JsValue;
        use wasm_game_lib::graphics::color::Color;
        canvas.context.set_fill_style(&JsValue::from_str(&Color::black().to_string()));
        canvas.context.fill_rect(0.0,0.0, screen_center.0 as f64 * 2.0, screen_center.1 as f64 * 2.0);
        canvas.context.set_global_composite_operation("source-over").unwrap();
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

        if self.air != (Block{block_type: BlockType::Air, natural_background: NaturalBackground::Sky, light: 0}) {
            self.air = Block{block_type: BlockType::Air, natural_background: NaturalBackground::Sky, light: 0};
        }

        &mut self.air
    }
}

#[allow(invalid_value)]
#[allow(clippy::uninit_assumed_init)]
#[test]
fn test() {
    use std::mem::MaybeUninit;

    let textures = unsafe {MaybeUninit::uninit().assume_init()};
    let canvas = unsafe {MaybeUninit::uninit().assume_init()};

    let mut map = Map {
        chunks: Vec::new(),
        textures,
        first_chunk_number: -5,
        first_block: 0, 
        air: Block{block_type: BlockType::Air, natural_background: NaturalBackground::Sky, light: 0},
        to_update_chunks: Vec::new(),
        light_update: Vec::new(),
        canvas,
    };

    let mut height: f64 = 20.0;
    let mut slope: f64 = 0.2;
    for i in -5..5 {
        let chunk_canvas = unsafe {MaybeUninit::uninit().assume_init()};

        let light_chunk_canvas = unsafe {MaybeUninit::uninit().assume_init()};

        map.chunks.push((
            Chunk::generate(&mut height, &mut slope, true, i * 32),
            chunk_canvas,
            light_chunk_canvas
        ));
    }

    map.init_lights();

    println!("{:?}", map[(10, 10)]);

    println!("\x1B[1;32mSUCCESS\x1B[0m");
}