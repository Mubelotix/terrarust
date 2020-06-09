#[derive(Copy, Clone)]
pub enum Item {

}

pub struct Inventory {
    slots: Vec<Option<(Item, usize)>>,
}

impl Inventory {
    pub fn new(slots_number: usize) -> Inventory {
        let mut slots = Vec::new();
        for _ in 0..slots_number {
            slots.push(None);
        }
        Inventory {
            slots
        }
    }
}

impl std::ops::Index<usize> for Inventory {
    type Output = Option<(Item, usize)>;

    fn index(&self, idx: usize) -> &Self::Output {
        self.slots.get(idx).unwrap_or(&None)
    }
}