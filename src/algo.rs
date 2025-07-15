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
        let mid1 = (min1 + max1) / 2.;
        let mid2 = (min2 + max2) / 2.;

        if bbox1.width() < DEFAULT_ACCURACY && bbox1.height() < DEFAULT_ACCURACY {
            // Skip the intersection at the endpoints which are connected or are at the same position.
            if (seg1.start().distance(seg2.start()) < 1. && mid1 < 0.01 && mid2 < 0.01)
                || (seg1.start().distance(seg2.end()) < 1. && mid1 < 0.01 && mid2 > 0.99)
                || (seg1.end().distance(seg2.start()) < 1. && mid1 > 0.99 && mid2 < 0.01)
                || (seg1.end().distance(seg2.end()) < 1. && mid1 > 0.99 && mid2 > 0.99)
            {
                return;
            }
            // let close_endpoints = seg1.start().distance(seg2.end()) < 0.05
            //     || seg1.start().distance(seg2.start()) < 0.05
            //     || seg1.end().distance(seg2.end()) < 0.05
            //     || seg1.end().distance(seg2.start()) < 0.05;

            // let intersect_at_endpoints = mid1 < 0.05 || mid1 > 0.95 || mid2 < 0.05 || mid2 > 0.95;

            // TODO: find a better solution to this.
            // if intersection is near the endpoints or either segments and their endpoints are closer then we skip the intersection.
            // if !(close_endpoints && intersect_at_endpoints) {
            //     result.puhs(min1);
            // }
            result.push(min1);
            return;
        }
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
