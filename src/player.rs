use crate::{map::Map, textures::Textures, items::*};
use wasm_game_lib::{graphics::{canvas::*, color::Color}};
use wasm_bindgen::JsValue;

const INVENTORY_BORDER_STYLE: LineStyle = LineStyle {
    cap: LineCap::Square,
    join: LineJoin::Miter,
    color: Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 150,
    },
    size: 3.0,
};
const SELECTED_INVENTORY_BORDER_STYLE: LineStyle = LineStyle {
    cap: LineCap::Square,
    join: LineJoin::Miter,
    color: Color {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 225,
    },
    size: 4.0,
};

pub struct Player<'a> {
    pub x: f64,
    pub y: f64,
    speed_y: f64,
    textures: &'a Textures,
    last_frame_running: usize,
    to_left: bool,
    is_inventory_open: bool,
    selected_slot: u8,
    pub inventory: Inventory,
}

impl<'a> Player<'a> {
    pub fn new(textures: &Textures) -> Player {
        Player {
            x: 3.0,
            y: -10.0,
            speed_y: 0.0,
            textures,
            last_frame_running: 0,
            to_left: true,
            is_inventory_open: false,
            inventory: Inventory::new(27),
            selected_slot: 0,
        }
    }

    pub fn is_touching_the_surface(&self, map: &Map) -> bool {
        !map[(self.x.floor() as isize, (self.y + 0.03).floor() as isize)].can_pass_through()
            || !map[(self.x.ceil() as isize, (self.y + 0.03).floor() as isize)].can_pass_through()
    }

    pub fn is_under_the_surface(&self, map: &Map) -> bool {
        !map[(self.x.floor() as isize, self.y.floor() as isize)].can_pass_through()
            || !map[(self.x.ceil() as isize, self.y.floor() as isize)].can_pass_through()
    }

