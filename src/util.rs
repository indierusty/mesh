use kurbo::{CubicBez, Line, ParamCurve, PathSeg, Point, QuadBez};
use macroquad::{
    color::{Color, SKYBLUE},
    input::mouse_position,
    math::DVec2,
    shapes::{draw_circle, draw_line},
};

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

pub fn point_to_gvec2(point: Point) -> glam::Vec2 {
    glam::Vec2::new(point.x as f32, point.y as f32)
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

pub fn points_to_segment(p1: Point, p2: Option<Point>, p3: Option<Point>, p4: Point) -> PathSeg {
    match (p2, p3) {
        (None, None) => PathSeg::Line(Line::new(p1, p4)),
        (Some(p2), None) | (None, Some(p2)) => PathSeg::Quad(QuadBez::new(p1, p2, p4)),
        (Some(p2), Some(p3)) => PathSeg::Cubic(CubicBez::new(p1, p2, p3, p4)),
    }
}

pub fn xdraw_circle(center: Point, r: f32, color: Color) {
    draw_circle(center.x as f32, center.y as f32, r, color);
}

pub fn xdraw_line(p1: Point, p2: Point, thickness: f32, color: Color) {
    draw_line(
        p1.x as f32,
        p1.y as f32,
        p2.x as f32,
        p2.y as f32,
        thickness,
        color,
    );
}
