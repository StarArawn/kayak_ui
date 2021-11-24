use lyon_geom::math::Vector;

use crate::{contour::Contour, median, utils::EdgeDistance, ColorFlags};

/// Computes an MSDF from a list of contours. The returned vectors are a `dim` by `dim`
/// matrix of signed distance values. The output represents the signed distances to the contours
/// within [0, 1] x [0, 1]
pub fn compute_msdf(contours: &[Contour], dim: usize) -> Vec<Vec<(f32, f32, f32)>> {
    #[derive(Copy, Clone, PartialEq)]
    struct MultiDistance {
        r: f32,
        g: f32,
        b: f32,
        med: f32,
    }
    impl MultiDistance {
        fn new(v: f32) -> Self {
            Self {
                r: v,
                g: v,
                b: v,
                med: v,
            }
        }
    }
    let scale: f32 = 1.0 / (dim as f32);
    let windings: Vec<i32> = contours.iter().map(|c| c.winding() as i32).collect();

    (0..dim)
        .map(|y| {
            let py = (y as f32 + 0.5) * scale;
            (0..dim)
                .map(|x| {
                    // We assume there is at least 1 contour
                    // If there isn't make everything magenta
                    if contours.len() == 0 {
                        return (1.0f32, 0.0, 1.0);
                    }

                    let px = (x as f32 + 0.5) * scale;
                    let p = Vector::new(px, py);

                    let mut neg_dist = 1e24f32;
                    let mut pos_dist = -1e24f32;
                    let mut d = 1e24f32;
                    let mut winding = 0;
                    let mut contour_distances = Vec::new();
                    contour_distances.reserve(contours.len());

                    let mut sr = EdgeDistance::new();
                    let mut sg = EdgeDistance::new();
                    let mut sb = EdgeDistance::new();

                    for (i, contour) in contours.iter().enumerate() {
                        let mut contour_min_r = EdgeDistance::new();
                        let mut contour_min_g = EdgeDistance::new();
                        let mut contour_min_b = EdgeDistance::new();

                        for elem in &contour.elements {
                            let (d, na) = elem.distance(p.to_point());

                            if elem.color.contains(ColorFlags::RED) && d < contour_min_r.dist {
                                contour_min_r.dist = d;
                                contour_min_r.edge = Some(&elem);
                                contour_min_r.nearest_approach = na;
                            }
                            if elem.color.contains(ColorFlags::GREEN) && d < contour_min_g.dist {
                                contour_min_g.dist = d;
                                contour_min_g.edge = Some(&elem);
                                contour_min_g.nearest_approach = na;
                            }
                            if elem.color.contains(ColorFlags::BLUE) && d < contour_min_b.dist {
                                contour_min_b.dist = d;
                                contour_min_b.edge = Some(&elem);
                                contour_min_b.nearest_approach = na;
                            }
                        }

                        if contour_min_r.dist < sr.dist {
                            sr = contour_min_r;
                        }
                        if contour_min_g.dist < sg.dist {
                            sg = contour_min_g;
                        }
                        if contour_min_b.dist < sb.dist {
                            sb = contour_min_b;
                        }

                        let med_min_dist = median(
                            contour_min_r.dist.distance,
                            contour_min_g.dist.distance,
                            contour_min_b.dist.distance,
                        )
                        .abs();
                        if med_min_dist < d {
                            d = med_min_dist;
                            winding = -windings[i];
                        }

                        contour_min_r.to_pseudodistance(p);
                        contour_min_g.to_pseudodistance(p);
                        contour_min_b.to_pseudodistance(p);

                        let med_min_dist = median(
                            contour_min_r.dist.distance,
                            contour_min_g.dist.distance,
                            contour_min_b.dist.distance,
                        );

                        let mut msd = MultiDistance::new(med_min_dist);
                        msd.r = contour_min_r.dist.distance;
                        msd.g = contour_min_g.dist.distance;
                        msd.b = contour_min_b.dist.distance;
                        msd.med = med_min_dist;
                        contour_distances.push(msd);
                        if windings[i] > 0
                            && med_min_dist >= 0.0
                            && med_min_dist.abs() < pos_dist.abs()
                        {
                            pos_dist = med_min_dist;
                        }
                        if windings[i] < 0
                            && med_min_dist <= 0.0
                            && med_min_dist.abs() < neg_dist.abs()
                        {
                            neg_dist = med_min_dist;
                        }
                    }

                    assert!(contour_distances.len() == windings.len());

                    sr.to_pseudodistance(p);
                    sg.to_pseudodistance(p);
                    sb.to_pseudodistance(p);

                    let mut mmsd = MultiDistance::new(-1e24);
                    if pos_dist >= 0.0 && pos_dist.abs() <= neg_dist.abs() {
                        mmsd.med = -1e24;
                        winding = 1;
                        for (csd, cw) in contour_distances.iter().zip(windings.iter()) {
                            if *cw > 0 && csd.med > mmsd.med && csd.med.abs() < neg_dist.abs() {
                                mmsd = *csd;
                            }
                        }
                    } else if neg_dist <= 0.0 && neg_dist.abs() <= pos_dist.abs() {
                        mmsd.med = 1e24;
                        winding = -1;
                        for (csd, cw) in contour_distances.iter().zip(windings.iter()) {
                            if *cw < 0 && csd.med < mmsd.med && csd.med.abs() < pos_dist.abs() {
                                mmsd = *csd;
                            }
                        }
                    }
                    for (csd, w) in contour_distances.iter().zip(windings.iter()) {
                        if *w != winding && csd.med.abs() < mmsd.med.abs() {
                            mmsd = *csd;
                        }
                    }

                    if median(sr.dist.distance, sg.dist.distance, sb.dist.distance) == mmsd.med {
                        mmsd.r = sr.dist.distance;
                        mmsd.g = sg.dist.distance;
                        mmsd.b = sb.dist.distance;
                    }

                    (mmsd.r / 0.5, mmsd.g / 0.5, mmsd.b / 0.5)
                })
                .collect()
        })
        .collect()
}
