use console_error_panic_hook::set_once;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use wasm_game_lib::inputs::event::types::*;
use wasm_game_lib::system::sleep;
use wasm_game_lib::{
    graphics::{color::Color, window::Window},
    inputs::{
        event::Event,
        keyboard::{Key, KeyboardEvent},
        mouse::{start_recording_mouse_events, get_mouse_position, is_mouse_pressed},
    },
};

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

mod items;
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

    let mut map = Map::new(&textures);
    let mut player = Player::new(&textures);
    let mut frame = 0;

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
                        Key::E => player.change_inventory_state(),
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
        }

        if is_mouse_pressed() {
            let (x, y) = crate::coords::screen_to_map(get_mouse_position().0 as f64, get_mouse_position().1 as f64, &player, screen_center);
            map[(x, y)] = crate::map::Block::Air;
        }

        player.handle_events(direction_keys, &map, frame);
        map.update_chunks(&player);

        canvas.clear_with_color(Color::cyan());
        map.draw_on_canvas(&mut canvas, &player, screen_center);
        player.draw_on_canvas(&mut canvas, screen_center, frame);

        sleep(Duration::from_millis(16)).await;
        frame += 1;
    }
}
