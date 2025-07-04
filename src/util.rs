use kurbo::{ParamCurve, Point};
use macroquad::{color::SKYBLUE, input::mouse_position, math::DVec2, shapes::draw_line};

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

pub fn draw_bez(segment: impl ParamCurve) {
    let mut t = 0.;
    let mut last_point: Option<Point> = None;
    while t <= 1. {
        let next_point = segment.eval(t);
        if let Some(last_point) = last_point {
            draw_line(
                last_point.x as f32,
                last_point.y as f32,
                next_point.x as f32,
                next_point.y as f32,
                2.,
                SKYBLUE,
            );
        }
        last_point = Some(next_point);
        t += 1e-3;
    }
}
