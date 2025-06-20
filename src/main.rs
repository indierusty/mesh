use kurbo::{BezPath, DEFAULT_ACCURACY, Point, Shape};
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
    let mut bezpath = BezPath::new();
    bezpath.move_to(Point::new(2685.8597041876837, -1251.8357401225196));
    bezpath.quad_to(
        Point::new(2253.49552534553, -1301.7456717617613),
        Point::new(2253.49552534553, -1303.7456717617613),
    );
    for seg in bezpath.segments() {
        let len = seg.perimeter(0.1);
        println!("len {}", len);
    }
    loop {
        clear_background(WHITE);
        next_frame().await
    }
}
