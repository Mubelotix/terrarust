pub struct Player {
    pub x: usize,
    pub y: usize,
}

impl Player {
    pub fn new() -> Player {
        Player { x: 20, y: 20 }
    }

    pub fn handle_events(&mut self, keys: (bool, bool, bool, bool)) {
        if keys.1 {
            self.x += 1;
        }
        if keys.3 {
            self.x -= 1;
        }
    }
}
