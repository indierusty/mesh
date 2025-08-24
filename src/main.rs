use macroquad::color::*;
use macroquad::prelude::*;
use mesh::dynamic::DynamicRegions;
use mesh::{
    HEIGHT, WIDTH, dynamic::intersection, mesh::MMesh, path::Path, pen::Pen,
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
    let mut mesh = MMesh::empty();
    // mesh.append_bezpath(&bezpath);

    let mut pen = Pen::new();
    let mut path = Path::new();

    let mut is_pen_active = true;
    // let mut edit_mesh = true;

    let result = mesh.to_bezpath();
    let mut dynamic = DynamicRegions::new();
    let mut prev_intersection_data = None;

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
            setcolor = Some((mouse_position_point(), RED));
        } else if is_key_pressed(KeyCode::G) {
            setcolor = Some((mouse_position_point(), GRAY));
        } else if is_key_pressed(KeyCode::B) {
            setcolor = Some((mouse_position_point(), BLUE));
        } else if is_key_pressed(KeyCode::Y) {
            setcolor = Some((mouse_position_point(), YELLOW));
        } else if is_key_pressed(KeyCode::C) {
            setcolor = Some((mouse_position_point(), BLACK));
        }
        if is_key_down(KeyCode::D) {
            println!("Planar Graph");
            // let (new_mesh, parents) = mesh.planar_graph();
            // let (regions, points) = new_mesh.calculate_regions();
            // styles = calculate_and_draw_style(&regions, parents, &points, styles, setcolor);
            let intersection = intersection(&mesh);
            prev_intersection_data = Some(intersection.clone());
            intersection.draw();
            let mut regions = DynamicRegions::build(intersection)
                .style(dynamic.clone())
                .filter_outer_regions();
            if let Some((position, color)) = setcolor {
                regions.apply_style(Some(color), position);
            }
            regions.render();
            dynamic = regions.clone();
        }
        if is_key_pressed(KeyCode::P) {
            // mesh = mesh.planar_graph().0;
        }
        if is_key_pressed(KeyCode::Space) {
            is_pen_active = !is_pen_active;
            pen = Pen::new();
            path = Path::new();
        }
        // let (x, y) = mouse_position();
        // draw_text(&format!("({:.2}, {:.2})", x, y), x, y, 20., BLACK);

        // if let Some(data) = &prev_intersection_data {
        //     for i in 0..data.segments.len() {
        //         let mid = data.segments[i].eval(0.5);
        //         draw_text(&format!("{}", i), mid.x as f32, mid.y as f32, 18., BLACK);
        //     }
        // }
        next_frame().await
    }
}
