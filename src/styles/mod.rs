use bevy::{
    prelude::{Color, Component, Vec2},
    reflect::Reflect,
};

mod corner;
mod edge;
mod options_ref;
mod render_command;
mod style;
mod units;

pub use corner::Corner;
pub use edge::Edge;
use fancy_regex::Matches;
pub use options_ref::AsRefOption;
pub use render_command::RenderCommand;
pub use style::*;
pub use units::*;

#[derive(Component, Reflect, Debug, Default, Clone, PartialEq)]
pub struct ComputedStyles(pub KStyle);

impl From<KStyle> for ComputedStyles {
    fn from(val: KStyle) -> Self {
        ComputedStyles(val)
    }
}

#[derive(Reflect, Clone, Copy, Default, Debug, PartialEq)]
pub struct BoxShadow {
    pub color: Color,
    pub radius: f32,
    pub offset: Vec2,
    pub spread: Vec2,
}

fn is_length(v: &str) -> bool {
    v == "0"
        || fancy_regex::Regex::new(r"^[0-9]+[a-zA-Z%]+?$")
            .map(|m| m.is_match(v).unwrap_or(false))
            .unwrap_or(false)
}

impl BoxShadow {
    pub fn from_string(s: impl ToString) -> Vec<BoxShadow> {
        let box_shadow_string: String = s.to_string();
        let box_shadow_string = box_shadow_string
            .replace("box-shadow: ", "")
            .replace(';', "");

        let values_parsed = fancy_regex::Regex::new(r",(?![^\(]*\))").unwrap();
        let split_regex = fancy_regex::Regex::new(r"\s(?![^(]*\))").unwrap();

        let mut box_shadows = vec![];
        let values_split = Split {
            finder: values_parsed.find_iter(&box_shadow_string),
            last: 0,
        };
        for value in values_split.map(|s| s.trim()) {
            // Split single shadow
            let parts = Split {
                finder: split_regex.find_iter(value),
                last: 0,
            }
            .collect::<Vec<_>>();
            let _inset = parts.contains(&"inset");
            let color = parts
                .last()
                .map(|last| {
                    if last.contains("rgb") || last.contains('#') {
                        Some(*last)
                    } else {
                        None
                    }
                })
                .and_then(|s| s)
                .unwrap_or(parts.first().cloned().unwrap());

            let nums = parts
                .iter()
                .filter(|n| **n != "inset")
                .filter(|n| color != **n)
                .map(|v| v.replace("px", "").parse::<f32>().unwrap_or(0.0))
                .collect::<Vec<f32>>();

            let offset_x = nums.first().copied().unwrap_or(0.0);
            let offset_y = nums.get(1).copied().unwrap_or(0.0);
            let blur_radius = nums.get(2).copied().unwrap_or(0.0);
            let spread = nums.get(3).copied().unwrap_or(0.0);

            let color = if is_rgba(color) {
                parse_rgba(color)
            } else {
                Color::hex(color).unwrap_or_default()
            };

            box_shadows.push(BoxShadow {
                color,
                radius: blur_radius,
                offset: Vec2::new(offset_x, offset_y),
                spread: Vec2::splat(spread),
            });
        }

        box_shadows
    }
}

fn is_rgba(s: &str) -> bool {
    s.contains("rgba") || s.contains("rgb")
}

fn parse_rgba(s: &str) -> Color {
    let s = s.replace("rgba(", "").replace("rgb(", "").replace(')', "");
    let values = s.split(',').collect::<Vec<_>>();

    let r = values
        .first()
        .map(|s| s.trim().parse::<f32>().map(|v| v / 255.0).unwrap_or(0.0))
        .unwrap_or(0.0);
    let g = values
        .get(1)
        .map(|s| s.trim().parse::<f32>().map(|v| v / 255.0).unwrap_or(0.0))
        .unwrap_or(0.0);
    let b = values
        .get(2)
        .map(|s| s.trim().parse::<f32>().map(|v| v / 255.0).unwrap_or(0.0))
        .unwrap_or(0.0);
    let a = values
        .get(3)
        .map(|s| s.trim().parse::<f32>().unwrap_or(1.0))
        .unwrap_or(1.0);

    Color::rgba(r, g, b, a)
}

mod tests {
    #[test]
    fn test_box_shadow_from_string() {
        let box_shadow_string = "box-shadow: rgba(50, 50, 93, 0.25) 0px 50px 100px -20px, rgba(0, 0, 0, 0.3) 0px 30px 60px -30px;";
        let result = crate::styles::BoxShadow::from_string(box_shadow_string);

        dbg!(result);

        // assert!(true == false);
    }
}

/// Yields all substrings delimited by a regular expression match.
///
/// `'r` is the lifetime of the compiled regular expression and `'t` is the
/// lifetime of the string being split.
#[derive(Debug)]
pub struct Split<'r, 't> {
    finder: Matches<'r, 't>,
    last: usize,
}

impl<'r, 't> Iterator for Split<'r, 't> {
    type Item = &'t str;

    fn next(&mut self) -> Option<&'t str> {
        let text = self.finder.text();
        match self.finder.next() {
            None => {
                if self.last > text.len() {
                    None
                } else {
                    let s = &text[self.last..];
                    self.last = text.len() + 1; // Next call will return None
                    Some(s)
                }
            }
            Some(m) => match m {
                Ok(m) => {
                    let matched = &text[self.last..m.start()];
                    self.last = m.end();
                    Some(matched)
                }
                Err(_) => None,
            },
        }
    }
}
