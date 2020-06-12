use crate::{chunks::Biome, player::Player};
use std::hash::Hasher;
use twox_hash::XxHash32;
use wasm_game_lib::elog;

pub fn screen_to_map(
    x: f64,
    y: f64,
    player: &Player,
    screen_center: (isize, isize),
) -> (isize, isize) {
    let a = screen_center.0 as f64 - x;
    let b = screen_center.1 as f64 - y;
    let diff_x = a / 16.0;
    let diff_y = b / 16.0;
    let x = player.x - diff_x;
    let y = player.y - diff_y;
    (x.floor() as isize, y.floor() as isize)
}

pub fn map_to_screen(
    x: isize,
    y: isize,
    player: &Player,
    screen_center: (isize, isize),
) -> (f64, f64) {
    let diff_x = player.x - x as f64;
    let diff_y = player.y - y as f64;

    (
        screen_center.0 as f64 - diff_x * 16.0,
        screen_center.1 as f64 - diff_y * 16.0,
    )
}

pub fn x_to_chunk(x: isize) -> isize {
    x_to_chunk_and_column(x).0
}

pub fn x_to_chunk_and_column(x: isize) -> (isize, isize) {
    let mut column_index = x % 32;

    let mut chunk_number = (x - column_index) / 32;
    if x < 0 && column_index < 0 {
        chunk_number -= 1;
        column_index += 32;
    }

    (chunk_number, column_index)
}

pub fn x_to_biome(x: isize) -> Biome {
    let mut chunk = x_to_chunk(x);
    chunk -= chunk % 8;
    chunk /= 8;
    let mut hasher = XxHash32::with_seed(42);
    hasher.write_isize(chunk);
    let hash = hasher.finish();

    match hash % 3 {
        0 => Biome::Hills,
        1 => Biome::Grassland,
        2 => Biome::TemperateBroadleafForest,
        i => {
            elog!("ERROR! Pattern {} not covered in x_to_biome function", i);
            Biome::Hills
        }
    }
}
