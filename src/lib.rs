use wasm_bindgen::{prelude::*, JsCast};
use wasm_game_lib::{graphics::{image::Image, sprite::Sprite, window::Window, color::Color}, inputs::{mouse::start_recording_mouse_events, keyboard::KeyboardEvent, event::Event}};
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
use map::Map;
use textures::Textures;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_once(); // needed to see panic messages in the console of your web browser
    start_recording_mouse_events();

    let (mut window, mut canvas) =
        Window::init_with_events(KEYBOARD_EVENT + RESIZE_EVENT + MOUSE_EVENT);

    let textures = Textures::load(&mut canvas).await;

    let map = Map::new(&textures);
    let chara = Sprite::new((264.0, 0.0), &textures.character, (0.0,0.0));

    loop {
        for event in window.poll_events() {
            //log!("{:?}", event);
        }

        canvas.clear_with_color(Color::cyan());
        canvas.draw(&map);
        canvas.draw(&chara);
        //scanvas.draw(&units);

        sleep(Duration::from_millis(16)).await;
    }
}