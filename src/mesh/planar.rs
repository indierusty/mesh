use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
};

use kurbo::ParamCurve;

use crate::{algo::path_intersections, mesh::MMesh};

use super::{PointData, PointId};

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
    pub fn planar_graph(&self) -> MMesh {
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

        // TODO: Delete the points in the points table which are merged to the main points. (Might not need to do this.)
        result.points.remove_multiple(&merge_points.iter().fold(
            HashSet::new(),
            |mut set, (k, _)| {
                set.insert(*k);
                set
            },
        ));
        ////////////////////////////////////////////////////////////////////////////////

        result
    }

    pub fn prev_planar_graph(&self) -> MMesh {
        print!("Planar!!\n");
        let points = self.points_map();
        let segments_data = self.segments.data();
        println!("segment data {:#?}", segments_data);

        let mut new_mesh = self.clone();

        // Collect all the intersections for each segments in the mesh.
        for i in 0..segments_data.len() {
            let seg1_data = segments_data[i];
            let seg1 = self.segment(seg1_data.idx, &points);
            let mut intersections = Vec::new();

            for j in 0..segments_data.len() {
                if i == j {
                    continue;
                }
                let seg2_data = segments_data[j];
                let seg2 = self.segment(seg2_data.idx, &points);
                intersections.append(&mut path_intersections(seg1, seg2));
            }

            let mut intersections = intersections
                .iter()
                .filter(|t| **t > 0.01 && **t < 0.09)
                .map(|t| *t)
                .collect::<Vec<f64>>();

            intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
            intersections.push(1.);

            println!("Intersection {:?}", intersections);

            let mut last_point: Option<(PointId, f64)> = None;

            for t in intersections {
                let p1 = last_point.unwrap_or((seg1_data.p1, 0.));
                let subseg = seg1.subsegment(p1.1..t);

                let p4 = if t == 1. {
                    seg1_data.p4
                } else if let Some(closest_point) = new_mesh.closest_point(subseg.end(), Some(1.)) {
                    closest_point.0
                } else {
                    new_mesh.append_point(subseg.end())
                };

                println!("p1 {:?}, p4 {:?}", p1, (p4, t));

                let (p2, p3) = match subseg {
                    kurbo::PathSeg::Line(_) => (None, None),
                    kurbo::PathSeg::Quad(quad) => (Some(new_mesh.append_point(quad.p1)), None),
                    kurbo::PathSeg::Cubic(cubic) => (
                        Some(new_mesh.append_point(cubic.p1)),
                        Some(new_mesh.append_point(cubic.p2)),
                    ),
                };

                new_mesh.append_segment(p1.0, p2, p3, p4);

                last_point = Some((p4, t));
            }
            // TODO: delete original segment
            new_mesh.remove_segment(seg1_data.id);
        }

        println!("new segment data {:#?}", new_mesh.segments.data());
        new_mesh
    }
}
