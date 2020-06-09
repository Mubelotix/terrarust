#[derive(Copy, Clone, PartialEq)]
pub enum Item {
    Dirt,
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

    pub fn push(&mut self, item: Item) -> bool {
        for slot in &mut self.slots {
            if let Some((slot_item, quantity)) = slot {
                if slot_item == &item {
                    *quantity += 1;
                    return true;
                }
            }
        }

        for slot in &mut self.slots {
            if slot.is_none() {
                *slot = Some((item, 1));
                return true;
            }
        }
        
        false
    }
}

impl std::ops::Index<usize> for Inventory {
    type Output = Option<(Item, usize)>;

    fn index(&self, idx: usize) -> &Self::Output {
        self.slots.get(idx).unwrap_or(&None)
    }
}