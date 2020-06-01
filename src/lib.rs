use console_error_panic_hook::set_once;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_game_lib::inputs::event::types::*;
use wasm_game_lib::system::sleep;
use wasm_game_lib::{
    graphics::{color::Color, sprite::Sprite, window::Window},
    inputs::{
        event::Event,
        keyboard::{Key, KeyboardEvent},
        mouse::start_recording_mouse_events,
    },
};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod coords;
mod loader;
mod map;
mod player;
mod progress_bar;
mod textures;
use map::Map;
use player::Player;
use textures::Textures;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_once();
    start_recording_mouse_events();

    let (mut window, mut canvas) =
        Window::init_with_events(KEYBOARD_EVENT + RESIZE_EVENT + MOUSE_EVENT);

    let screen_center = (
        canvas.get_width() as isize / 2,
        canvas.get_height() as isize / 2,
    );

    let textures = Textures::load(&mut canvas).await;

    let map = Map::new(&textures);
    let mut player = Player::new();
    let chara = Sprite::new((screen_center.0 as f64, screen_center.1 as f64), &textures.character, (0.0, 0.0));

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
                    },
                },
                _ => (),
            }
            //log!("{:?}", event);
        }

        player.handle_events(direction_keys);

        canvas.clear_with_color(Color::cyan());
        map.draw_on_canvas(&mut canvas, &player, screen_center);
        canvas.draw(&chara);
        //scanvas.draw(&units);

        sleep(Duration::from_millis(16)).await;
    }
}
