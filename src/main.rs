use macroquad::prelude::*;
use mesh::{HEIGHT, WIDTH};

fn conf() -> Conf {
    Conf {
        window_title: "Mesh".to_string(),
        window_width: WIDTH,
        window_height: HEIGHT,
        ..Default::default()
    }
}

#[macroquad::main(conf)]
async fn main() {
    loop {
        clear_background(WHITE);
        next_frame().await
    }
}
