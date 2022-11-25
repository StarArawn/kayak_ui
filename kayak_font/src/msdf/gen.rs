use crate::msdf::{
    bitmap::{FloatRGB, FloatRGBBmp},
    edge_point::EdgePoint,
    shape::Shape,
    signed_distance::SignedDistance,
    vector::Vector2,
    MultiDistance,
};

use super::EdgeColor;

fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        b
    } else {
        a
    }
}

fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}

fn median<T: PartialOrd + Copy>(a: T, b: T, c: T) -> T {
    max(min(a, b), min(max(a, b), c))
}

pub fn pixel_clash(a: FloatRGB, b: FloatRGB, threshold: f64) -> bool {
    let mut a0 = a.r;
    let mut a1 = a.g;
    let mut a2 = a.b;
    let mut b0 = b.r;
    let mut b1 = b.g;
    let mut b2 = b.b;

    let mut tmp;
    if (b0 - a0).abs() < (b1 - a1).abs() {
        tmp = a0;
        a0 = a1;
        a1 = tmp;
        tmp = b0;
        b0 = b1;
        b1 = tmp;
    }

    if (b1 - a1).abs() < (b2 - a2).abs() {
        tmp = a1;
        a1 = a2;
        a2 = tmp;
        tmp = b1;
        b1 = b2;
        b2 = tmp;
        if (b0 - a0).abs() < (b1 - a1).abs() {
            tmp = a0;
            a1 = tmp;
            tmp = b0;
            b0 = b1;
            b1 = tmp;
        }
    }

    ((b1 - a1).abs() >= threshold as f32) && !(b0 == b1 && b0 == b2) && (a2 - 0.5).abs() >= (b2 - 0.5).abs()

    // let a_calcd = if a.r > 0.5 { 1.0 } else { 0.0 }
    //     + if a.g > 0.5 { 1.0 } else { 0.0 }
    //     + if a.b > 0.5 { 1.0 } else { 0.0 };
    // let b_calcd = if b.r > 0.5 { 1.0 } else { 0.0 }
    //     + if b.g > 0.5 { 1.0 } else { 0.0 }
    //     + if b.b > 0.5 { 1.0 } else { 0.0 };
    // let a_in = a_calcd >= 2.0;
    // let b_in = b_calcd >= 2.0;

    // if a_in != b_in {
    //     return false;
    // }

    // if (a.r > 0.5 && a.g > 0.5 && a.b > 0.5)
    //     || (a.r < 0.5 && a.g < 0.5 && a.b < 0.5)
    //     || (b.r > 0.5 && b.g > 0.5 && b.b > 0.5)
    //     || (b.r < 0.5 && b.g < 0.5 && b.b < 0.5)
    // {
    //     return false;
    // }

    // let aa;
    // let ab;
    // let ba;
    // let bb;
    // let ac;
    // let bc;

    // if (a.r > 0.5) != (b.r > 0.5) && (a.r < 0.5) != (b.r < 0.5) {
    //     aa = a.r;
    //     ba = b.r;
    //     if (a.g > 0.5) != (b.g > 0.5) && (a.g < 0.5) != (b.g < 0.5) {
    //         ab = a.g;
    //         bb = b.g;
    //         ac = a.b;
    //         bc = b.b;
    //     } else if (a.b > 0.5) != (b.b > 0.5) && (a.b < 0.5) != (b.b < 0.5) {
    //         ab = a.b;
    //         bb = b.b;
    //         ac = a.g;
    //         bc = b.g;
    //     } else {
    //         return false;
    //     }
    // } else if (a.g > 0.5) != (b.g > 0.5)
    //     && (a.g < 0.5) != (b.g < 0.5)
    //     && (a.b > 0.5) != (b.b > 0.5)
    //     && (a.b < 0.5) != (b.b < 0.5)
    // {
    //     aa = a.g;
    //     ba = b.g;
    //     ab = a.b;
    //     bb = b.b;
    //     ac = a.r;
    //     bc = b.r;
    // } else {
    //     return false;
    // }

    // return ((aa - ba).abs() >= threshold as f32)
    //     && ((ab - bb).abs() >= threshold as f32)
    //     && (ac - 0.5).abs() >= (bc - 0.5).abs();
}

