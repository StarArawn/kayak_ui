#![allow(dead_code)]

use crate::msdf::{edge_segment::EdgeSegment, shape::Shape, vector::Vector2, EdgeColor};

fn is_corner(a_dir: Vector2, b_dir: Vector2, cross_threshold: f64) -> bool {
    Vector2::dot_product(a_dir, b_dir) <= 0.0
        || Vector2::cross_product(a_dir, b_dir).abs() > cross_threshold
}

const MSDFGEN_EDGE_LENGTH_PRECISION: usize = 4;

fn estimate_edge_length(edge: &EdgeSegment) -> f64 {
    let mut len = 0.0;
    let mut prev = edge.point(0.0);
    for i in 1..MSDFGEN_EDGE_LENGTH_PRECISION {
        let cur = edge.point(1.0 / MSDFGEN_EDGE_LENGTH_PRECISION as f64 * i as f64);
        len += (cur - prev).length();
        prev = cur;
    }
    len
}

fn switch_color(color: &mut EdgeColor, seed: &mut usize, banned: EdgeColor) {
    let combined: EdgeColor =
        num::cast::FromPrimitive::from_usize(*color as usize & banned as usize).unwrap();

    if combined == EdgeColor::RED || combined == EdgeColor::GREEN || combined == EdgeColor::BLUE {
        *color =
            num::cast::FromPrimitive::from_usize(combined as usize ^ EdgeColor::WHITE as usize)
                .unwrap();
        return;
    }
    if *color == EdgeColor::BLACK || *color == EdgeColor::WHITE {
        match *seed % 3 {
            0 => {
                *color = EdgeColor::CYAN;
            }
            1 => {
                *color = EdgeColor::MAGENTA;
            }
            2 => {
                *color = EdgeColor::YELLOW;
            }
            _ => panic!("Not supported!"),
        }

        *seed /= 3;
        return;
    }

    let shifted = (*color as usize) << (1 + (*seed & 1));
    *color = num::cast::FromPrimitive::from_usize(
        (shifted | shifted >> 3) & (EdgeColor::WHITE as usize),
    )
    .unwrap();
    *seed >>= 1;
}

pub fn simple(shape: &mut Shape, angle_threshold: f64, mut seed: usize) {
    let cross_threshold = angle_threshold.sin();
    let mut corners = Vec::new();

    for contour in shape.contours.iter_mut() {
        corners.clear();

        let edges = &mut contour.edges;

        let edge_count = edges.len();
        if edge_count != 0 {
            let mut prev_dir = edges.last().unwrap().direction(1.0);
            #[allow(clippy::needless_range_loop)]
            for i in 0..edge_count {
                let edge = &edges[i];
                if is_corner(
                    prev_dir.normalize(false),
                    edge.direction(0.0).normalize(false),
                    cross_threshold,
                ) {
                    corners.push(i);
                }
                prev_dir = edge.direction(1.0);
            }
        }

        if corners.is_empty() {
            #[allow(clippy::needless_range_loop)]
            for i in 0..edge_count {
                edges[i].set_color(EdgeColor::WHITE);
            }
        } else if corners.len() == 1 {
            let mut colors = vec![EdgeColor::WHITE, EdgeColor::WHITE, EdgeColor::BLACK];
            switch_color(&mut colors[0], &mut seed, EdgeColor::BLACK);
            colors[2] = colors[0];
            switch_color(&mut colors[2], &mut seed, EdgeColor::BLACK);

            let corner = corners[0];
            if edge_count >= 3 {
                let m = edge_count;
                for i in 0..m {
                    let lookup =
                        ((3.0 + 2.875 * i as f64 / (m as f64 - 1.0) - 1.4375 + 0.5) as i32 - 3) + 1;
                    contour.edges[(corner + i) % m].set_color(colors[lookup as usize]);
                }
            } else if edge_count >= 1 {
                let mut parts = [EdgeSegment::default(); 7];

                let (o1, o2, o3) = edges[0].split_in_thirds();
                parts[3 * corner] = o1;
                parts[1 + 3 * corner] = o2;
                parts[2 + 3 * corner] = o3;

                if edge_count >= 2 {
                    let (o1, o2, o3) = edges[1].split_in_thirds();
                    parts[3 - 3 * corner] = o1;
                    parts[4 - 3 * corner] = o2;
                    parts[5 - 3 * corner] = o3;
                    parts[1].set_color(colors[0]);
                    parts[0].set_color(parts[1].get_color());
                    parts[3].set_color(colors[1]);
                    parts[2].set_color(parts[3].get_color());
                    parts[5].set_color(colors[2]);
                    parts[4].set_color(parts[5].get_color());
                } else {
                    parts[0].set_color(colors[0]);
                    parts[1].set_color(colors[1]);
                    parts[2].set_color(colors[2]);
                }
                edges.clear();
                #[allow(clippy::needless_range_loop)]
                for i in 0..7 {
                    edges.push(parts[i]);
                }
            }
        } else {
            let corner_count = corners.len();
            let mut spline = 0;
            let start = corners[0];

            let mut color = EdgeColor::WHITE;
            switch_color(&mut color, &mut seed, EdgeColor::BLACK);
            let initial_color = color;
            for i in 0..edge_count {
                let index = (start + i) % edge_count;
                if spline + 1 < corner_count && corners[spline + 1] == index {
                    spline += 1;
                    let banned_color =
                        (if spline == corner_count - 1 { 1 } else { 0 }) * initial_color as usize;
                    switch_color(
                        &mut color,
                        &mut seed,
                        num::cast::FromPrimitive::from_usize(banned_color).unwrap(),
                    );
                }
                edges[index].set_color(color);
            }
        }
    }
}

