use wasm_bindgen::prelude::*;
use wasm_game_lib::{graphics::{image::Image, sprite::Sprite, window::Window, color::Color}, inputs::{mouse::start_recording_mouse_events, keyboard::{KeyboardEvent, Key}, event::Event}};
use wasm_game_lib::inputs::event::types::*;
use wasm_game_lib::system::sleep;
use std::time::Duration;
use console_error_panic_hook::set_once;
use lazy_static::lazy_static;
use std::sync::Mutex;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod loader;
mod progress_bar;
mod textures;
use loader::load_images;
mod map;
mod coords;
mod player;
use player::Player;
use map::Map;
use textures::Textures;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_once();
    start_recording_mouse_events();

    let (mut window, mut canvas) =
        Window::init_with_events(KEYBOARD_EVENT + RESIZE_EVENT + MOUSE_EVENT);

    let textures = Textures::load(&mut canvas).await;

    let map = Map::new(&textures);
    let mut player = Player::new();
    let chara = Sprite::new((264.0, 0.0), &textures.character, (0.0,0.0));

    let mut direction_keys = (false, false, false, false);

    loop {
        
        for event in window.poll_events() {
            #[allow(clippy::single_match)]
            match event {
                Event::KeyboardEvent(event) => match event {
                    KeyboardEvent::Down(key) => match key {
                        Key::UpArrow => direction_keys.0 = true,
                        Key::RightArrow => direction_keys.1 = true,
                        Key::DownArrow => direction_keys.2 = true,
                        Key::LeftArrow => direction_keys.3 = true,
                        _ => (),
                    },
                    KeyboardEvent::Up(key) => match key {
                        Key::UpArrow => direction_keys.0 = false,
                        Key::RightArrow => direction_keys.1 = false,
                        Key::DownArrow => direction_keys.2 = false,
                        Key::LeftArrow => direction_keys.3 = false,
                        _ => (),
                    }
                }
                _ => (),
            }
            //log!("{:?}", event);
        }

        player.handle_events(direction_keys);

        canvas.clear_with_color(Color::cyan());
        map.draw_on_canvas(&mut canvas, &player);
        canvas.draw(&chara);
        //scanvas.draw(&units);

        sleep(Duration::from_millis(16)).await;
    }
}