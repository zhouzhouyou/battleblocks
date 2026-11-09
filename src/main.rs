mod board;
#[cfg(test)]
use board::drop_board;
mod content;
#[cfg(test)]
use board::matches;
#[cfg(test)]
use board::valid_swap;
use board::{fresh, parallel_swap, swap, Board, N};
mod debug;
mod effects;
mod icons;
mod menu;
mod presentation;
#[allow(dead_code)]
mod role_state;
mod slingshot_art;
mod ui;
use presentation::{CLEAR_TIME, TURN_HOLD};
mod rng;
mod strategy;
mod turns;
use content::{effect, HEROES, VALUES};
use sprites::SpriteAsset;
mod audio;
mod sprites;
use macroquad::prelude::*;
mod graphics;
use graphics::*;
mod app;
mod assets;
mod headless;
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

fn main() {
    if std::env::args().any(|arg| arg == "--headless" || arg == "--autoplay") {
        headless::run();
    } else {
        macroquad::Window::from_config(conf(), gui_main());
    }
}

async fn gui_main() {
    init_font();
    let mut session = app::Session::new();
    session.start();
    loop {
        // The web build fetches atlases and sheets in the background; the menu
        // and battle keep drawing, with a summoning stand-in for missing artwork.
        #[cfg(target_arch = "wasm32")]
        if assets::pump() {
            session.refresh_textures();
        }
        session.frame();
        next_frame().await;
    }
}

#[cfg(test)]
mod tests;
