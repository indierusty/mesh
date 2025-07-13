use std::{
    collections::{HashMap, HashSet},
    f32::consts::PI,
};

use kurbo::{BezPath, ParamCurve, PathSeg, Point, Shape};
use macroquad::{
    color::{BLUE, BROWN, GREEN, YELLOW},
    shapes::draw_rectangle,
};

use crate::{
    algo::path_intersections,
    mesh::MMesh,
    util::{point_to_gvec2, points_to_segment},
};

use super::{PointData, PointId, SegmentData, SegmentId};

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    StartToEnd,
    EndToStart,
}

pub fn draw_region((regions, points_map): &(Vec<Vec<SegmentData>>, HashMap<PointId, PointData>)) {
    let colors = [BLUE, GREEN, YELLOW, BROWN];
    let mut ci = 0;
    for (_, region) in regions.iter().enumerate() {
        let bez = BezPath::from_path_segments(region.iter().map(|d| {
            match d.direction.unwrap_or(Direction::StartToEnd) {
                Direction::StartToEnd => points_id_to_segment(&points_map, d.p1, d.p2, d.p3, d.p4),
                Direction::EndToStart => points_id_to_segment(&points_map, d.p4, d.p3, d.p2, d.p1),
            }
        }));

        let bbox = bez.bounding_box();

        for x in (0..bbox.width() as usize).skip(10) {
            for y in (0..bbox.height() as usize).skip(10) {
                let x = x as f64 + bbox.x0;
                let y = y as f64 + bbox.y0;
                let point = Point::new(x as f64, y as f64);
                if bez.contains(point) {
                    draw_rectangle(x as f32, y as f32, 3., 3., colors[ci % colors.iter().len()]);
                }
            }
        }
        ci += 1;
    }
}

