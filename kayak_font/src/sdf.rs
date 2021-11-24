use lyon_geom::math::Vector;

use crate::{contour::Contour, utils::EdgeDistance};

/// Computes a SDF from a list of contours. The returned vectors are a `dim` by `dim`
/// matrix of signed distances. The output represents the signed distances to the contours
/// within [0, 1] x [0, 1]
pub fn compute_sdf(contours: &[Contour], dim: usize) -> Vec<Vec<f32>> {
    let scale: f32 = 1.0 / (dim as f32);
    let windings: Vec<i32> = contours.iter().map(|c| c.winding() as i32).collect();

    (0..dim)
        .map(|y| {
            let py = (y as f32 + 0.5) * scale;
            (0..dim)
                .map(|x| {
                    if contours.len() == 0 {
                        return 1.0f32;
                    }

                    let px = (x as f32 + 0.5) * scale;
                    let p = Vector::new(px, py);

                    let mut neg_dist = 1e24f32;
                    let mut pos_dist = -1e24f32;
                    let mut winding = 0;
                    let mut contour_distances = Vec::new();
                    contour_distances.reserve(contours.len());

                    for (i, contour) in contours.iter().enumerate() {
                        let mut contour_min = EdgeDistance::new();

                        for elem in contour.elements.iter() {
                            let (d, na) = elem.distance(p.to_point());

                            if d < contour_min.dist {
                                contour_min.dist = d;
                                contour_min.edge = Some(&elem);
                                contour_min.nearest_approach = na;
                            }
                        }

                        // contour_min.to_pseudodistance(p);
                        let cmdd = contour_min.dist.distance;

                        contour_distances.push(cmdd);

                        if windings[i] > 0 && cmdd >= 0.0 && cmdd.abs() < pos_dist.abs() {
                            pos_dist = cmdd;
                        }
                        if windings[i] < 0 && cmdd <= 0.0 && cmdd.abs() < neg_dist.abs() {
                            neg_dist = cmdd;
                        }
                    }

                    assert!(contour_distances.len() == windings.len());

                    let mut md = -1e24;
                    if pos_dist >= 0.0 && pos_dist.abs() <= neg_dist.abs() {
                        md = pos_dist;
                        winding = 1;
                        for (d, w) in contour_distances.iter().zip(windings.iter()) {
                            if *w > 0 && *d > md && d.abs() < neg_dist.abs() {
                                md = *d;
                            }
                        }
                    } else if neg_dist <= 0.0 && neg_dist.abs() <= pos_dist.abs() {
                        md = neg_dist;
                        winding = -1;
                        for (d, w) in contour_distances.iter().zip(windings.iter()) {
                            if *w < 0 && *d < md && d.abs() < pos_dist.abs() {
                                md = *d;
                            }
                        }
                    }
                    for (c, w) in contour_distances.iter().zip(windings.iter()) {
                        if *w != winding && c.abs() < md.abs() {
                            md = *c;
                        }
                    }

                    md / 0.5
                })
                .collect()
        })
        .collect()
}
