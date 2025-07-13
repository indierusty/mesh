pub mod planar;

use std::collections::{HashMap, HashSet};

use kurbo::{BezPath, CubicBez, Line, ParamCurve, PathSeg, Point, QuadBez, Shape};
use macroquad::prelude::*;
use planar::{Direction, points_id_to_segment};

use crate::{algo::path_intersections, next_id::NextId, util::xdraw_circle};

#[derive(Debug, Clone)]
pub struct MMesh {
    points: PointTable,
    segments: SegmentTable,
    next_id: NextId,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PointId(usize);

impl PointId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PointIndex(usize);

impl PointIndex {
    pub fn index(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PointData {
    idx: PointIndex,
    id: PointId,
    position: Point,
}

impl PointData {
    pub fn new(idx: PointIndex, id: PointId, position: Point) -> Self {
        Self { idx, id, position }
    }
}

#[derive(Clone, Debug)]
pub struct PointTable {
    id: Vec<PointId>,
    position: Vec<Point>,
}

impl PointTable {
    pub fn new() -> Self {
        Self {
            id: vec![],
            position: vec![],
        }
    }

    pub fn data(&self) -> Vec<PointData> {
        (0..self.id.len())
            .into_iter()
            .map(|idx| PointData::new(PointIndex(idx), self.id[idx], self.position[idx]))
            .collect()
    }

    pub fn push(&mut self, id: PointId, position: Point) {
        self.id.push(id);
        self.position.push(position);
    }

    pub fn remove(&mut self, id: PointId) {
        let Some(index) = self
            .id
            .iter()
            .enumerate()
            .find(|(_, this_id)| **this_id == id)
            .map(|(i, _)| i)
        else {
            return;
        };

        self.id.remove(index);
        self.position.remove(index);
    }

    pub fn remove_multiple(&mut self, points: &HashSet<PointId>) {
        println!("removing multiple points");
        let mut new = Self::new();
        for i in 0..self.id.len() {
            if !points.contains(&self.id[i]) {
                new.id.push(self.id[i]);
                new.position.push(self.position[i]);
            }
        }
        *self = new;
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SegmentId(usize);

impl SegmentId {
    pub fn id(&self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SegmentIndex(usize);

impl SegmentIndex {
    pub fn idx(&self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, Copy)]
pub struct SegmentData {
    idx: SegmentIndex,
    id: SegmentId,
    p1: PointId,
    p2: Option<PointId>,
    p3: Option<PointId>,
    p4: PointId,
    parent: Option<SegmentId>,
    direction: Option<Direction>,
}

impl SegmentData {
    pub fn new(
        idx: SegmentIndex,
        id: SegmentId,
        p1: PointId,
        p2: Option<PointId>,
        p3: Option<PointId>,
        p4: PointId,
        parent: Option<SegmentId>,
        direction: Option<Direction>,
    ) -> Self {
        Self {
            idx,
            id,
            p1,
            p2,
            p3,
            p4,
            parent,
            direction,
        }
    }

    pub fn to_path_seg(&self, points: &HashMap<PointId, PointData>) -> PathSeg {
        let p1 = points.get(&self.p1).unwrap().position;
        let p2 = self.p2.and_then(|p2| points.get(&p2));
        let p3 = self.p3.and_then(|p3| points.get(&p3));
        let p4 = points.get(&self.p4).unwrap().position;

        match (p2, p3) {
            (Some(p2), Some(p3)) => PathSeg::Cubic(CubicBez::new(p1, p2.position, p3.position, p4)),
            (Some(p2), None) | (None, Some(p2)) => PathSeg::Quad(QuadBez::new(p1, p2.position, p4)),
            (None, None) => PathSeg::Line(Line::new(p1, p4)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SegmentTable {
    id: Vec<SegmentId>,
    p1: Vec<PointId>,
    p2: Vec<Option<PointId>>,
    p3: Vec<Option<PointId>>,
    p4: Vec<PointId>,
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

    pub fn data(&self) -> Vec<SegmentData> {
        (0..self.id.len())
            .into_iter()
            .map(|idx| {
                SegmentData::new(
                    SegmentIndex(idx),
                    self.id[idx],
                    self.p1[idx],
                    self.p2[idx],
                    self.p3[idx],
                    self.p4[idx],
                    None,
                    None,
                )
            })
            .collect()
    }

    pub fn push(
        &mut self,
        id: SegmentId,
        p1: PointId,
        p2: Option<PointId>,
        p3: Option<PointId>,
        p4: PointId,
    ) {
        self.id.push(id);
        self.p1.push(p1);
        self.p2.push(p2);
        self.p3.push(p3);
        self.p4.push(p4);
    }

    pub fn remove(&mut self, id: SegmentId) {
        let Some(index) = self
            .id
            .iter()
            .enumerate()
            .find(|(_, this_id)| **this_id == id)
            .map(|(i, _)| i)
        else {
            return;
        };

        self.id.remove(index);
        self.p1.remove(index);
        self.p2.remove(index);
        self.p3.remove(index);
        self.p4.remove(index);
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

    pub fn next_point_id(&mut self) -> PointId {
        PointId(self.next_id.next())
    }

    pub fn next_segment_id(&mut self) -> SegmentId {
        SegmentId(self.next_id.next())
    }

    pub fn points_map(&self) -> HashMap<PointId, PointData> {
        self.points
            .data()
            .iter()
            .fold(HashMap::new(), |mut acc, data| {
                acc.insert(data.id, *data);
                acc
            })
    }

    pub fn segments_map(&self) -> HashMap<SegmentId, SegmentData> {
        self.segments
            .data()
            .iter()
            .fold(HashMap::new(), |mut map, data| {
                map.insert(data.id, *data);
                map
            })
    }

    pub fn segment(&self, index: SegmentIndex, points: &HashMap<PointId, PointData>) -> PathSeg {
        let p1_id = self.segments.p1[index.idx()];
        let p2_id = self.segments.p2[index.idx()];
        let p3_id = self.segments.p3[index.idx()];
        let p4_id = self.segments.p4[index.idx()];

        let p1 = points.get(&p1_id).unwrap().position;
        let p2 = p2_id.and_then(|id| points.get(&id)).map(|p| p.position);
        let p3 = p3_id.and_then(|id| points.get(&id)).map(|p| p.position);
        let p4 = points.get(&p4_id).unwrap().position;

        match (p2, p3) {
            (Some(p2), Some(p3)) => PathSeg::Cubic(CubicBez::new(p1, p2, p3, p4)),
            (Some(p2), None) | (None, Some(p2)) => PathSeg::Quad(QuadBez::new(p1, p2, p4)),
            (None, None) => PathSeg::Line(Line::new(p1, p4)),
        }
    }

    pub fn append_point(&mut self, point: Point) -> PointId {
        let id = self.next_point_id();
        self.points.push(id, point);
        id
    }

    pub fn append_segment(
        &mut self,
        p1: PointId,
        p2: Option<PointId>,
        p3: Option<PointId>,
        p4: PointId,
    ) -> Option<SegmentId> {
        let mut points = [Some(p1), p2, p3, Some(p4)];

        // If any point is not in the points table then return [`None`].
        for point_id in &self.points.id {
            for points in &mut points {
                if points.is_some_and(|p| p.id() == point_id.id()) {
                    *points = None;
                }
            }
        }
        if !points.iter().all(|p| p.is_none()) {
            return None;
        }

        let id = self.next_segment_id();
        self.segments.push(id, p1, p2, p3, p4);
        Some(id)
    }

    pub fn closest_point(&self, point: Point, max_radius: Option<f64>) -> Option<(PointId, Point)> {
        self.points.id.iter().zip(self.points.position.iter()).fold(
            None,
            |mut closest_point, (next_point_id, next_point)| {
                if next_point.distance(point) < max_radius.unwrap_or(5.) {
                    closest_point = match closest_point {
                        Some((_, closest))
                            if next_point.distance(closest) < point.distance(closest) =>
                        {
                            Some((*next_point_id, *next_point))
                        }
                        None => Some((*next_point_id, *next_point)),
                        x => x,
                    };
                }

                closest_point
            },
        )
    }

    pub fn set_point(&mut self, point_id: PointId, point_position: Point) {
        if let Some((_, point)) = self
            .points
            .id
            .iter()
            .zip(self.points.position.iter_mut())
            .find(|(id, _)| id.id() == point_id.id())
        {
            *point = point_position;
        }
    }

    pub fn get_point(&self, point_id: PointId) -> Option<Point> {
        self.points
            .id
            .iter()
            .enumerate()
            .find(|(_, id)| id.id() == point_id.id())
            .and_then(|(index, _)| self.points.position.get(index).copied())
    }

    pub fn remove_floating_point(&mut self, point_id: PointId) {
        let mut is_floating_point = true;
        for segment_index in 0..self.segments.id.len() {
            if self.segments.p1[segment_index] == point_id
                || self.segments.p2[segment_index] == Some(point_id)
                || self.segments.p3[segment_index] == Some(point_id)
                || self.segments.p4[segment_index] == point_id
            {
                is_floating_point = false;
            }
        }

        if is_floating_point {
            self.points.remove(point_id);
        }
    }

    pub fn set_segment(
        &mut self,
        id: SegmentId,
        p1: PointId,
        p2: Option<PointId>,
        p3: Option<PointId>,
        p4: PointId,
    ) {
        if let Some((index, _)) = self
            .segments
            .id
            .iter()
            .enumerate()
            .find(|(_, this)| this.id() == id.id())
        {
            self.segments.p1[index] = p1;
            self.segments.p2[index] = p2;
            self.segments.p3[index] = p3;
            self.segments.p4[index] = p4;
        }
    }

    pub fn remove_segment(&mut self, id: SegmentId) {
        let Some(index) = self
            .segments
            .id
            .iter()
            .enumerate()
            .find(|(_, this_id)| **this_id == id)
            .map(|(i, _)| i)
        else {
            return;
        };

        let p1 = self.segments.p1[index];
        let p2 = self.segments.p2[index];
        let p3 = self.segments.p3[index];
        let p4 = self.segments.p4[index];

        self.segments.remove(id);

        self.remove_floating_point(p1);
        self.remove_floating_point(p4);

        if let Some(p2) = p2 {
            self.points.remove(p2);
        }
        if let Some(p3) = p3 {
            self.points.remove(p3);
        }
    }

    pub fn append_bezpath(&mut self, bezpath: &BezPath) {
        let mut last_point_id = None;

        for element in bezpath.elements() {
            match element {
                kurbo::PathEl::MoveTo(point) => {
                    let id = self.next_point_id();
                    self.points.push(id, *point);
                    last_point_id = Some(id);
                }
                kurbo::PathEl::LineTo(p4) => {
                    let p4_id = self.next_point_id();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_segment_id();
                    let p1 = last_point_id.unwrap();
                    self.segments.push(segment_id, p1, None, None, p4_id);

                    last_point_id = Some(p4_id);
                }
                kurbo::PathEl::QuadTo(p3, p4) => {
                    let p3_id = self.next_point_id();
                    self.points.push(p3_id, *p3);

                    let p4_id = self.next_point_id();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_segment_id();
                    let p1 = last_point_id.unwrap();
                    self.segments.push(segment_id, p1, None, Some(p3_id), p4_id);

                    last_point_id = Some(p4_id);
                }
                kurbo::PathEl::CurveTo(p2, p3, p4) => {
                    let p2_id = self.next_point_id();
                    self.points.push(p2_id, *p2);

                    let p3_id = self.next_point_id();
                    self.points.push(p3_id, *p3);

                    let p4_id = self.next_point_id();
                    self.points.push(p4_id, *p4);

                    let segment_id = self.next_segment_id();
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

    pub fn draw(&self) {
        for point in &self.points.position {
            draw_circle(point.x as f32, point.y as f32, 3., RED);
        }
        let points = self.points.id.iter().zip(self.points.position.iter()).fold(
            HashMap::new(),
            |mut acc, (id, pos)| {
                acc.insert(*id, *pos);
                acc
            },
        );

        let mut segments = Vec::new();

        for index in 0..self.segments.id.len() {
            let p1_id = self.segments.p1[index];
            let p2_id = self.segments.p2[index];
            let p3_id = self.segments.p3[index];
            let p4_id = self.segments.p4[index];

            let p1 = points.get(&p1_id).unwrap();
            let p2 = p2_id.and_then(|id| points.get(&id));
            let p3 = p3_id.and_then(|id| points.get(&id));
            let p4 = points.get(&p4_id).unwrap();

            let segment = match (p2, p3) {
                (Some(p2), Some(p3)) => PathSeg::Cubic(CubicBez::new(*p1, *p2, *p3, *p4)),
                (Some(p2), None) | (None, Some(p2)) => PathSeg::Quad(QuadBez::new(*p1, *p2, *p4)),
                (None, None) => PathSeg::Line(Line::new(*p1, *p4)),
            };

            segments.push(segment);

            let mut last_point: Option<Point> = None;
            let mut t = 0.;
            loop {
                if t > 1. {
                    break;
                }
                let next_point = segment.eval(t);
                if let Some(last_point) = last_point {
                    draw_line(
                        last_point.x as f32,
                        last_point.y as f32,
                        next_point.x as f32,
                        next_point.y as f32,
                        2.,
                        BLACK,
                    );
                }
                last_point = Some(next_point);
                t += 1e-3;
            }
        }

        // for i in 0..segments.len() {
        //     for j in i + 1..segments.len() {
        //         for t in path_intersections(segments[i], segments[j]) {
        //             let p = segments[i].eval(t);
        //             xdraw_circle(p, 5., GREEN);
        //         }
        //     }
        // }
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
