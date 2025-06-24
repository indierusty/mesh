use std::collections::HashMap;

use kurbo::{BezPath, Point};

use crate::next_id::NextId;

pub struct MMesh {
    points: PointTable,
    segments: SegmentTable,
    next_id: NextId,
}

#[derive(Clone, Debug)]
pub struct PointTable {
    id: Vec<usize>,
    position: Vec<Point>,
}

impl PointTable {
    pub fn new() -> Self {
        Self {
            id: vec![],
            position: vec![],
        }
    }

    pub fn push(&mut self, id: usize, position: Point) {
        self.id.push(id);
        self.position.push(position);
    }
}

#[derive(Clone, Debug)]
pub struct SegmentTable {
    id: Vec<usize>,
    p1: Vec<usize>,
    p2: Vec<Option<usize>>,
    p3: Vec<Option<usize>>,
    p4: Vec<usize>,
}

impl SegmentTable {
    pub fn new() -> Self {
        Self {
            id: vec![],
            p1: vec![],
            p2: vec![],
            p3: vec![],
            p4: vec![],
        }
    }

    pub fn push(&mut self, id: usize, p1: usize, p2: Option<usize>, p3: Option<usize>, p4: usize) {
        self.id.push(id);
        self.p1.push(p1);
        self.p2.push(p2);
        self.p3.push(p3);
        self.p4.push(p4);
    }
}

impl MMesh {
    pub fn empty() -> Self {
        Self {
            points: PointTable::new(),
            segments: SegmentTable::new(),
            next_id: NextId::new(),
        }
    }

    pub fn append_point(&mut self, point: Point) -> usize {
        let id = self.next_id.next();
        self.points.push(id, point);
        id
    }

    pub fn append_segment(
        &mut self,
        p1: usize,
        p2: Option<usize>,
        p3: Option<usize>,
        p4: usize,
    ) -> Option<usize> {
        let mut points = [Some(p1), p2, p3, Some(p4)];

        // If any point is not in the points table then return [`None`].
        for point_id in &self.points.id {
            for points in &mut points {
                if points.is_some_and(|p| p == *point_id) {
                    *points = None;
                }
            }
        }
        if !points.iter().all(|p| p.is_none()) {
            return None;
        }

        let id = self.next_id.next();
        self.segments.push(id, p1, p2, p3, p4);
        Some(id)
    }

    pub fn closest_point(&self, position: Point) -> Option<(usize, Point)> {
        self.points.id.iter().zip(self.points.position.iter()).fold(
            None,
            |mut closest_point, (id, point)| {
                if point.distance(position) < 5. {
                    if !closest_point.is_some_and(|(_, ppoint)| {
                        point.distance(position) > ppoint.distance(position)
                    }) {
                        closest_point = Some((*id, *point))
                    }
                }
                closest_point
            },
        )
    }

    pub fn set_point(&mut self, point_id: usize, point_position: Point) {
        if let Some((_, point)) = self
            .points
            .id
            .iter()
            .zip(self.points.position.iter_mut())
            .find(|(id, _)| **id == point_id)
        {
            *point = point_position;
        }
    }

    pub fn set_segment(
        &mut self,
        id: usize,
        p1: usize,
        p2: Option<usize>,
        p3: Option<usize>,
        p4: usize,
    ) {
        if let Some((index, _)) = self
            .segments
            .id
            .iter()
            .enumerate()
            .find(|(_, this)| **this == id)
        {
            self.segments.p1[index] = p1;
            self.segments.p2[index] = p2;
            self.segments.p3[index] = p3;
            self.segments.p4[index] = p4;
        }
    }

    pub fn append_bezpath(&mut self, bezpath: &BezPath) {
        let mut last_point_id = None;

        for element in bezpath.elements() {
            match element {
                kurbo::PathEl::MoveTo(point) => {
                    let id = self.next_id.next();
                    self.points.push(id, *point);
                    last_point_id = Some(id);
                }
                kurbo::PathEl::LineTo(p4) => {
                    let p4_id = self.next_id.next();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_id.next();
                    let p1 = last_point_id.unwrap();
                    self.segments.push(segment_id, p1, None, None, p4_id);

                    last_point_id = Some(p4_id);
                }
                kurbo::PathEl::QuadTo(p3, p4) => {
                    let p3_id = self.next_id.next();
                    self.points.push(p3_id, *p3);

                    let p4_id = self.next_id.next();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_id.next();
                    let p1 = last_point_id.unwrap();
                    self.segments.push(segment_id, p1, None, Some(p3_id), p4_id);

                    last_point_id = Some(p4_id);
                }
                kurbo::PathEl::CurveTo(p2, p3, p4) => {
                    let p2_id = self.next_id.next();
                    self.points.push(p2_id, *p2);

                    let p3_id = self.next_id.next();
                    self.points.push(p3_id, *p3);

                    let p4_id = self.next_id.next();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_id.next();
                    let p1 = last_point_id.unwrap();
                    self.segments
                        .push(segment_id, p1, Some(p2_id), Some(p3_id), p4_id);

                    last_point_id = Some(p4_id);
                }
                kurbo::PathEl::ClosePath => {
                    last_point_id = None;
                    // TODO: Append multiple paths and close the path.
                }
            };
        }
    }

    pub fn to_bezpath(&self) -> BezPath {
        let mut bezpath = BezPath::new();
        if self.segments.id.is_empty() {
            return bezpath;
        }

        let segments_from_start =
            self.segments
                .p1
                .iter()
                .enumerate()
                .fold(HashMap::new(), |mut acc, (index, p1)| {
                    acc.insert(p1, index);
                    acc
                });

        let segments_from_end =
            self.segments
                .p4
                .iter()
                .enumerate()
                .fold(HashMap::new(), |mut acc, (index, p4)| {
                    acc.insert(p4, index);
                    acc
                });

        let mut start_point_id = *self.segments.p1.first().unwrap();
        let first_point_id = start_point_id;

        while segments_from_end.contains_key(&start_point_id) {
            let segment_index = segments_from_end.get(&start_point_id).unwrap();
            start_point_id = self.segments.p1[*segment_index];

            if start_point_id == first_point_id {
                break;
            }
        }

        let mut next_point_id = start_point_id;

        let points = self.points.id.iter().zip(self.points.position.iter()).fold(
            HashMap::new(),
            |mut points, (id, point)| {
                points.insert(id, point);
                points
            },
        );

        while let Some(segment_index) = segments_from_start.get(&next_point_id) {
            let p1 = self.segments.p1[*segment_index];
            let p2 = self.segments.p2[*segment_index];
            let p3 = self.segments.p3[*segment_index];
            let p4 = self.segments.p4[*segment_index];

            next_point_id = p4;

            let p1 = points.get(&p1).unwrap();
            let p2 = p2.and_then(|p2| points.get(&p2));
            let p3 = p3.and_then(|p3| points.get(&p3));
            let p4 = points.get(&p4).unwrap();

            if bezpath.elements().is_empty() {
                bezpath.move_to(**p1);
            }

            match [p2, p3] {
                [None, None] => bezpath.line_to(**p4),
                [Some(p2), None] | [None, Some(p2)] => bezpath.quad_to(**p2, **p4),
                [Some(p2), Some(p3)] => bezpath.curve_to(**p2, **p3, **p4),
            };
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
