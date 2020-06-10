use crate::items::Item;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Block {
    Grass,
    Air,
    Dirt,
    Tree,
}

impl Block {
    pub fn can_pass_through(self) -> bool {
        match self {
            Block::Grass => false,
            Block::Dirt => false,
            Block::Air => true,
            Block::Tree => true,
        }
    }

    pub fn as_item(&self) -> Vec<Item> {
        match self {
            Block::Grass => vec![Item::Dirt],
            Block::Dirt => vec![Item::Dirt],
            Block::Air => vec![],
            Block::Tree => vec![Item::Log, Item::WoodStick, Item::Foliage],
        }
    }
}