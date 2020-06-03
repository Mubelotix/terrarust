use crate::{map::Map, textures::Textures};
use wasm_game_lib::{graphics::{canvas::Canvas}};

pub struct Player<'a> {
    pub x: f64,
    pub y: f64,
    speed_y: f64,
    textures: &'a Textures,
    last_frame_running: usize,
}

impl<'a> Player<'a> {
    pub fn new(textures: &Textures) -> Player {
        Player {
            x: 3.0,
            y: -10.0,
            speed_y: 0.0,
            textures,
            last_frame_running: 0,
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
            }

            if self.is_touching_the_surface(&map) && !self.can_move_right_by(0.9, &map) {
                self.y -= 1.0;
                if self.can_move_right_by(0.9, &map) {
                    self.speed_y = -0.36;
                    self.last_frame_running = frame;
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
            }

            if self.is_touching_the_surface(&map) && !self.can_move_left_by(0.9, &map) {
                self.y -= 1.0;
                if self.can_move_left_by(0.9, &map) {
                    self.speed_y = -0.36;
                    self.last_frame_running = frame;
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

    pub fn draw_on_canvas(&mut self, canvas: &mut Canvas, screen_center: (isize, isize), mut frame: usize) {
        if frame - self.last_frame_running < 6 {
            frame -= frame % 5;
            frame /= 5;
            frame %= 8;
            let x = frame as f64 * 96.0;
            canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(self.textures.character.1.get_html_element(), x ,0.0, 96.0, 128.0, screen_center.0 as f64 - 32.0, screen_center.1 as f64 - 128.0, 96.0, 128.0).unwrap();
        } else {
            canvas.get_2d_canvas_rendering_context().draw_image_with_html_image_element(self.textures.character.0.get_html_element(), screen_center.0 as f64 - 16.0, screen_center.1 as f64 - 128.0).unwrap();
        }
    }
}
