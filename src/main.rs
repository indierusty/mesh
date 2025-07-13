use std::collections::HashMap;

use kurbo::{BezPath, Point, Shape};
use macroquad::prelude::*;
use mesh::{
    HEIGHT, WIDTH,
    mesh::{
        MMesh, PointData, PointId, SegmentData,
        planar::{draw_region, points_id_to_segment},
    },
    path::Path,
    pen::Pen,
    util::{draw_bez, xdraw_circle},
};

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
    let mut region = (Vec::new(), HashMap::new());
    // mesh.append_bezpath(&bezpath);

    let mut pen = Pen::new();
    let mut path = Path::new();

    let mut is_pen_active = true;
    let mut edit_mesh = true;

    let result = mesh.to_bezpath();

    println!("bezpath => {:#?}", bezpath);
    println!("result => {:#?}", result);

    loop {
        clear_background(WHITE);
        // if edit_mesh {
        //     mesh.draw();
        // } else {
        //     pmesh.draw();
        // }
        mesh.draw();

        if is_pen_active {
            pen.update(&mut mesh);
            pen.draw(&mesh);
        } else {
            path.update(&mut mesh);
            path.draw(&mesh);
        }
        if is_key_down(KeyCode::P) {
            println!("Planar Graph");
            let (mmh, r) = mesh.planar_graph();
            draw_region(&r);
        }
        draw_region(&region);
        if is_key_pressed(KeyCode::Space) {
            is_pen_active = !is_pen_active;
            pen = Pen::new();
            path = Path::new();
        }
        next_frame().await
    }
}
