use std::collections::HashMap;

use kurbo::{BezPath, PathEl, Point};

use crate::next_id::NextId;

pub type PointId = usize;
pub type SegmentId = (PointId, PointId);

pub struct Mesh {
    points: HashMap<PointId, Point>,
    segments: HashMap<SegmentId, Segment>,
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
            points: HashMap::new(),
            segments: HashMap::new(),
            next_id: NextId::new(),
        }
    }

    pub fn append_bezpath(&mut self, bezpath: BezPath) {
        let mut last_points_id = (None, None);

        for elm in bezpath.elements() {
            if *elm == PathEl::ClosePath {
            } else {
                let next_id = self.next_id.next();
                self.points.insert(next_id, elm.end_point().unwrap());
                last_points_id.0 = last_points_id.1;
                last_points_id.1 = Some(next_id);

                match elm {
                    PathEl::QuadTo(point, _) => {
                        let segment_id = (last_points_id.0.unwrap(), last_points_id.1.unwrap());
                        self.segments.insert(segment_id, Segment::Quad(*point));
                    }
                    PathEl::CurveTo(point, point1, _) => {
                        let segment_id = (last_points_id.0.unwrap(), last_points_id.1.unwrap());
                        self.segments
                            .insert(segment_id, Segment::Cubic(*point, *point1));
                    }
                    _ => {}
                }
            }
        }
    }
}
