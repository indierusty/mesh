use kurbo::{BezPath, PathEl, Point};
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
    let points = vec![(10., 10.), (10., 100.)]
        .iter()
        .enumerate()
        .map(|(i, (x, y))| {
            if i == 0 {
                PathEl::MoveTo(Point::new(*x, *y))
            } else {
                PathEl::LineTo(Point::new(*x, *y))
            }
        })
        .collect::<Vec<PathEl>>();

    let bezpath = BezPath::from_vec(points);

    loop {
        clear_background(WHITE);
        next_frame().await
    }
}
