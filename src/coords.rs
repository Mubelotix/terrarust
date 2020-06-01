use crate::player::Player;

pub fn _screen_to_map(_x: isize, _y: isize) -> (isize, isize) {
    todo!();
}

pub fn map_to_screen(
    x: isize,
    y: isize,
    player: &Player,
    screen_center: (isize, isize),
) -> (f64, f64) {
    let diff_x = player.x - x as f64;
    let diff_y = player.y - y as f64;

    (screen_center.0 as f64 - diff_x * 16.0, screen_center.1 as f64 - diff_y * 16.0)
}
