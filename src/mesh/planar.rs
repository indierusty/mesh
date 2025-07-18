use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use kurbo::{BezPath, ParamCurve, PathSeg, Point, Shape};
use macroquad::{
    color::{BLANK, BLUE, BROWN, Color, GREEN, RED, YELLOW},
    shapes::draw_rectangle,
};

use crate::{
    algo::path_intersections,
    mesh::MMesh,
    util::{point_to_gvec2, points_to_segment},
};

use super::{PointData, PointId, SegmentData, SegmentId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    StartToEnd,
    EndToStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum XColor {
    Red,
    Green,
    Yellow,
    Blue,
    Brown,
    Gray,
    Blank,
}

impl XColor {
    fn to_color(&self) -> Color {
        match self {
            XColor::Red => RED,
            XColor::Green => GREEN,
            XColor::Yellow => YELLOW,
            XColor::Blue => BLUE,
            XColor::Brown => BROWN,
            XColor::Gray => GREEN,
            XColor::Blank => BLANK,
        }
    }
}

pub fn draw_region(
    regions: &Vec<Vec<(SegmentData, Direction)>>,
    points: &HashMap<PointId, PointData>,
) {
    let colors = [BLUE, GREEN, YELLOW, BROWN];
    let mut ci = 0;
    for (_, region) in regions.iter().enumerate() {
        let mut bez = BezPath::from_path_segments(
            region
                .iter()
                .map(|(data, direction)| segment_data_to_pathseg(&points, *data, *direction)),
        );
        bez.close_path();

        let bbox = bez.bounding_box();

        for x in (0..bbox.width() as usize).step_by(10) {
            for y in (0..bbox.height() as usize).step_by(10) {
                let x = x as f64 + bbox.x0;
                let y = y as f64 + bbox.y0;
                let point = Point::new(x as f64, y as f64);
                if bez.contains(point) {
                    draw_rectangle(x as f32, y as f32, 3., 3., colors[ci % colors.len()]);
                }
            }
        }
        ci += 1;
    }
}

pub fn segment_data_to_pathseg(
    points_map: &HashMap<PointId, PointData>,
    data: SegmentData,
    direction: Direction,
) -> PathSeg {
    let (p1, p2, p3, p4) = match direction {
        Direction::StartToEnd => (data.p1, data.p2, data.p3, data.p4),
        Direction::EndToStart => (data.p4, data.p3, data.p2, data.p1),
    };
    let p1 = points_map.get(&p1).unwrap().position;
    let p2 = p2.and_then(|p2| points_map.get(&p2)).map(|p| p.position);
    let p3 = p3.and_then(|p3| points_map.get(&p3)).map(|p| p.position);
    let p4 = points_map.get(&p4).unwrap().position;
    points_to_segment(p1, p2, p3, p4)
}

fn cleanup_intersections(intersections: &[f64]) -> Vec<f64> {
    let mut intersections = intersections
        .iter()
        .filter(|t| **t > 0. || **t <= 1.)
        .map(|t| *t)
        .collect::<Vec<f64>>();

    intersections.push(1.);
    intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mut res2 = Vec::new();
    if !intersections.is_empty() {
        let mut last_t = intersections[0];
        res2.push(last_t);
        for i in 1..intersections.len() {
            if intersections[i] - last_t > 0.01 {
                res2.push(intersections[i]);
                last_t = intersections[i];
            }
        }
    }

    res2
}

impl MMesh {
    pub fn planar_graph(&self) -> (MMesh, HashMap<SegmentId, SegmentId>) {
        let points_map = self.points_map();
        let segments_data = self.segments.data();

        let mut result = MMesh::empty();
        let mut parents: HashMap<SegmentId, SegmentId> = HashMap::new();

        for i in 0..segments_data.len() {
            let iseg_data = segments_data[i];
            let iseg = iseg_data.to_path_seg(&points_map);
            let mut intersections = Vec::new();
            for j in 0..segments_data.len() {
                if i == j {
                    continue;
                }
                let jseg = segments_data[j].to_path_seg(&points_map);
                intersections.append(&mut path_intersections(iseg, jseg));
            }

            let intersections = cleanup_intersections(&intersections);

            let mut p1 = (0., result.append_point(iseg.start()));

            for t in intersections {
                let sub_seg = iseg.subsegment(p1.0..t);
                let (p2, p3) = match sub_seg {
                    kurbo::PathSeg::Line(_) => (None, None),
                    kurbo::PathSeg::Quad(quad) => (Some(result.append_point(quad.p1)), None),
                    kurbo::PathSeg::Cubic(cubic) => (
                        Some(result.append_point(cubic.p1)),
                        Some(result.append_point(cubic.p2)),
                    ),
                };
                let p4 = result.append_point(sub_seg.end());

                // Append the segment and its parent segment in the original mesh
                let id = result.append_segment(p1.1, p2, p3, p4).unwrap();
                parents.insert(id, iseg_data.id);

                p1 = (t, p4);
            }
        }

        ////////////////////////////////////////////////////////////////////////////////
        // Find anchor points which are very close to each other.
        let points_data = result.points.data();
        let segment_data = result.segments.data();

        let mut main_points: Vec<PointData> = Vec::new();
        let mut merge_points = HashMap::new();
        let handle_points = segment_data.iter().fold(HashSet::new(), |mut set, data| {
            if let Some(p2) = data.p2 {
                set.insert(p2);
            };
            if let Some(p3) = data.p3 {
                set.insert(p3);
            };
            set
        });

        for i in 0..points_data.len() {
            let ipoint = points_data[i];
            // If this point is a handle of a segment then we skip it.
            // (NOTE: Might not need this as we won't modify the planar graph)
            if handle_points.contains(&ipoint.id) {
                continue;
            }
            // There is already a main points for this point to get merge with so we skip it.
            if merge_points.get(&ipoint.id).is_some() {
                continue;
            }
            // If there is no points close to this point which is in main points then we make this a main point.
            let main_point = main_points
                .iter()
                .find(|main_point| main_point.position.distance(ipoint.position) < 3.);

            if let Some(main_point) = main_point {
                merge_points.insert(ipoint.id, main_point.id);
            } else {
                main_points.push(ipoint);
            }
        }

        for segdata in &segment_data {
            if let Some(main_point) = merge_points.get(&segdata.p1) {
                result.segments.p1[segdata.idx.idx()] = *main_point;
            }
            if let Some(main_point) = merge_points.get(&segdata.p4) {
                result.segments.p4[segdata.idx.idx()] = *main_point;
            }
        }

        // Delete the points in the points table which are merged to the main points. (Might not need to do this.)
        result.points.remove_multiple(&merge_points.iter().fold(
            HashSet::new(),
            |mut set, (k, _)| {
                set.insert(*k);
                set
            },
        ));
        ////////////////////////////////////////////////////////////////////////////////

        (result, parents)
    }

    pub fn calculate_regions(
        &self,
    ) -> (
        Vec<Vec<(SegmentData, Direction)>>,
        HashMap<PointId, PointData>,
    ) {
        let points_map = self.points_map();
        let segments_data = self.segments.data();

        let mut visited_start_to_end: HashSet<SegmentId> = HashSet::new();
        let mut visited_end_to_start: HashSet<SegmentId> = HashSet::new();

        let mut regions = Vec::new();

        for segment in &segments_data {
            if !visited_start_to_end.contains(&segment.id) {
                let mut region = Vec::new();

                let mut prev_seg = *segment;
                let mut prev_seg_direction = Direction::StartToEnd;

                'a: loop {
                    let mut closest_next_seg = None;

                    // Iterate thourgh all the segment which are connect to prev_seg and find the segment which has closest angle between in anticlock direction.
                    for next_seg in &segments_data {
                        let connected = match prev_seg_direction {
                            Direction::StartToEnd => {
                                prev_seg.p4 == next_seg.p1 || prev_seg.p4 == next_seg.p4
                            }
                            Direction::EndToStart => {
                                prev_seg.p1 == next_seg.p1 || prev_seg.p1 == next_seg.p4
                            }
                        };

                        let same_segment = prev_seg.id == next_seg.id;
                        if same_segment || !connected {
                            continue;
                        }

                        let prev_pseg =
                            segment_data_to_pathseg(&points_map, prev_seg, prev_seg_direction);

                        let next_seg = next_seg.clone();
                        let next_seg_direction;

                        let next_pseg = if next_seg.p1
                            == match prev_seg_direction {
                                Direction::StartToEnd => prev_seg.p4,
                                Direction::EndToStart => prev_seg.p1,
                            } {
                            // Measure the angle between endpoints in anticlockwise direction.
                            next_seg_direction = Direction::StartToEnd;
                            segment_data_to_pathseg(&points_map, next_seg, next_seg_direction)
                        } else {
                            next_seg_direction = Direction::EndToStart;
                            segment_data_to_pathseg(&points_map, next_seg, next_seg_direction)
                        };

                        let curr_start = prev_pseg.eval(1.);
                        let curr_end = prev_pseg.eval(0.99);
                        let curr_dir = point_to_gvec2(curr_end) - point_to_gvec2(curr_start);

                        let next_start = next_pseg.eval(0.);
                        let next_end = next_pseg.eval(0.01);
                        let next_dir = point_to_gvec2(next_end) - point_to_gvec2(next_start);

                        let angle = curr_dir.angle_to(next_dir);
                        let angle = if angle.is_sign_negative() {
                            PI + (PI + angle)
                        } else {
                            angle
                        };

                        closest_next_seg = closest_next_seg
                            .and_then(|(closest_seg, closest_angle, closest_direction)| {
                                if angle < closest_angle {
                                    Some((next_seg, angle, next_seg_direction))
                                } else {
                                    Some((closest_seg, closest_angle, closest_direction))
                                }
                            })
                            .or(Some((next_seg, angle, next_seg_direction)));
                    }

                    println!(
                        "closest_next_seg {:?}, curr seg {:?}",
                        closest_next_seg, segment.id
                    );

                    match closest_next_seg {
                        Some((next_seg, _angle, next_seg_direction)) => {
                            match next_seg_direction {
                                Direction::StartToEnd => visited_start_to_end.insert(next_seg.id),
                                Direction::EndToStart => visited_end_to_start.insert(next_seg.id),
                            };

                            region.push((next_seg, next_seg_direction));

                            if next_seg.id == segment.id {
                                regions.push(region);

                                // Reached to the beginning of the region hence we close the region and break loop
                                break 'a;
                            } else {
                                prev_seg = next_seg;
                                prev_seg_direction = next_seg_direction;
                            }
                        }
                        None => {
                            // This is an open path so no region will be formed.
                            break 'a;
                        }
                    }
                }
            }
            if !visited_end_to_start.contains(&segment.id) {
                let mut prev_seg = *segment;
                let mut prev_seg_direction = Direction::EndToStart;

                let mut region = Vec::new();

                'a: loop {
                    let mut closest_next_seg = None;

                    // Iterate thourgh all the segment which are connect to next_curr_seg and find the segment which has closest angle between in anticlock direction.
                    for next_seg in &segments_data {
                        let connected = match prev_seg_direction {
                            Direction::StartToEnd => {
                                prev_seg.p4 == next_seg.p1 || prev_seg.p4 == next_seg.p4
                            }
                            Direction::EndToStart => {
                                prev_seg.p1 == next_seg.p1 || prev_seg.p1 == next_seg.p4
                            }
                        };
                        let same_segment = prev_seg.id == next_seg.id;
                        if same_segment || !connected {
                            continue;
                        }
                        let prev_pseg =
                            segment_data_to_pathseg(&points_map, prev_seg, prev_seg_direction);

                        let next_seg = next_seg.clone();
                        let next_seg_direction;

                        let next_pseg = if next_seg.p1
                            == match prev_seg_direction {
                                Direction::StartToEnd => prev_seg.p4,
                                Direction::EndToStart => prev_seg.p1,
                            } {
                            // Measure the angle between endpoints in anticlockwise direction.
                            next_seg_direction = Direction::StartToEnd;
                            segment_data_to_pathseg(&points_map, next_seg, next_seg_direction)
                        } else {
                            next_seg_direction = Direction::EndToStart;
                            segment_data_to_pathseg(&points_map, next_seg, next_seg_direction)
                        };

                        // TODO: Find a better way to do this
                        let curr_start = prev_pseg.eval(1.);
                        let curr_end = prev_pseg.eval(0.99);
                        let curr_dir = point_to_gvec2(curr_end) - point_to_gvec2(curr_start);

                        let next_start = next_pseg.eval(0.);
                        let next_end = next_pseg.eval(0.01);
                        let next_dir = point_to_gvec2(next_end) - point_to_gvec2(next_start);

                        let angle = curr_dir.angle_to(next_dir);
                        let angle = if angle.is_sign_negative() {
                            2. * PI + angle
                        } else {
                            angle
                        };

                        closest_next_seg = closest_next_seg
                            .and_then(|(closest_seg, closest_angle, closest_direction)| {
                                if angle < closest_angle {
                                    Some((next_seg, angle, next_seg_direction))
                                } else {
                                    Some((closest_seg, closest_angle, closest_direction))
                                }
                            })
                            .or(Some((next_seg, angle, next_seg_direction)));
                    }

                    match closest_next_seg {
                        Some((next_seg, _angle, next_seg_direction)) => {
                            match next_seg_direction {
                                Direction::StartToEnd => visited_start_to_end.insert(next_seg.id),
                                Direction::EndToStart => visited_end_to_start.insert(next_seg.id),
                            };

                            println!("Pushed a next_seg ID: {:?}", next_seg.id);
                            region.push((next_seg, next_seg_direction));

                            if next_seg.id == segment.id {
                                regions.push(region);
                                // Reached to the beginning of the region hence we close the region and break loop
                                break 'a;
                            } else {
                                prev_seg = next_seg;
                                prev_seg_direction = next_seg_direction
                            }
                        }
                        None => {
                            // This is an open path so no region will be formed.
                            break 'a;
                        }
                    }
                }
            }
        }

        (regions, points_map)
    }
}

