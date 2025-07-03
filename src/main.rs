use kurbo::{BezPath, Point};
use macroquad::prelude::*;
use mesh::{HEIGHT, WIDTH, mesh::MMesh, pen::Pen};

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

    bezpath.move_to(Point::new(10., 15.));
    bezpath.quad_to(Point::new(100., 120.), Point::new(100., 200.));
    bezpath.quad_to(Point::new(200., 220.), Point::new(200., 300.));

    let mut mesh = MMesh::empty();
    // mesh.append_bezpath(&bezpath);
    let mut pen = Pen::new();
    let result = mesh.to_bezpath();

    println!("bezpath => {:#?}", bezpath);
    println!("result => {:#?}", result);

    loop {
        clear_background(WHITE);
        mesh.draw();
        pen.update(&mut mesh);
        pen.draw(&mesh);
        next_frame().await
    }
}
