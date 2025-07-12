use kurbo::{DEFAULT_ACCURACY, ParamCurve, PathSeg, Shape};

fn intersections(
    seg1: PathSeg,
    min1: f64,
    max1: f64,
    seg2: PathSeg,
    min2: f64,
    max2: f64,
    result: &mut Vec<f64>,
) {
    let bbox1 = seg1.subsegment(min1..max1).bounding_box();
    let bbox2 = seg2.subsegment(min2..max2).bounding_box();

    if bbox1.overlaps(bbox2) {
        if bbox1.width() < DEFAULT_ACCURACY && bbox1.height() < DEFAULT_ACCURACY {
            result.push(min1);
            return;
        }
        let mid1 = (min1 + max1) / 2.;
        let mid2 = (min2 + max2) / 2.;
        intersections(seg1, min1, mid1, seg2, min2, mid2, result);
        intersections(seg1, min1, mid1, seg2, mid2, max2, result);
        intersections(seg1, mid1, max1, seg2, min2, mid2, result);
        intersections(seg1, mid1, max1, seg2, mid2, max2, result);
    }
}

// TODO: Cleanup and find a better way to do this
pub fn path_intersections(seg1: PathSeg, seg2: PathSeg) -> Vec<f64> {
    let mut result = Vec::new();
    intersections(seg1, 0., 1., seg2, 0., 1., &mut result);
    result
}
