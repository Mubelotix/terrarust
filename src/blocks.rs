use crate::items::Item;

#[derive(PartialEq, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
    pub natural_background: NaturalBackground,
}

impl Block {
    pub const fn new(block_type: BlockType, natural_background: NaturalBackground) -> Block {
        Block {
            block_type,
            natural_background
        }
    }

    pub fn can_pass_through(&self) -> bool {
        self.block_type.can_pass_through()
    }

    pub fn as_item(&self) -> Vec<Item> {
        self.block_type.as_item()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlockType {
    Grass,
    Air,
    Dirt,
    Tree,
}

impl BlockType {
    pub fn can_pass_through(self) -> bool {
        match self {
            BlockType::Grass => false,
            BlockType::Dirt => false,
            BlockType::Air => true,
            BlockType::Tree => true,
        }
    }

    pub fn as_item(&self) -> Vec<Item> {
        match self {
            BlockType::Grass => vec![Item::Dirt],
            BlockType::Dirt => vec![Item::Dirt],
            BlockType::Air => vec![],
            BlockType::Tree => vec![Item::Log, Item::WoodStick, Item::Foliage],
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum NaturalBackground {
    Sky,
    Dirt,
}