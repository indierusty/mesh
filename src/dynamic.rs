use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
    fmt::{Display, Write},
};

use kurbo::{BezPath, Line, ParamCurve, PathSeg, Point, Rect, Shape};
use macroquad::{
    color::{BLACK, Color},
    math::DVec2,
    shapes::draw_line,
};

use crate::{
    MIN_SEPARATION,
    algo::{cleanup_intersections, pathseg_intersections},
    mesh::{DynamicMesh, SegmentId},
    util::{segment_data_to_pathseg, xdraw_circle},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    StartToEnd,
    EndToStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Part {
    First,
    Second,
}

#[derive(Clone)]
pub struct IntersectData {
    /// Sub-segments
    pub segments: Vec<PathSeg>,
    /// Parent segments id and part of the cubic segment for each sub-segment.
    pub parents: Vec<(SegmentId, Option<Part>)>,
}

impl Display for IntersectData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        for (seg, par) in self.segments.iter().zip(self.parents.iter()) {
            f.write_str(&format!("{:?}\n", (seg, par)))?;
        }
        f.write_char('[')?;
        Ok(())
    }
}

impl IntersectData {
    fn new() -> Self {
        Self {
            segments: Vec::new(),
            parents: Vec::new(),
        }
    }

    fn push(&mut self, segment: PathSeg, parent: (SegmentId, Option<Part>)) {
        self.segments.push(segment);
        self.parents.push(parent);
    }

    pub fn draw(&self) {
        println!("intersection");
        println!("len: {}", self.segments.len());
        for seg in &self.segments {
            xdraw_circle(seg.start(), 3., BLACK);
            xdraw_circle(seg.end(), 3., BLACK);
        }
    }
}

pub fn intersection(mesh: &DynamicMesh) -> IntersectData {
    let segments_data = mesh
        .segments_points()
        .iter()
        .map(|(seg_id, segment_data)| match segment_data.to_pathseg() {
            PathSeg::Cubic(cubic_bez) => {
                let (first, second) = cubic_bez.subdivide();
                [
                    Some((*seg_id, Some(Part::First), PathSeg::Cubic(first))),
                    Some((*seg_id, Some(Part::Second), PathSeg::Cubic(second))),
                ]
            }
            segment => [Some((*seg_id, None, segment)), None],
        })
        .flatten()
        .flatten()
        .fold(HashMap::new(), |mut acc, (seg_id, part, segment)| {
            acc.insert((seg_id, part), segment);
            acc
        });

    // get all the intersection for each segment with every other segment in the mesh.
    let mut segments_intersections = Vec::new();

    for iseg_id in segments_data.keys() {
        let mut intersections = Vec::new();
        for jseg_id in segments_data.keys() {
            if iseg_id == jseg_id {
                continue;
            }
            let iseg = segments_data[iseg_id];
            let jseg = segments_data[jseg_id];

            let mut intersection = pathseg_intersections(iseg, jseg);
            intersections.append(&mut intersection);
        }

        segments_intersections.push((iseg_id, intersections));
    }

    let mut intersection_data = IntersectData::new();

    for (seg_id, mut intersections) in segments_intersections {
        intersections.push(1.);
        intersections.push(0.);
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let intersections = cleanup_intersections(intersections);

        // println!("{:?}", &intersections);

        // TODO: cleanup intersecitons

        let mut last_t = 0.;
        for &next_t in intersections.iter().skip(1) {
            let segment = segments_data[seg_id];
            let subsegment = segment.subsegment(last_t..next_t);

            intersection_data.push(subsegment, *seg_id);

            last_t = next_t;
        }
    }

    intersection_data
}

pub struct MergeData {
    num: usize,
}

pub fn merge(intersect_data: IntersectData) -> MergeData {
    todo!()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    StartToEnd,
    EndToStart,
}

impl Flow {
    fn start(&self) -> f64 {
        match *self {
            Flow::StartToEnd => 0.,
            Flow::EndToStart => 1.,
        }
    }