pub fn msdf_error_correction(output: &mut FloatRGBBmp, threshold: Vector2) {
    let mut clashes: Vec<(usize, usize)> = Vec::new();
    let w = output.width();
    let h = output.height();
    for y in 0..h {
        for x in 0..w {
            if (x > 0
                && pixel_clash(
                    output.get_pixel(x, y),
                    output.get_pixel(x - 1, y),
                    threshold.x,
                ))
                || (x < w - 1
                    && pixel_clash(
                        output.get_pixel(x, y),
                        output.get_pixel(x + 1, y),
                        threshold.x,
                    ))
                || (y > 0
                    && pixel_clash(
                        output.get_pixel(x, y),
                        output.get_pixel(x, y - 1),
                        threshold.y,
                    ))
                || (y < h - 1
                    && pixel_clash(
                        output.get_pixel(x, y),
                        output.get_pixel(x, y + 1),
                        threshold.y,
                    ))
            {
                clashes.push((x, y));
            }
        }
    }
    let clash_count = clashes.len();
    for i in 0..clash_count {
        let clash = clashes[i];
        let pixel = output.get_pixel(clash.0, clash.1);
        let med = median(pixel.r, pixel.g, pixel.b);
        output.set_pixel(clash.0, clash.1, FloatRGB::new(med, med, med));
    }
}