pub struct RegionStyle {
    parent_and_direction: Vec<(SegmentId, Direction)>,
    color: XColor,
}

impl RegionStyle {
    fn new(parent_and_direction: Vec<(SegmentId, Direction)>, color: XColor) -> Self {
        Self {
            parent_and_direction,
            color,
        }
    }

    pub fn match_style(&self, parent_and_direction: &Vec<(SegmentId, Direction)>) -> usize {
        let mut matches = 0;

        for (parent, directon) in parent_and_direction {
            for (sparent, sdirection) in &self.parent_and_direction {
                if *parent == *sparent && *directon == *sdirection {
                    matches += 1;
                    break;
                }
            }
        }

        matches
    }
}

pub fn calculate_and_draw_style(
    regions: &Vec<Vec<(SegmentData, Direction)>>,
    parents: HashMap<SegmentId, SegmentId>,
    points: &HashMap<PointId, PointData>,
    styles: Vec<RegionStyle>,
    setcolor: Option<(Point, XColor)>,
) -> Vec<RegionStyle> {
    let mut new_styles = Vec::new();

    for (_, region) in regions.iter().enumerate() {
        let mut bez = BezPath::from_path_segments(
            region
                .iter()
                .map(|(data, direction)| segment_data_to_pathseg(&points, *data, *direction)),
        );
        bez.close_path();

        let parent_and_direction = region
            .iter()
            .map(|(seg, dir)| (*parents.get(&seg.id).unwrap(), *dir))
            .collect::<Vec<(SegmentId, Direction)>>();

        let style = styles.iter().max_by(|&a, &b| {
            a.match_style(&parent_and_direction)
                .cmp(&b.match_style(&parent_and_direction))
        });

        let mut color = style.map(|s| s.color).unwrap_or(XColor::Blank);

        // set color
        if let Some((setpoint, setcolor)) = setcolor {
            if bez.contains(setpoint) {
                color = setcolor;
            }
        }
        // Reassign the styles
        new_styles.push(RegionStyle::new(parent_and_direction, color));

        let bbox = bez.bounding_box();

        for x in (0..bbox.width() as usize).step_by(10) {
            for y in (0..bbox.height() as usize).step_by(10) {
                let x = x as f64 + bbox.x0;
                let y = y as f64 + bbox.y0;
                let point = Point::new(x as f64, y as f64);
                if bez.contains(point) {
                    draw_rectangle(x as f32, y as f32, 3., 3., color.to_color());
                }
            }
        }
    }

    new_styles
}
