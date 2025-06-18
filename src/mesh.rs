use std::collections::HashMap;

use kurbo::{BezPath, PathEl, Point};

use crate::next_id::NextId;

pub struct Mesh {
    points: Vec<(usize, Point)>,
    segments: Vec<(usize, Segment, usize)>,
    next_id: NextId,
}

pub enum Segment {
    Line,
    Quad(Point),
    Cubic(Point, Point),
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
        self.points.push((id, point));
        id
    }

    pub fn append_segment(&mut self, start: usize, end: usize, segment: Segment) -> Option<usize> {
        if self.points.iter().find(|(id, _)| *id == start).is_none()
            || self.points.iter().find(|(id, _)| *id == start).is_none()
        {
            return None;
        }
        let id = self.next_id.next();
        self.segments.push((start, segment, end));
        Some(id)
    }
}
