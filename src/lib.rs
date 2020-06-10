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
        mouse::{start_recording_mouse_events, get_mouse_position, is_pressed, Button, MouseEvent},
    },
};

mod items;
mod coords;
mod loader;
mod map;
mod player;
mod progress_bar;
mod textures;
mod blocks;
use blocks::{Block, BlockType};
use map::Map;
use player::Player;
use textures::Textures;

#[wasm_bindgen(start)]
pub async fn start() -> Result<(), JsValue> {
    set_once();
    start_recording_mouse_events();

    let (mut window, mut canvas) =
        Window::init_with_events(KEYBOARD_EVENT + MOUSE_EVENT);

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
                Event::MouseEvent(event) => match event {
                    MouseEvent::Scroll(_, movement, _, _) => {
                        if movement > 0.0 && player.selected_slot < 8 {
                            player.selected_slot += 1;
                        } else if movement < 0.0 && player.selected_slot > 0 {
                            player.selected_slot -= 1;
                        }
                    }
                    _ => (),
                }
                _ => (),
            }
        }

        if is_pressed(Button::Main) {
            let (x, y) = crate::coords::screen_to_map(get_mouse_position().0 as f64, get_mouse_position().1 as f64, &player, screen_center);
            if map[(x, y)].block_type != BlockType::Air {
                let items = map[(x, y)].as_item();
                for item in items {
                    player.inventory.push(item);
                }
            }
            map[(x, y)].block_type = BlockType::Air;
        }

        if is_pressed(Button::Secondary) {
            let (x, y) = crate::coords::screen_to_map(get_mouse_position().0 as f64, get_mouse_position().1 as f64, &player, screen_center);
            if map[(x, y)].block_type == BlockType::Air {
                if let Some((item, quantity)) = &mut player.inventory[player.selected_slot as usize] {
                    if *quantity > 0 {
                        if let Some(block) = item.as_block() {
                            *quantity -= 1;
                            if *quantity == 0 {
                                player.inventory[player.selected_slot as usize] = None;
                            }
                            map[(x, y)].block_type = block;
                        }
                    }
                }
            }
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
