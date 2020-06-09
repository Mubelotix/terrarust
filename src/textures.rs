use crate::{loader::load_images, items::Item};
use wasm_game_lib::graphics::{canvas::Canvas, image::Image};

// resize : convert running.png -interpolate Integer -filter point -resize "200%" output.png

pub struct Textures {
    pub character: ((Image, Image), (Image, Image)),
    pub grass: Image,
    pub dirt: Image,
    pub tree: Image,
    pub item_log: Image,
    pub item_wood_stick: Image,
    pub item_foliage: Image,
}

impl Textures {
    pub async fn load(mut canvas: &mut Canvas) -> Textures {
        let mut t = load_images(
            vec![
                "ressources/character/idle.png",
                "ressources/character/idle2.png",
                "ressources/character/running.png",
                "ressources/character/running2.png",
                "ressources/blocks/grass.png",
                "ressources/blocks/dirt.png",
                "ressources/tree.png",
                "ressources/items/log.png",
                "ressources/items/wood_stick.png",
                "ressources/items/foliage.png",
            ],
            &mut canvas,
        )
        .await;

        Textures {
            character: ((t.remove(0), t.remove(0)), (t.remove(0), t.remove(0))),
            grass: t.remove(0),
            dirt: t.remove(0),
            tree: t.remove(0),
            item_log: t.remove(0),
            item_wood_stick: t.remove(0),
            item_foliage: t.remove(0),
        }
    }

    pub fn get_for_item(&self, item: Item) -> &Image {
        match item {
            Item::Dirt => &self.dirt,
            Item::Log => &self.item_log,
            Item::WoodStick => &self.item_wood_stick,
            Item::Foliage => &self.item_foliage,
        }
    }
}
