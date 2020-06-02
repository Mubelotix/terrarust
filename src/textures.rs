use crate::loader::load_images;
use wasm_game_lib::graphics::{canvas::Canvas, image::Image};

pub struct Textures {
    pub character: Image,
    pub grass: ([Image; 4], Image, Image),
    pub dirt: Image,
    pub tree: Image,
}

impl Textures {
    pub async fn load(mut canvas: &mut Canvas) -> Textures {
        let mut t = load_images(
            vec![
                "ressources/character.png",
                "ressources/blocks/grass/1.png",
                "ressources/blocks/grass/2.png",
                "ressources/blocks/grass/3.png",
                "ressources/blocks/grass/4.png",
                "ressources/blocks/grass/corner1.png",
                "ressources/blocks/grass/corner2.png",
                "ressources/blocks/dirt.png",
                "ressources/tree.png",
            ],
            &mut canvas,
        )
        .await;

        Textures {
            character: t.remove(0),
            grass: (
                [t.remove(0), t.remove(0), t.remove(0), t.remove(0)],
                t.remove(0),
                t.remove(0),
            ),
            dirt: t.remove(0),
            tree: t.remove(0),
        }
    }
}
