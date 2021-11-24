use lyon_geom::math::{Angle, Vector};

use crate::contour::Contour;
use crate::ColorFlags;

/// Recolor the contours prior to MSDF computation.
/// This function uses a simple technique,
/// again based on Viktor's implementation.
/// It is left as a separate step so you can implement more complex techniques as desired.
pub fn recolor_contours(contours: Vec<Contour>, threshold: Angle, mut seed: u64) -> Vec<Contour> {
    let (threshold, _) = threshold.sin_cos();

    // Determine if a point is a corner, assuming i and o are incoming and
    // outgoing normalized direction vectors
    let is_corner = |i: Vector, o: Vector| {
        let d = i.dot(o); /* |i| |o| cos(t) */
        let c = i.cross(o).abs(); /* |i| |o| sin(t) */

        // if this corner turns more than 90 degrees (detected by dot product/cos)
        // or if it turns more than the threshold angle (detected by cross product/sin)
        (d <= 0.0) || (c > threshold)
    };
    contours
        .into_iter()
        .map(|mut c| {
            let mut corners = Vec::new();
            let n = c.elements.len();
            // Find all the corners
            if n != 0 {
                let mut prev_dir = c.elements[n - 1].direction(1.0).normalize();
                for (i, e) in c.elements.iter().enumerate() {
                    let c_dir = e.direction(0.0).normalize();
                    if is_corner(prev_dir, c_dir) {
                        corners.push(i)
                    }
                    prev_dir = e.direction(1.0).normalize();
                }
            }

            match corners.len() {
                0 => {
                    // The whole contour is smooth, and we initialized all colors to white.
                    // No work to do
                    c
                }
                1 => {
                    // "Teardrop" case: there is only one sharp corner so we
                    // just pick 3 colors up front and cycle through them
                    let mut colors = [
                        (ColorFlags::WHITE).switch(&mut seed),
                        ColorFlags::WHITE,
                        ColorFlags::WHITE,
                    ];
                    colors[1] = colors[0].switch(&mut seed);
                    colors[2] = colors[1].switch(&mut seed);
                    let corner = corners[0];
                    match n {
                        0 => {
                            unreachable!();
                        }
                        1 => {
                            // Only a single edge segment, but it's a teardrop.
                            // We split it in 3 to make the colors happen
                            let mut split = c.elements[0].split_in_thirds();
                            split[0].color = colors[0];
                            split[1].color = colors[1];
                            split[2].color = colors[2];

                            c.elements.clear();
                            c.elements.extend_from_slice(&split);
                        }
                        2 => {
                            // 2 segments. We split it into 6, and assign colors by hand
                            let mut split0 = c.elements[0].split_in_thirds();
                            let mut split1 = c.elements[1].split_in_thirds();
                            split0[0].color = colors[0];
                            split0[1].color = colors[0];
                            split0[2].color = colors[1];
                            split1[0].color = colors[1];
                            split1[1].color = colors[2];
                            split1[2].color = colors[2];

                            c.elements.clear();
                            c.elements.extend_from_slice(&split0);
                            c.elements.extend_from_slice(&split1);
                        }
                        _ => {
                            // We have more than 3 edges to rotate colors through
                            for (i, e) in c.elements.iter_mut().enumerate() {
                                // ported from this cursed C++ code:
                                // contour->edges[(corner+i)%m]->color = (colors+1)[int(3+2.875*i/(m-1)-1.4375+.5)-3];
                                let i = (n + i - corner) % n; // Emulate the ( corner + i) % m
                                let idx_fractional =
                                    3.5f32 + 2.875f32 * (i as f32) / ((n - 1) as f32) - 1.4375f32;
                                let idx = idx_fractional.floor() as usize - 2;
                                e.color = colors[idx];
                            }
                        }
                    }
                    c
                }
                _ => {
                    // We have 2 or more corners
                    // Cycle through colors, switching whenever we hit another corner
                    let n_corners = corners.len();
                    let mut spline = 0;
                    let start = corners[0];
                    let mut color = ColorFlags::WHITE;
                    color = color.switch(&mut seed);
                    let initial_color = color;
                    for i in 0..n {
                        let i = (start + i) % n;
                        if spline + 1 < n_corners && corners[spline + 1] == i {
                            spline = spline + 1;
                            color = color.switch_banned(&mut seed, initial_color);
                        }
                        c.elements[i].color = color;
                    }
                    c
                }
            }
        })
        .collect()
}
