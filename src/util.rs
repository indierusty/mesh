use std::{collections::HashMap, f64};

use kurbo::{CubicBez, Line, ParamCurve, ParamCurveDeriv, PathSeg, Point, QuadBez};
use macroquad::{
    color::{Color, SKYBLUE},
    input::mouse_position,
    math::DVec2,
    shapes::{draw_circle, draw_line},
};

use crate::{
    dynamic::Direction,
    mesh::{PointData, PointId, SegmentData},
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

pub fn segment_data_to_pathseg(
    points_data: &HashMap<PointId, PointData>,
    segment_data: SegmentData,
    direction: Direction,
) -> PathSeg {
    let (p1, p2, p3, p4) = match direction {
        Direction::StartToEnd => (
            segment_data.p1,
            segment_data.p2,
            segment_data.p3,
            segment_data.p4,
        ),
        Direction::EndToStart => (
            segment_data.p4,
            segment_data.p3,
            segment_data.p2,
            segment_data.p1,
        ),
    };
    let p1 = points_data.get(&p1).unwrap().position;
    let p2 = p2.and_then(|p2| points_data.get(&p2)).map(|p| p.position);
    let p3 = p3.and_then(|p3| points_data.get(&p3)).map(|p| p.position);
    let p4 = points_data.get(&p4).unwrap().position;
    points_to_segment(p1, p2, p3, p4)
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

pub fn pathseg_tangent(segment: PathSeg, t: f64) -> DVec2 {
    // NOTE: .deriv() method gives inaccurate result when it is 1.
    let t = t.max(0.98).min(0.02);

    let tangent = match segment {
        PathSeg::Line(line) => line.deriv().eval(t),
        PathSeg::Quad(quad_bez) => quad_bez.deriv().eval(t),
        PathSeg::Cubic(cubic_bez) => cubic_bez.deriv().eval(t),
    };

    DVec2::new(tangent.x, tangent.y)
}

pub fn xdraw_segment(segment: PathSeg, color: Color) {
    let mut t = 0.;
    while t < 1. {
        let start = segment.eval(t);
        let end = segment.eval(t + 0.05);
        xdraw_line(start, end, 2., color);
        t += 0.05
    }
}
