use macroquad::prelude::*;
fn conf() -> Conf {
    Conf {
        window_title: "BattleBlocks memory probe".into(),
        window_width: 1024,
        window_height: 720,
        high_dpi: false,
        ..Default::default()
    }
}
#[macroquad::main(conf)]
async fn main() {
    loop {
        clear_background(BLACK);
        next_frame().await;
    }
}
