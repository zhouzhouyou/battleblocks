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
        // The web build fetches an atlas or sheet when the session first asks
        // for it; the loading screen replaces the frame that asked, so a blank
        // stand-in is never presented.
        #[cfg(target_arch = "wasm32")]
        if session.needs_assets() {
            assets::pump();
            draw_loading_screen();
            next_frame().await;
            session.refresh_textures();
            continue;
        }
        session.frame();
        #[cfg(target_arch = "wasm32")]
        if session.needs_assets() {
            assets::pump();
            draw_loading_screen();
            next_frame().await;
            session.refresh_textures();
            continue;
        }
        next_frame().await;
    }
}

/// Web-only loading screen: sprite atlases and effect sheets ship beside the
/// `.wasm`, so they are fetched when the session first needs them.
#[cfg(target_arch = "wasm32")]
fn draw_loading_screen() {
    clear_background(Color::from_rgba(25, 27, 38, 255));
    let bar = Rect::new(312., 348., 400., 24.);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, Color::from_rgba(48, 53, 64, 255));
    let pulse = ((get_time() * 1.6).sin() * 0.5 + 0.5) as f32;
    draw_rectangle(bar.x, bar.y, bar.w * (0.15 + 0.7 * pulse), bar.h, GOLD);
    label_centered(tr_key(true, "ui.loading.assets"), 512., 330., 24., WHITE);
    flush_labels();
}
#[cfg(test)]
mod tests;