    fn end(&self) -> f64 {
        match *self {
            Flow::StartToEnd => 1.,
            Flow::EndToStart => 0.,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DynamicRegionStructure {
    parent: Vec<(SegmentId, Option<Part>)>,
    flow: Vec<Flow>,
}

impl DynamicRegionStructure {
    fn new() -> Self {
        Self {
            parent: Vec::new(),
            flow: Vec::new(),
        }
    }

    fn push(&mut self, parent: (SegmentId, Option<Part>), flow: Flow) {
        self.parent.push(parent);
        self.flow.push(flow);
    }

    fn match_structure(&self, other: &DynamicRegionStructure) -> bool {
        // Find if this cycle is a subset of other cycle
        let i_flows = self.flow.clone();
        let i_prnts = self.parent.clone();

        let mut j_flows = other.flow.clone();
        let mut j_prnts = other.parent.clone();

        let mut max_matches = 0;

        for i in 0..i_flows.len() {
            for j in 0..j_flows.len() {
                if i_flows[i] == j_flows[j] && i_prnts[i] == j_prnts[j] {
                    max_matches += 1;
                    // Remove the flows and parent of i's cyles which is not present in the j's cycle
                    j_flows.remove(j);
                    j_prnts.remove(j);
                    break;
                }
            }
        }

        // This cycle can be a subset of other cycle if atleast two of its edge is also edge of the other cycle
        if max_matches != self.flow.len() {
            return false;
        }

        // NOTE: A cycle can atmost have two edges with same flow and parent.
        // Check if the order matches

        let j_flows = other.flow.clone();
        let j_prnts = other.parent.clone();

        let mut i_flows = self.flow.clone();
        let mut i_prnts = self.parent.clone();
        for _ in 0..i_flows.len() {
            if i_flows == j_flows && i_prnts == j_prnts {
                return true;
            }
            i_flows.rotate_left(1);
            i_prnts.rotate_left(1);
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct DynamicData {
    paths: Vec<BezPath>,
    colors: Vec<Option<Color>>,
    structures: Vec<DynamicRegionStructure>,
}

impl DynamicData {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            colors: Vec::new(),
            structures: Vec::new(),
        }
    }

    fn push(&mut self, path: BezPath, structure: DynamicRegionStructure) {
        self.paths.push(path);
        self.structures.push(structure);
        self.colors.push(None);
    }

    pub fn render(&self) {
        // println!("paths {:?}", self.paths);
        // println!("RENDER \npaths len {}", self.paths.len());
        for (i, path) in self.paths.clone().iter().enumerate() {
            if path.elements().is_empty() {
                continue;
            }
            let Some(color) = self.colors[i] else {
                continue;
            };
            // println!("#");
            let mut bbox = path.bounding_box().abs();
            while bbox.y0 < bbox.y1 {
                let line = Line::new(Point::new(bbox.x0, bbox.y0), Point::new(bbox.x1, bbox.y0));
                let mut intersections = path.segments().fold(Vec::new(), |mut acc, seg| {
                    acc.append(
                        &mut seg
                            .intersect_line(line)
                            .iter()
                            .map(|it| line.eval(it.line_t).x)
                            .collect(),
                    );
                    acc
                });

                intersections.sort_by(|a, b| a.partial_cmp(&b).unwrap());

                for xs in intersections.chunks_exact(2) {
                    draw_line(
                        xs[0] as f32,
                        bbox.y0 as f32,
                        xs[1] as f32,
                        bbox.y0 as f32,
                        2.,
                        color,
                    );
                }
                bbox.y0 += 1.;
            }
        }
    }

    pub fn build(intersect_data: IntersectData) -> DynamicData {
        // println!("{}", intersect_data);
        let mut dynamic_regions = DynamicData::new();

        let mut visited_start_to_end = HashSet::<usize>::new();
        let mut visited_end_to_start = HashSet::<usize>::new();

        for next_idx in 0..intersect_data.segments.len() {
            for flow in [Flow::StartToEnd, Flow::EndToStart] {
                let visited = match flow {
                    Flow::StartToEnd => visited_start_to_end.contains(&next_idx),
                    Flow::EndToStart => visited_end_to_start.contains(&next_idx),
                };

                if visited {
                    continue;
                }

                if let Some((bezpath, structure, visited_idxs)) =
                    Self::build_region(&intersect_data, next_idx, flow)
                {
                    // println!("{:#?}", structure); // NOTE: DEBUG:
                    // update visited state
                    for i in 0..visited_idxs.len() {
                        let flow = structure.flow[i];
                        let idx = visited_idxs[i];
                        match flow {
                            Flow::StartToEnd => visited_start_to_end.insert(idx),
                            Flow::EndToStart => visited_end_to_start.insert(idx),
                        };
                    }

                    dynamic_regions.push(bezpath, structure);
                }
            }
        }

        dynamic_regions
    }

    fn build_region(
        intersect_data: &IntersectData,
        start: usize,
        flow: Flow,
    ) -> Option<(BezPath, DynamicRegionStructure, Vec<usize>)> {
        let mut path = BezPath::new();
        let mut structure = DynamicRegionStructure::new();
        let mut visited_index = Vec::new();

        let mut curr_idx = start;
        let mut curr_flow = flow;

        // println!(
        //     "====================================================> {}, {:?}",
        //     curr_idx, curr_flow
        // );
        // let mut i = 0;
        loop {
            let curr_segment = intersect_data.segments[curr_idx];
            let curr_segment = match curr_flow {
                Flow::StartToEnd => curr_segment,
                Flow::EndToStart => curr_segment.reverse(),
            };
            let curr_parent = intersect_data.parents[curr_idx];
            if path.elements().is_empty() {
                path.move_to(curr_segment.start());
            }
            path.push(curr_segment.as_path_el());
            structure.push(curr_parent, curr_flow);
            visited_index.push(curr_idx);

            let curr_tangent = {
                let a = curr_segment.eval(1.);
                let b = curr_segment.eval(0.98);
                (b - a).normalize()
            };
            let curr_angle = curr_tangent.angle();
            let curr_angle = if curr_angle.is_sign_negative() {
                2. * PI + curr_angle
            } else {
                curr_angle
            };

            // println!(
            //     "{}, |__ {:.3} {:?}, {:?}",
            //     curr_idx,
            //     curr_angle.to_degrees(),
            //     curr_flow,
            //     curr_segment.end()
            // );

            let mut min_angle = 8.;
            let mut best_next = None;

            for next_idx in 0..intersect_data.segments.len() {
                if next_idx == curr_idx {
                    continue;
                }
                let next_segment = intersect_data.segments[next_idx];

                let next_flow =
                    if curr_segment.end().distance(next_segment.start()) < MIN_SEPARATION {
                        Flow::StartToEnd
                    } else if curr_segment.end().distance(next_segment.end()) < MIN_SEPARATION {
                        Flow::EndToStart
                    } else {
                        continue;
                    };

                let next_segment = match next_flow {
                    Flow::StartToEnd => next_segment,
                    Flow::EndToStart => next_segment.reverse(),
                };

                let next_tangent = {
                    let a = next_segment.eval(0.);
                    let b = next_segment.eval(0.02);
                    (b - a).normalize()
                };
                let ctn = curr_tangent.normalize();
                let ntn = next_tangent.normalize();
                // println!(
                //     "\t\t curr_tangent ({:.3}, {:.3}), next_tangent ({:.3}, {:.3})",
                //     ctn.x, ctn.y, ntn.x, ntn.y
                // );
                let angle = DVec2::new(curr_tangent.x, curr_tangent.y)
                    .angle_between(DVec2::new(next_tangent.x, next_tangent.y));
                let angle = if angle.is_sign_negative() {
                    2. * PI + angle
                } else {
                    angle
                };
                // println!(
                //     "\t\tcnext {}, |__ {:.2}, {:?}, {:?}",
                //     next_idx,
                //     angle.to_degrees(),
                //     next_segment.start(),
                //     next_segment.end()
                // );
                if angle < min_angle {
                    min_angle = angle;
                    best_next = Some((next_idx, next_flow));
                    // println!(
                    //     "\tpnext {}, {:?}, {:?}",
                    //     next_idx,
                    //     next_flow,
                    //     next_segment.start()
                    // );
                }
            }

            if let Some((next_idx, next_flow)) = best_next {
                curr_idx = next_idx;
                curr_flow = next_flow;
            } else {
                // this should not happen because a segment is connected to it by itself. it will just be different flow.
                curr_flow = match curr_flow {
                    Flow::StartToEnd => Flow::EndToStart,
                    Flow::EndToStart => Flow::StartToEnd,
                };
            }

            if curr_idx == start && curr_flow == flow {
                break;
            }
            // if i > 10 {
            //     break;
            // }
            // i += 1;
        }

        // println!(
        //     "====================================================> {}, {:?}",
        //     curr_idx, curr_flow
        // );

        if structure.flow.len() > 1 && curr_idx == start && curr_flow == flow {
            Some((path, structure, visited_index))
        } else {
            None
        }
    }

    pub fn filter_outer_regions(self) -> Self {
        let mut paths = Vec::new();
        let mut colors = Vec::new();
        let mut structures = Vec::new();

        let bboxes = self
            .paths
            .iter()
            .map(|path| path.bounding_box())
            .collect::<Vec<Rect>>();

        let max_bbox = bboxes
            .iter()
            .max_by(|a, b| a.area().abs().partial_cmp(&b.area().abs()).unwrap());

        // There is atleast two closed path in a mesh when one is the outer and other is the inner closed paths/faces/cycles
        if self.paths.len() > 1 {
            for i in 0..self.paths.len() {
                let mxbbox = max_bbox.unwrap();
                if mxbbox.area() > bboxes[i].area() {
                    paths.push(self.paths[i].clone());
                    colors.push(self.colors[i].clone());
                    structures.push(self.structures[i].clone());
                }
            }
        }

        DynamicData {
            paths,
            colors,
            structures,
        }
    }

    pub fn dynamic(mut self, prev_dynamic_region: DynamicData) -> Self {
        for i in 0..self.paths.len() {
            let curr_structure = self.structures[i].clone();
            let structure_match = prev_dynamic_region
                .structures
                .iter()
                .enumerate()
                .find(|&(_idx, prev_strut)| curr_structure.match_structure(prev_strut));

            if let Some((idx, _structure)) = structure_match {
                self.colors[i] = prev_dynamic_region.colors[idx];
            }
        }
        self
    }

    pub fn apply_style(&mut self, color: Option<Color>, position: Point) {
        for i in 0..self.paths.len() {
            let mut path = self.paths[i].clone();
            path.close_path();
            if path.contains(position) {
                self.colors[i] = color;
            }
        }
    }
}