    pub fn can_move_right_by(&self, distance: f64, map: &Map) -> bool {
        map[((self.x + distance).ceil() as isize, self.y.floor() as isize)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 1)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 2)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 3)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 4)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 5)].can_pass_through()
            && map[((self.x + distance).ceil() as isize, self.y.floor() as isize - 6)].can_pass_through()
    }

    pub fn can_move_left_by(&self, distance: f64, map: &Map) -> bool {
        map[((self.x - distance).floor() as isize, self.y.floor() as isize)].can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 1)]
                .can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 2)]
                .can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 3)]
                .can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 4)]
                .can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 5)]
                .can_pass_through()
            && map[((self.x - distance).floor() as isize, self.y.floor() as isize - 6)]
                .can_pass_through()
    }

    pub fn can_move_up_by(&self, distance: f64, map: &Map) -> bool {
        map[(
            self.x.floor() as isize,
            (self.y + distance).floor() as isize - 7,
        )]
            .can_pass_through()
            && map[(
                self.x.ceil() as isize,
                (self.y + distance).floor() as isize - 7,
            )]
                .can_pass_through()
    }

    pub fn handle_events(&mut self, keys: (bool, bool, bool, bool), map: &Map, frame: usize) {
        if keys.1 {
            if self.can_move_right_by(0.3, &map) {
                self.x += 0.3;
                self.last_frame_running = frame;
                self.to_left = false;
            }

            if self.is_touching_the_surface(&map) && !self.can_move_right_by(0.9, &map) {
                self.y -= 1.0;
                if self.can_move_right_by(0.9, &map) {
                    self.speed_y = -0.36;
                    self.last_frame_running = frame;
                    self.to_left = false;
                }
                self.y += 1.0;
            }
        }
        if keys.0 && self.is_touching_the_surface(&map) {
            self.speed_y = -0.40;
        }
        if keys.3 {
            if self.can_move_left_by(0.3, &map) {
                self.x -= 0.3;
                self.last_frame_running = frame;
                self.to_left = true;
            }

            if self.is_touching_the_surface(&map) && !self.can_move_left_by(0.9, &map) {
                self.y -= 1.0;
                if self.can_move_left_by(0.9, &map) {
                    self.speed_y = -0.36;
                    self.last_frame_running = frame;
                    self.to_left = true;
                }
                self.y += 1.0;
            }
        }

        if self.speed_y < 0.0 {
            if self.can_move_up_by(self.speed_y, &map) {
                self.y += self.speed_y;
            } else {
                self.speed_y = 0.0;
            }
        } else {
            self.y += self.speed_y;
        }

        if self.is_under_the_surface(&map) {
            self.y -= self.speed_y;
            self.y = self.y.ceil() - 0.01;
            self.speed_y = 0.0;
        } else if !self.is_touching_the_surface(&map) {
            self.speed_y += 0.03;
        }
    }

    pub fn change_inventory_state(&mut self) {
        self.is_inventory_open = !self.is_inventory_open;
    }

    pub fn draw_on_canvas(&mut self, mut canvas: &mut Canvas, screen_center: (isize, isize), mut frame: usize) {
        if frame - self.last_frame_running < 6 {
            frame -= frame % 5;
            frame /= 5;
            frame %= 8;
            let x = frame as f64 * 96.0;
            canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(if self.to_left {&(self.textures.character.1).1} else {&(self.textures.character.1).0}.get_html_element(), x ,0.0, 96.0, 128.0, screen_center.0 as f64 - 32.0, screen_center.1 as f64 - 128.0, 96.0, 128.0).unwrap();
        } else {
            canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element(if self.to_left {&(self.textures.character.0).1} else {&(self.textures.character.0).0}.get_html_element(), screen_center.0 as f64 - 16.0, screen_center.1 as f64 - 128.0).unwrap();
        }

        INVENTORY_BORDER_STYLE.apply_on_canvas(&mut canvas);
        canvas.context.begin_path();
        canvas.context.set_fill_style(&JsValue::from_str("rgba(24, 28, 39, 0.9)"));

        if self.is_inventory_open {
            canvas.context.fill_rect(
                0.0,
                0.0,
                screen_center.0 as f64 * 2.0 + 1.0,
                screen_center.1 as f64 * 2.0 + 1.0,
            );
            canvas.context.stroke();

            canvas.context.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.23)"));
            
            for idx in 0..27 {
                let x = idx % 9;
                let y = (idx - x) / 9;

                canvas.context.fill_rect(
                    82.0 + x as f64 * (64.0 + 32.0),
                    82.0 + y as f64 * (64.0 + 32.0),
                    64.0,
                    64.0,
                );

                canvas.context.rect(
                    82.0 + x as f64 * (64.0 + 32.0),
                    82.0 + y as f64 * (64.0 + 32.0),
                    64.0,
                    64.0
                );

                if let Some((item, _quantity)) = self.inventory[idx] {
                    canvas.context.draw_image_with_html_image_element(self.textures.get_for_item(item).get_html_element(), 82.0 + x as f64 * (64.0 + 32.0), 82.0 + y as f64 * (64.0 + 32.0)).unwrap();
                }
            }
            canvas.context.stroke();
        } else {
            canvas.context.fill_rect(
                screen_center.0 as f64 - 4.5 * 64.0,
                screen_center.1 as f64 * 2.0 - 64.0,
                64.0 * 9.0,
                64.0,
            );

            canvas.context.set_fill_style(&JsValue::from_str("rgba(255, 255, 255, 0.23)"));
            
            for x in 0..9 {
                canvas.context.fill_rect(
                    screen_center.0 as f64 - 4.5 * 64.0 + x as f64 * 64.0,
                    screen_center.1 as f64 * 2.0 - 64.0,
                    64.0,
                    64.0,
                );

                if x == self.selected_slot {
                    SELECTED_INVENTORY_BORDER_STYLE.apply_on_canvas(&mut canvas);
                }

                canvas.context.rect(
                    screen_center.0 as f64 - 4.5 * 64.0 + x as f64 * 64.0,
                    screen_center.1 as f64 * 2.0 - 64.0,
                    64.0,
                    64.0
                );

                if x == self.selected_slot {
                    canvas.context.stroke();
                    INVENTORY_BORDER_STYLE.apply_on_canvas(&mut canvas);
                }

                if let Some((item, _quantity)) = self.inventory[x as usize] {
                    canvas.context.draw_image_with_html_image_element(self.textures.get_for_item(item).get_html_element(), screen_center.0 as f64 - 4.5 * 64.0 + x as f64 * 64.0, screen_center.1 as f64 * 2.0 - 64.0).unwrap();
                }
            }
            canvas.context.stroke();
        }
    }
}
