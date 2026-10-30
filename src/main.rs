mod board;
#[cfg(test)]
use board::drop_board;
mod content;
#[cfg(test)]
use board::valid_swap;
use board::{fresh, matches, parallel_swap, swap, Board, N};
mod debug;
mod effects;
mod icons;
mod menu;
mod presentation;
#[allow(dead_code)]
mod role_state;
mod ui;
use presentation::{CLEAR_TIME, TURN_HOLD};
mod rng;
mod strategy;
mod turns;
use content::{effect, HEROES, VALUES};
struct SpriteAsset {
    bytes: &'static [u8],
    w: f32,
    h: f32,
    durations: &'static [f64],
    indices: &'static [usize],
}
mod audio;
mod sprites;
use macroquad::prelude::*;
mod graphics;
use graphics::*;
mod app;
fn conf() -> macroquad::conf::Conf {
    let (window_width, window_height) = if diagnostic("small_window") {
        (512, 360)
    } else {
        ui::WINDOW_SIZES[1]
    };
    let window = Conf {
        window_title: "BattleBlocks".into(),
        window_width,
        window_height,
        high_dpi: diagnostic("hidpi"),
        ..Default::default()
    };
    macroquad::conf::Conf {
        miniquad_conf: window,
        draw_call_vertex_capacity: if diagnostic("large_buffers") {
            10000
        } else {
            512
        },
        draw_call_index_capacity: if diagnostic("large_buffers") {
            5000
        } else {
            768
        },
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    init_font();
    let mut session = app::Session::new();
    session.start();
    loop {
        session.frame();
        next_frame().await;
    }
}
#[cfg(test)]
mod tests;