pub fn points_id_to_segment(
    points_map: &HashMap<PointId, PointData>,
    p1: PointId,
    p2: Option<PointId>,
    p3: Option<PointId>,
    p4: PointId,
) -> PathSeg {
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
    pub fn planar_graph(&self) -> (MMesh, (Vec<Vec<SegmentData>>, HashMap<PointId, PointData>)) {
        let points_map = self.points_map();
        let segments_data = self.segments.data();

        let mut result = MMesh::empty();

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

                result.append_segment(p1.1, p2, p3, p4);
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

        // let segments_from_start = segments_data.iter().fold(HashMap::new(), |mut acc, data| {
        //     acc.insert(data.p1, *data);
        //     acc
        // });
        // let segments_from_end = segments_data.iter().fold(HashMap::new(), |mut acc, data| {
        //     acc.insert(data.p4, *data);
        //     acc
        // });

        let rpoints_map = result.points_map();
        let rsegments_data = result.segments.data();

        let mut visited_start_to_end: HashSet<SegmentId> = HashSet::new();
        let mut visited_end_to_start: HashSet<SegmentId> = HashSet::new();

        let mut regions = Vec::new();

        for curr_seg in &rsegments_data {
            if !visited_start_to_end.contains(&curr_seg.id) {
                let mut region = Vec::new();
                let mut next_curr_seg = *curr_seg;
                'a: loop {
                    let mut closest_next_seg = None;

                    // Iterate thourgh all the segment which are connect to next_curr_seg and find the segment which has closest angle between in anticlock direction.
                    for next_seg in &rsegments_data {
                        let connected =
                            next_curr_seg.p4 == next_seg.p1 || next_curr_seg.p4 == next_seg.p4;
                        let same_segment = next_curr_seg.id == next_seg.id;
                        if same_segment || !connected {
                            continue;
                        }
                        let mut next_seg = next_seg.clone();
                        let curr_pseg = points_id_to_segment(
                            &rpoints_map,
                            next_curr_seg.p1,
                            next_curr_seg.p2,
                            next_curr_seg.p3,
                            next_curr_seg.p4,
                        );
                        let next_pseg = if next_seg.p1 == next_curr_seg.p4 {
                            // Measure the angle between endpoints in anticlockwise direction.
                            next_seg.direction = Some(Direction::StartToEnd);
                            points_id_to_segment(
                                &rpoints_map,
                                next_seg.p1,
                                next_seg.p2,
                                next_seg.p3,
                                next_seg.p4,
                            )
                        } else {
                            next_seg.direction = Some(Direction::EndToStart);
                            points_id_to_segment(
                                &rpoints_map,
                                next_seg.p4,
                                next_seg.p3,
                                next_seg.p2,
                                next_seg.p1,
                            )
                        };

                        let curr_start = curr_pseg.eval(1.);
                        let curr_end = curr_pseg.eval(0.99);
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
                            .and_then(|(prev_seg, prev_angle)| {
                                if angle < prev_angle {
                                    Some((next_seg, angle))
                                } else {
                                    Some((prev_seg, prev_angle))
                                }
                            })
                            .or(Some((next_seg, angle)));
                    }

                    println!("closest_next_seg {:?}", closest_next_seg);

                    match closest_next_seg {
                        Some((next_seg, _angle)) => {
                            match next_seg.direction.unwrap() {
                                Direction::StartToEnd => visited_start_to_end.insert(next_seg.id),
                                Direction::EndToStart => visited_end_to_start.insert(next_seg.id),
                            };

                            region.push(next_seg);

                            if next_seg.id == curr_seg.id {
                                regions.push(region);

                                // Reached to the beginning of the region hence we close the region and break loop
                                break 'a;
                            } else {
                                next_curr_seg = next_seg;
                            }
                        }
                        None => {
                            // This is an open path so no region will be formed.
                            break 'a;
                        }
                    }
                }
            }
            if !visited_end_to_start.contains(&curr_seg.id) {
                let mut next_curr_seg = *curr_seg;
                let mut region = Vec::new();
                'a: loop {
                    let mut closest_next_seg = None;
                    // Iterate thourgh all the segment which are connect to next_curr_seg and find the segment which has closest angle between in anticlock direction.
                    for next_seg in &rsegments_data {
                        let connected =
                            next_curr_seg.p1 == next_seg.p1 || next_curr_seg.p1 == next_seg.p4;
                        let same_segment = next_curr_seg.id == next_seg.id;
                        if same_segment || !connected {
                            continue;
                        }
                        let mut next_seg = next_seg.clone();
                        let curr_pseg = points_id_to_segment(
                            &rpoints_map,
                            next_curr_seg.p4,
                            next_curr_seg.p3,
                            next_curr_seg.p2,
                            next_curr_seg.p1,
                        );
                        let next_pseg = if next_seg.p1 == next_curr_seg.p1 {
                            // Measure the angle between endpoints in anticlockwise direction.
                            next_seg.direction = Some(Direction::StartToEnd);
                            points_id_to_segment(
                                &rpoints_map,
                                next_seg.p1,
                                next_seg.p2,
                                next_seg.p3,
                                next_seg.p4,
                            )
                        } else {
                            next_seg.direction = Some(Direction::EndToStart);
                            points_id_to_segment(
                                &rpoints_map,
                                next_seg.p4,
                                next_seg.p3,
                                next_seg.p2,
                                next_seg.p1,
                            )
                        };

                        let curr_start = curr_pseg.eval(0.99);
                        let curr_end = curr_pseg.eval(1.);
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
                            .and_then(|(prev_seg, prev_angle)| {
                                if angle < prev_angle {
                                    Some((next_seg, angle))
                                } else {
                                    Some((prev_seg, prev_angle))
                                }
                            })
                            .or(Some((next_seg, angle)));
                    }

                    match closest_next_seg {
                        Some((next_seg, _angle)) => {
                            match next_seg.direction.unwrap() {
                                Direction::StartToEnd => visited_start_to_end.insert(next_seg.id),
                                Direction::EndToStart => visited_end_to_start.insert(next_seg.id),
                            };

                            println!("Pushed a next_seg ID: {:?}", next_seg.id);
                            region.push(next_seg);

                            if next_seg.id == curr_seg.id {
                                regions.push(region);
                                // Reached to the beginning of the region hence we close the region and break loop
                                break 'a;
                            } else {
                                next_curr_seg = next_seg;
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

        (result, (regions, rpoints_map))
    }
}
