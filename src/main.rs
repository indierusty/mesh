use kurbo::{BezPath, DEFAULT_ACCURACY, Point, Shape};
use macroquad::prelude::*;
use mesh::{HEIGHT, WIDTH, mesh::MMesh};

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
    let mut n = 5.;

    let mut bezpath = BezPath::new();

    bezpath.move_to(Point::new(10., 15.));
    bezpath.quad_to(Point::new(100., 120.), Point::new(100., 200.));
    bezpath.quad_to(Point::new(200., 220.), Point::new(200., 300.));

    let mut mesh = MMesh::empty();
    mesh.append_bezpath(&bezpath);
    let result = mesh.to_bezpath();

    println!("bezpath => {:#?}", bezpath);
    println!("result => {:#?}", result);

    loop {
        clear_background(WHITE);
        n += 1.;
        next_frame().await
    }
}