struct EdgeColoringInkTrapCorner {
    pub index: i32,
    pub prev_edge_length_estimate: f64,
    pub minor: bool,
    pub color: EdgeColor,
    pub spline_length: f64,
}

pub fn ink_trap(shape: &mut Shape, angle_threshold: f64, mut seed: usize) {
    let cross_threshold = angle_threshold.sin();
    let mut corners = Vec::new();
    for contour in shape.contours.iter_mut() {
        let mut spline_length = 0.0;
        corners.clear();
        if !contour.edges.is_empty() {
            let mut prev_direction = contour.edges.last().unwrap().direction(1.0);
            let mut index = 0;
            #[allow(clippy::explicit_counter_loop)]
            for edge in contour.edges.iter() {
                if is_corner(
                    prev_direction.normalize(false),
                    edge.direction(0.0).normalize(false),
                    cross_threshold,
                ) {
                    let corner = EdgeColoringInkTrapCorner {
                        index,
                        spline_length,
                        color: EdgeColor::BLACK,
                        prev_edge_length_estimate: 0.0,
                        minor: false,
                    };
                    corners.push(corner);
                    spline_length = 0.0;
                }
                spline_length += estimate_edge_length(edge);
                prev_direction = edge.direction(1.0);
                index += 1;
            }
        }

        if corners.is_empty() {
            for edge in contour.edges.iter_mut() {
                edge.set_color(EdgeColor::WHITE);
            }
        } else if corners.len() == 1 {
            let mut colors = vec![EdgeColor::WHITE, EdgeColor::WHITE, EdgeColor::BLACK];
            switch_color(&mut colors[0], &mut seed, EdgeColor::BLACK);
            colors[2] = colors[0];
            switch_color(&mut colors[2], &mut seed, EdgeColor::BLACK);
            let corner = corners[0].index as usize;
            if contour.edges.len() >= 3 {
                let m = contour.edges.len();
                for i in 0..m {
                    let lookup =
                        ((3.0 + 2.875 * i as f64 / (m as f64 - 1.0) - 1.4375 + 0.5) as i32 - 3) + 1;
                    contour.edges[(corner + i) % m].set_color(colors[lookup as usize]);
                }
            } else if !contour.edges.is_empty() {
                let mut parts = vec![EdgeSegment::default(); 7];
                let (o1, o2, o3) = contour.edges[0].split_in_thirds();
                parts[3 * corner] = o1;
                parts[1 + 3 * corner] = o2;
                parts[2 + 3 * corner] = o3;
                if contour.edges.len() >= 2 {
                    let (o1, o2, o3) = contour.edges[1].split_in_thirds();
                    parts[3 - 3 * corner] = o1;
                    parts[4 - 3 * corner] = o2;
                    parts[5 - 3 * corner] = o3;
                    parts[1].set_color(colors[0]);
                    let part1_color = parts[1].get_color();
                    parts[0].set_color(part1_color);
                    parts[3].set_color(colors[1]);
                    let part3_color = parts[3].get_color();
                    parts[2].set_color(part3_color);
                    parts[5].set_color(colors[2]);
                    let part5_color = parts[5].get_color();
                    parts[4].set_color(part5_color);
                } else {
                    parts[0].set_color(colors[0]);
                    parts[1].set_color(colors[1]);
                    parts[2].set_color(colors[2]);
                }
                contour.edges.clear();
                for part in parts.into_iter() {
                    contour.edges.push(part);
                }
            } else {
                let corner_count = corners.len();
                let mut major_corner_count = corner_count;

                if corner_count > 3 {
                    corners.first_mut().unwrap().prev_edge_length_estimate += spline_length;
                    for i in 0..corner_count {
                        if corners[i].prev_edge_length_estimate
                            > corners[(i + 1) % corner_count].prev_edge_length_estimate
                            && corners[(i + 1) % corner_count].prev_edge_length_estimate
                                < corners[(i + 2) % corner_count].prev_edge_length_estimate
                        {
                            corners[i].minor = true;
                            major_corner_count -= 1;
                        }
                    }

                    let mut color = EdgeColor::WHITE;
                    let mut initial_color = EdgeColor::BLACK;
                    #[allow(clippy::needless_range_loop)]
                    for i in 0..corner_count {
                        if !corners[i].minor {
                            major_corner_count -= 1;
                            switch_color(
                                &mut color,
                                &mut seed,
                                num::cast::FromPrimitive::from_usize(
                                    !major_corner_count * initial_color as usize,
                                )
                                .unwrap(),
                            );
                            corners[i].color = color;
                            if initial_color != EdgeColor::BLACK {
                                initial_color = color;
                            }
                        }
                    }
                    for i in 0..corner_count {
                        if corners[i].minor {
                            let next_color = corners[(i + 1) % corner_count].color;
                            corners[i].color = num::cast::FromPrimitive::from_usize(
                                (color as usize & next_color as usize) ^ EdgeColor::WHITE as usize,
                            )
                            .unwrap();
                        } else {
                            color = corners[i].color;
                        }
                    }

                    let mut spline = 0;
                    let start = corners[0].index as usize;
                    let mut color = corners[0].color;
                    let m = contour.edges.len();
                    for i in 0..m {
                        let index = (start + i) % m;
                        if spline + 1 < corner_count && corners[spline + 1].index as usize == index
                        {
                            spline += 1;
                            color = corners[spline].color;
                        }
                        contour.edges[index].set_color(color);
                    }
                }
            }
        }
    }
}