pub fn generate_msdf(
    output: &mut FloatRGBBmp,
    shape: &Shape,
    range: f64,
    scale: Vector2,
    translate: Vector2,
    edge_threshold: f64,
) {
    let contours = &shape.contours;
    let contour_count = contours.len();
    let w = output.width();
    let h = output.height();
    let mut windings = Vec::with_capacity(contour_count);

    for contour in contours {
        windings.push(contour.winding());
    }

    let mut contour_sd = vec![MultiDistance::default(); contour_count];

    for y in 0..h {
        let row = if shape.inverse_y_axis { h - y - 1 } else { y };
        for x in 0..w {
            let p = (Vector2::new(x as f64 + 0.5, y as f64 + 0.5) / scale) - translate;
            let mut sr = EdgePoint {
                min_distance: SignedDistance::infinite(),
                near_edge: None,
                near_param: 0.0,
            };
            let mut sg = EdgePoint {
                min_distance: SignedDistance::infinite(),
                near_edge: None,
                near_param: 0.0,
            };
            let mut sb = EdgePoint {
                min_distance: SignedDistance::infinite(),
                near_edge: None,
                near_param: 0.0,
            };

            let mut d = SignedDistance::infinite().distance.abs();
            let mut neg_dist = -SignedDistance::infinite().distance.abs();
            let mut pos_dist = d;

            let mut winding = 0;

            for (n, contour) in contours.iter().enumerate() {
                let edges = &contour.edges;
                let mut r = EdgePoint {
                    min_distance: SignedDistance::infinite(),
                    near_edge: None,
                    near_param: 0.0,
                };
                let mut g = EdgePoint {
                    min_distance: SignedDistance::infinite(),
                    near_edge: None,
                    near_param: 0.0,
                };
                let mut b = EdgePoint {
                    min_distance: SignedDistance::infinite(),
                    near_edge: None,
                    near_param: 0.0,
                };
                for edge in edges {
                    let (distance, param) = edge.signed_distance(p);
                    if edge.has_color(EdgeColor::RED) && distance.l(&r.min_distance) {
                        r.min_distance = distance;
                        r.near_edge = Some(*edge);
                        r.near_param = param;
                    }
                    if edge.has_color(EdgeColor::GREEN) && distance.l(&g.min_distance) {
                        g.min_distance = distance;
                        g.near_edge = Some(*edge);
                        g.near_param = param;
                    }
                    if edge.has_color(EdgeColor::BLUE) && distance.l(&b.min_distance) {
                        b.min_distance = distance;
                        b.near_edge = Some(*edge);
                        b.near_param = param;
                    }
                }

                if r.min_distance.l(&sr.min_distance) {
                    sr = r;
                }
                if g.min_distance.l(&sg.min_distance) {
                    sg = g;
                }
                if b.min_distance.l(&sb.min_distance) {
                    sb = b;
                }

                let mut med_min_distance = median(
                    r.min_distance.distance,
                    g.min_distance.distance,
                    b.min_distance.distance,
                )
                .abs();

                if med_min_distance < d {
                    d = med_min_distance;
                    winding = -windings[n];
                }

                if let Some(near_edge) = &mut r.near_edge {
                    near_edge.distance_to_pseudo_distance(&mut r.min_distance, p, r.near_param);
                }
                if let Some(near_edge) = &mut g.near_edge {
                    near_edge.distance_to_pseudo_distance(&mut g.min_distance, p, g.near_param);
                }
                if let Some(near_edge) = &mut b.near_edge {
                    near_edge.distance_to_pseudo_distance(&mut b.min_distance, p, b.near_param);
                }

                med_min_distance = median(
                    r.min_distance.distance,
                    g.min_distance.distance,
                    b.min_distance.distance,
                );
                contour_sd[n].r = r.min_distance.distance;
                contour_sd[n].g = g.min_distance.distance;
                contour_sd[n].b = b.min_distance.distance;
                contour_sd[n].med = med_min_distance;
                if windings[n] > 0
                    && med_min_distance >= 0.0
                    && med_min_distance.abs() < pos_dist.abs()
                {
                    pos_dist = med_min_distance;
                }
                if windings[n] < 0
                    && med_min_distance <= 0.0
                    && med_min_distance.abs() < neg_dist.abs()
                {
                    neg_dist = med_min_distance;
                }
            }

            if let Some(near_edge) = &mut sr.near_edge {
                near_edge.distance_to_pseudo_distance(&mut sr.min_distance, p, sr.near_param);
            }
            if let Some(near_edge) = &mut sg.near_edge {
                near_edge.distance_to_pseudo_distance(&mut sg.min_distance, p, sg.near_param);
            }
            if let Some(near_edge) = &mut sb.near_edge {
                near_edge.distance_to_pseudo_distance(&mut sb.min_distance, p, sb.near_param);
            }

            let mut msd = MultiDistance::default();
            msd.r = SignedDistance::infinite().distance;
            msd.b = msd.r;
            msd.g = msd.r;
            msd.med = msd.r;
            if pos_dist >= 0.0 && pos_dist.abs() <= neg_dist.abs() {
                msd.med = SignedDistance::infinite().distance;
                winding = 1;
                for i in 0..contours.len() {
                    if windings[i] > 0
                        && contour_sd[i].med > msd.med
                        && contour_sd[i].med.abs() < neg_dist.abs()
                    {
                        msd = contour_sd[i];
                    }
                }
            } else if neg_dist <= 0.0 && neg_dist.abs() <= pos_dist.abs() {
                msd.med = -SignedDistance::infinite().distance;
                winding = -1;
                for i in 0..contours.len() {
                    if windings[i] < 0
                        && contour_sd[i].med < msd.med
                        && contour_sd[i].med.abs() < pos_dist.abs()
                    {
                        msd = contour_sd[i];
                    }
                }
            }

            for i in 0..contours.len() {
                if windings[i] != winding && contour_sd[i].med.abs() < msd.med.abs() {
                    msd = contour_sd[i];
                }
            }

            if median(
                sr.min_distance.distance,
                sg.min_distance.distance,
                sb.min_distance.distance,
            ) == msd.med
            {
                msd.r = sr.min_distance.distance;
                msd.g = sg.min_distance.distance;
                msd.b = sb.min_distance.distance;
            }

            output.set_pixel(
                x,
                row,
                FloatRGB {
                    r: (msd.r / range + 0.5) as f32,
                    g: (msd.g / range + 0.5) as f32,
                    b: (msd.b / range + 0.5) as f32,
                },
            )
        }

        if edge_threshold > 0.0 {
            msdf_error_correction(output, edge_threshold / (scale * range));
        }
    }
}
