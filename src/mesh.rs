use kurbo::Point;

use crate::next_id::NextId;

pub struct Mesh {
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
    Cubic(Point),
}

impl Mesh {
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
}
