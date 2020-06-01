use crate::{map::Block, map::Map};

pub struct Player {
    pub x: f64,
    pub y: f64,
    speed_y: f64,
}

impl Player {
    pub fn new() -> Player {
        Player {
            x: 3.0,
            y: -10.0,
            speed_y: 0.0,
        }
    }

    pub fn is_touching_the_surface(&self, map: &Map) -> bool {
        map[(self.x.floor() as isize, (self.y + 0.03).floor() as isize)] != Block::Air
            || map[(self.x.ceil() as isize, (self.y + 0.03).floor() as isize)] != Block::Air
    }

    pub fn is_under_the_surface(&self, map: &Map) -> bool {
        map[(self.x.floor() as isize, self.y.floor() as isize)] != Block::Air
            || map[(self.x.ceil() as isize, self.y.floor() as isize)] != Block::Air
    }

    pub fn can_move_right(&self, map: &Map) -> bool {
        map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 1)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 2)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 3)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 4)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 5)] == Block::Air
            && map[((self.x + 0.3).ceil() as isize, self.y.floor() as isize - 6)] == Block::Air
    }

    pub fn can_move_left(&self, map: &Map) -> bool {
        map[((self.x - 0.3).floor() as isize, self.y.floor() as isize)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 1)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 2)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 3)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 4)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 5)] == Block::Air
            && map[((self.x - 0.3).floor() as isize, self.y.floor() as isize - 6)] == Block::Air
    }

    pub fn can_move_up_by(&self, distance: f64, map: &Map) -> bool {
        map[(
            self.x.floor() as isize,
            (self.y + distance).floor() as isize - 7,
        )] == Block::Air
            && map[(
                self.x.ceil() as isize,
                (self.y + distance).floor() as isize - 7,
            )] == Block::Air
    }

    pub fn handle_events(&mut self, keys: (bool, bool, bool, bool), map: &Map) {
        if keys.1 && self.can_move_right(&map) {
            self.x += 0.3;
        }
        if keys.0 && self.is_touching_the_surface(&map) {
            self.speed_y = -0.40;
        }
        if keys.3 && self.can_move_left(&map) {
            self.x -= 0.3;
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
            log!("y {}", self.y);
            self.speed_y = 0.0;
        } else if !self.is_touching_the_surface(&map) {
            self.speed_y += 0.03;
            log!("falling");
        }
    }
}
