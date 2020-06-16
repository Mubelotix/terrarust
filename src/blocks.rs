use crate::items::Item;

#[derive(PartialEq, Clone, Debug)]
pub struct Block {
    pub block_type: BlockType,
    pub natural_background: NaturalBackground,
    pub light: usize,
    pub water: usize,
}

impl Block {
    pub const fn _new(block_type: BlockType, natural_background: NaturalBackground) -> Block {
        Block {
            block_type,
            natural_background,
            light: 0,
            water: 0,
        }
    }

    pub fn can_pass_through(&self) -> bool {
        self.block_type.clone().can_pass_through()
    }

    pub fn as_item(&self) -> Vec<Item> {
        self.block_type.as_item()
    }
}

#[derive(Debug, Clone, PartialEq)]
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

    pub fn get_light_loss(&self) -> usize {
        match self {
            BlockType::Grass => 6,
            BlockType::Dirt => 10,
            BlockType::Air => 1,
            BlockType::Tree => 1,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum NaturalBackground {
    Sky,
    Dirt,
}
