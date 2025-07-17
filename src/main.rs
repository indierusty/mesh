use kurbo::{BezPath, Point};
use macroquad::prelude::*;
use mesh::{
    HEIGHT, WIDTH,
    mesh::{
        MMesh,
        planar::{XColor, calculate_and_draw_style},
    },
    path::Path,
    pen::Pen,
    util::mouse_position_point,
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
    // mesh.append_bezpath(&bezpath);

    let mut pen = Pen::new();
    let mut path = Path::new();

    let mut is_pen_active = true;
    // let mut edit_mesh = true;

    let result = mesh.to_bezpath();
    let mut styles = Vec::new();

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
        let mut setcolor = None;
        if is_key_pressed(KeyCode::R) {
            setcolor = Some((mouse_position_point(), XColor::Red));
        } else if is_key_pressed(KeyCode::G) {
            setcolor = Some((mouse_position_point(), XColor::Gray));
        } else if is_key_pressed(KeyCode::B) {
            setcolor = Some((mouse_position_point(), XColor::Blue));
        } else if is_key_pressed(KeyCode::Y) {
            setcolor = Some((mouse_position_point(), XColor::Yellow));
        } else if is_key_pressed(KeyCode::C) {
            setcolor = Some((mouse_position_point(), XColor::Blank));
        }
        if is_key_down(KeyCode::D) {
            println!("Planar Graph");
            let (new_mesh, parents) = mesh.planar_graph();
            let (regions, points) = new_mesh.calculate_regions();
            styles = calculate_and_draw_style(&regions, parents, &points, styles, setcolor);
        }
        if is_key_pressed(KeyCode::P) {
            mesh = mesh.planar_graph().0;
        }
        if is_key_pressed(KeyCode::Space) {
            is_pen_active = !is_pen_active;
            pen = Pen::new();
            path = Path::new();
        }
        next_frame().await
    }
}
