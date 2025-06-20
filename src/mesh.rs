use std::collections::HashMap;

use kurbo::{BezPath, CubicBez, Line, ParamCurve, PathSeg, Point, QuadBez};

use crate::next_id::NextId;

pub struct MMesh {
    points: Vec<MPoint>,
    segments: Vec<MSegment>,
    next_id: NextId,
}

#[derive(Clone, Copy, Debug)]
pub struct MPoint {
    id: usize,
    position: Point,
}

impl MPoint {
    pub fn new(id: usize, position: Point) -> Self {
        Self { id, position }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MSegment {
    id: (usize, usize),
    curve: MCurve,
}

impl MSegment {
    pub fn new(id: (usize, usize), curve: MCurve) -> Self {
        Self { id, curve }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum MCurve {
    Linear,
    Quad(Point),
    Cubic(Point, Point),
}

impl MMesh {
    pub fn empty() -> Self {
        Self {
            points: Vec::new(),
            segments: Vec::new(),
            next_id: NextId::new(),
        }
    }

    pub fn append_point(&mut self, point: Point) -> usize {
        let id = self.next_id.next();
        self.points.push(MPoint::new(id, point));
        id
    }

    pub fn append_segment(&mut self, start: usize, end: usize, curve: MCurve) -> Option<usize> {
        if self.points.iter().find(|point| point.id == start).is_none()
            || self.points.iter().find(|point| point.id == start).is_none()
        {
            return None;
        }
        let id = self.next_id.next();
        self.segments.push(MSegment::new((start, end), curve));
        Some(id)
    }

    pub fn closest_point(&self, position: Point) -> Option<MPoint> {
        self.points.iter().fold(None, |mut closest_point, point| {
            if point.position.distance(position) < 5. {
                if !closest_point.is_some_and(|ppoint| {
                    point.position.distance(position) > ppoint.position.distance(position)
                }) {
                    closest_point = Some(*point)
                }
            }
            closest_point
        })
    }

    pub fn set_point(&mut self, new_point: MPoint) {
        if let Some(point) = self
            .points
            .iter_mut()
            .find(|point| point.id == new_point.id)
        {
            point.position = new_point.position
        }
    }

    pub fn set_segment(&mut self, new_segment: MSegment) {
        if let Some(segment) = self
            .segments
            .iter_mut()
            .find(|segment| segment.id == new_segment.id)
        {
            segment.curve = new_segment.curve;
        }
    }

    pub fn append_bezpath(&mut self, bezpath: &BezPath) {
        let mut last_point = None;

        for element in bezpath.elements() {
            match element {
                kurbo::PathEl::MoveTo(point) => {
                    let id = self.next_id.next();
                    let new_point = MPoint::new(id, *point);
                    self.points.push(new_point);
                    last_point = Some(new_point);
                }
                kurbo::PathEl::LineTo(point) => {
                    let id = self.next_id.next();
                    let new_point = MPoint::new(id, *point);
                    self.points.push(new_point);

                    let prev_point = last_point.unwrap();
                    let curve = MCurve::Linear;
                    let segment = MSegment::new((prev_point.id, new_point.id), curve);
                    self.segments.push(segment);

                    last_point = Some(new_point);
                }
                kurbo::PathEl::QuadTo(point, point1) => {
                    let id = self.next_id.next();
                    let new_point = MPoint::new(id, *point1);
                    self.points.push(new_point);

                    let prev_point = last_point.unwrap();
                    let curve = MCurve::Quad(*point);
                    let segment = MSegment::new((prev_point.id, new_point.id), curve);
                    self.segments.push(segment);

                    last_point = Some(new_point);
                }
                kurbo::PathEl::CurveTo(point, point1, point2) => {
                    let id = self.next_id.next();
                    let new_point = MPoint::new(id, *point2);
                    self.points.push(new_point);

                    let prev_point = last_point.unwrap();
                    let curve = MCurve::Cubic(*point, *point1);
                    let segment = MSegment::new((prev_point.id, new_point.id), curve);
                    self.segments.push(segment);

                    last_point = Some(new_point);
                }
                kurbo::PathEl::ClosePath => {
                    last_point = None;
                    // TODO: Append multiple paths and close the path.
                }
            };
        }
    }

    pub fn to_bezpath(&self) -> BezPath {
        let mut bezpath = BezPath::new();
        if self.segments.is_empty() {
            return bezpath;
        }
        let mut segments_from_start = HashMap::new();
        let mut segments_from_end = HashMap::new();

        for segment in &self.segments {
            segments_from_start.insert(segment.id.0, segment);
            segments_from_end.insert(segment.id.1, segment);
        }

        let mut prev_point_id = self.segments.first().unwrap().id.0;
        let first_point_id = prev_point_id;

        while segments_from_end.contains_key(&prev_point_id) {
            prev_point_id = segments_from_end.get(&prev_point_id).unwrap().id.0;

            if prev_point_id == first_point_id {
                break;
            }
        }

        let mut next_point_id = prev_point_id;

        let points = self
            .points
            .iter()
            .fold(HashMap::new(), |mut points, point| {
                points.insert(point.id, *point);
                points
            });

        while let Some(segment) = segments_from_start.get(&next_point_id) {
            next_point_id = segment.id.1;

            let start_point = points.get(&segment.id.0).unwrap().position;
            let endpoint = points.get(&segment.id.1).unwrap().position;

            let path_segment = match segment.curve {
                MCurve::Linear => PathSeg::Line(Line::new(start_point, endpoint)),
                MCurve::Quad(point) => PathSeg::Quad(QuadBez::new(start_point, point, endpoint)),
                MCurve::Cubic(point, point1) => {
                    PathSeg::Cubic(CubicBez::new(start_point, point, point1, endpoint))
                }
            };

            if bezpath.elements().is_empty() {
                bezpath.move_to(path_segment.start());
            }
            bezpath.push(path_segment.as_path_el());
        }

        bezpath
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezpath_works() {
        let mut bezpath = BezPath::new();

        bezpath.move_to(Point::new(10., 15.));
        bezpath.quad_to(Point::new(100., 120.), Point::new(100., 200.));
        bezpath.quad_to(Point::new(200., 220.), Point::new(200., 300.));

        let mut mesh = MMesh::empty();
        mesh.append_bezpath(&bezpath);
        let result = mesh.to_bezpath();
        assert_eq!(result, bezpath);
    }
}
