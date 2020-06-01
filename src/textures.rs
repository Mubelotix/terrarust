use wasm_game_lib::{graphics::{image::Image, canvas::Canvas}};
use crate::loader::load_images;

pub struct Textures {
    pub character: Image,
    pub grass: [Image; 4],
    pub dirt: Image,
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
                "ressources/blocks/dirt.png",
            ],
            &mut canvas,
        )
        .await;

        Textures {
            character: t.remove(0),
            grass: [t.remove(0), t.remove(0), t.remove(0), t.remove(0)],
            dirt: t.remove(0),
        }
    }
}