use kurbo::Point;
use macroquad::{input::mouse_position, math::DVec2};

pub fn dvec2_to_point(point: DVec2) -> Point {
    Point {
        x: point.x,
        y: point.y,
    }
}

pub fn point_to_dvec2(point: Point) -> DVec2 {
    DVec2 {
        x: point.x,
        y: point.y,
    }
}

pub fn mouse_position_dvec2() -> DVec2 {
    let (x, y) = mouse_position();
    DVec2::new(x as f64, y as f64)
}

pub fn mouse_position_point() -> Point {
    let (x, y) = mouse_position();
    Point::new(x as f64, y as f64)
}
