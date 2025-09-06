use macroquad::color::*;
use macroquad::prelude::*;
use mesh::dynamic::DynamicData;
use mesh::tools::Tools;
use mesh::{HEIGHT, WIDTH, dynamic::intersection, mesh::DynamicMesh, util::mouse_position_point};

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
    let mut mesh = DynamicMesh::empty();
    // mesh.append_bezpath(&bezpath);
    // let result = mesh.to_bezpath();

    let mut tools = Tools::new();

    let mut clock = 0.;
    loop {
        tools.update(&mut mesh);

        if clock > 1. / 10. {
            clock = 0.;
            println!(
                "------------------------------------------------------------------------------\n"
            );

            let intersection = intersection(&mesh);
            // intersection.draw();
            let dynamic_data = DynamicData::build(intersection)
                .filter_outer_regions()
                .dynamic(mesh.dynamic_data.clone());

            println!("Dynamic Data \n{:#?}", dynamic_data);

            mesh.dynamic_data = dynamic_data.clone();

            println!(
                "------------------------------------------------------------------------------\n"
            );
        }
        clock += get_frame_time();

        clear_background(WHITE);

        mesh.draw();
        mesh.dynamic_data.render();
        tools.draw(&mesh);

        next_frame().await
    }
}
