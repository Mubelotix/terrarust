pub struct Player {
    pub x: f64,
    pub y: f64,
}

impl Player {
    pub fn new() -> Player {
        Player { x: 3.0, y: 0.0 }
    }

    pub fn handle_events(&mut self, keys: (bool, bool, bool, bool)) {
        if keys.1 {
            self.x += 0.1;
        }
        if keys.3 {
            self.x -= 0.1;
        }
    }
}
