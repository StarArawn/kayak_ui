use std::collections::{HashMap, HashSet};

use lyon_geom::math::{Angle, Point, Rect, Size};
use lyon_path::builder::FlatPathBuilder;

use crate::{compute_msdf, recolor_contours, rescale_contours, PathCollector};

#[derive(Debug, Clone)]
pub struct FontCache {
    count: usize,
    pub dimensions: (u32, u32),
    chars: HashMap<char, (usize, Vec<u8>)>,
    needs_processing: HashSet<usize>,
    id_to_char_mappings: HashMap<usize, char>,
}

impl FontCache {
    pub fn new(texture_size: (u32, u32)) -> Self {
        Self {
            count: 0,
            dimensions: texture_size,
            chars: HashMap::default(),
            needs_processing: HashSet::default(),
            id_to_char_mappings: HashMap::default(),
        }
    }

    pub fn add_character(&mut self, c: char) {
        self.chars.insert(c, (self.count, vec![]));
        self.id_to_char_mappings.insert(self.count, c);
        self.count += 1;
    }

    fn set_texture(&mut self, c: char, texture_data: Vec<Vec<(f32, f32, f32)>>) {
        // let pixels: Vec<u8> = texture_data
        //     .iter()
        //     .flat_map(|y| {
        //         y.iter().flat_map(|pixel| {
        //             vec![
        //                 (pixel.0 * 255.0) as u8,
        //                 (pixel.1 * 255.0) as u8,
        //                 (pixel.2 * 255.0) as u8,
        //                 255u8,
        //             ]
        //         })
        //     })
        //     .collect();
        let pixels = texture_data
            .iter()
            .flat_map(|x| {
                x.iter()
                    .flat_map(|p| {
                        vec![
                            p.0.to_le_bytes(),
                            p.1.to_le_bytes(),
                            p.2.to_le_bytes(),
                            1.0f32.to_le_bytes(),
                        ]
                        .into_iter()
                        .flatten()
                        .collect::<Vec<u8>>()
                    })
                    .collect::<Vec<u8>>()
            })
            .collect();
        self.chars.insert(c, (self.count, pixels));
        self.needs_processing.insert(self.count);
        self.id_to_char_mappings.insert(self.count, c);
        self.count += 1;
    }

    pub fn has_character(&self, c: char) -> bool {
        self.chars.contains_key(&c)
    }

    fn get_dimensions(&self) -> (u32, u32) {
        self.dimensions
    }
}

#[derive(Debug, Clone)]
pub struct Font {
    internal_face: ttf_parser::Face<'static>,
    font: fontdue::Font,
    pub cache: FontCache,
}

impl Font {
    pub fn new(font_data: &'static [u8], texture_size: (u32, u32)) -> Font {
        Font {
            internal_face: ttf_parser::Face::from_slice(&font_data, 0).unwrap(),
            font: fontdue::Font::from_bytes(font_data.clone(), fontdue::FontSettings::default())
                .unwrap(),
            cache: FontCache::new(texture_size),
        }
    }

    /// Adds all of the common known characters.
    pub fn add_all_common(&mut self) {
        let chars = vec![
            '`', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '~', '!', '@', '#',
            '$', '%', '^', '&', '*', '(', ')', '_', '+', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i',
            'o', 'p', '[', ']', '\\', 'Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P', '{', '}',
            '|', 'a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', 'A', 'S', 'D', 'F', 'G',
            'H', 'J', 'K', 'L', ':', '"', 'z', 'x', 'c', 'v', 'b', 'n', 'n', 'm', ',', '.', '/',
            'Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?',
        ];

        for char in chars {
            self.add_character(char);
        }
    }

    pub fn get_layout(
        &self,
        content: &String,
        font_size: f32,
        original_font_size: f32,
        max_glyph_size: (f32, f32),
    ) -> Vec<(char, (f32, f32), (f32, f32))> {
        let mut layout =
            fontdue::layout::Layout::new(fontdue::layout::CoordinateSystem::PositiveYDown);
        layout.append(
            &[&self.font],
            &fontdue::layout::TextStyle::new(content, font_size, 0),
        );
        let font_ratio = font_size / original_font_size;
        let resized_max_glyph_size = (
            (max_glyph_size.0 * font_ratio).round(),
            (max_glyph_size.1 * font_ratio).round(),
        );

        let glyphs = layout.glyphs();
        let glyphs: Vec<_> = glyphs
            .iter()
            .filter_map(|glyph_position| {
                if glyph_position.parent == ' ' {
                    return None;
                }
                // let metrics = self.font.metrics(glyph_position.parent, font_size);

                let shift_y = resized_max_glyph_size.1 - glyph_position.height as f32;
                Some((
                    glyph_position.parent,
                    (glyph_position.x, glyph_position.y - shift_y),
                    resized_max_glyph_size,
                ))
            })
            .collect();
        glyphs
    }

    // pub fn get_size(&self, c: char, font_size: f32) -> (f32, f32) {
    //     if let Some(glyph) = self.internal_face.glyph_index(c) {
    //         // Collect our path's from the glyph's outline shape.
    //         let mut path_collector = PathCollector::new();
    //         let rect = self
    //             .internal_face
    //             .outline_glyph(glyph, &mut path_collector)
    //             .unwrap();
    //         let metrics = font_size / self.font.units_per_em();

    //         (width as f32 * metrics, height as f32 * metrics)
    //     } else {
    //         panic!("")
    //     }
    // }

    pub fn get_char_id(&self, c: char) -> usize {
        if self.cache.has_character(c) {
            if let Some((id, _)) = self.cache.chars.get(&c) {
                return *id;
            }
        }
        panic!("No char found!");
    }

    pub fn add_character(&mut self, c: char) {
        if !self.cache.has_character(c) {
            if let Some(glyph) = self.internal_face.glyph_index(c) {
                // Collect our path's from the glyph's outline shape.
                let mut path_collector = PathCollector::new();
                let rect = self
                    .internal_face
                    .outline_glyph(glyph, &mut path_collector)
                    .unwrap();
                let contours = path_collector.build();

                // Bounds of our texture in UV's
                // TODO: Allow this to change because some people may want texture atlases instead.
                let uv_rect = Rect::new(Point::new(0.0, 0.0), Size::new(1.0, 1.0));

                // Bounds of our rect in font space coords.
                let font_rect = Rect::new(
                    Point::new(rect.x_min as f32, rect.y_min as f32),
                    Size::new(rect.width() as f32, rect.height() as f32),
                );

                let (contours, _transform) = rescale_contours(
                    contours,
                    font_rect,
                    uv_rect,
                    self.internal_face.units_per_em(),
                );
                let contours = recolor_contours(contours, Angle::degrees(3.0), 1);
                let msdf = compute_msdf(&contours, self.cache.get_dimensions().0 as usize);

                self.cache.set_texture(c, msdf);
            }
        }
    }

    pub fn get_data_to_process<'b>(&'b mut self) -> Vec<(char, usize, &'b Vec<u8>)> {
        let data = self
            .cache
            .needs_processing
            .iter()
            .filter_map(|unprocessed_id| {
                if let Some(c) = self.cache.id_to_char_mappings.get(unprocessed_id) {
                    if let Some((_, data)) = self.cache.chars.get(c) {
                        return Some((*c, *unprocessed_id, data));
                    }
                }

                None
            })
            .collect();

        self.cache.needs_processing.clear();

        data
    }

    // Checks the given chars and returns ones that haven't been seen before.
    pub fn check_chars(&self, chars: std::str::Chars<'_>) -> Vec<char> {
        chars
            .into_iter()
            .filter(|c| !self.cache.chars.contains_key(&c))
            .collect()
    }

    pub fn units_per_em(&self) -> f32 {
        self.font.units_per_em()
    }
}

fn get_new_size(org_width: f32, new_width: f32, org_height: f32, new_height: f32) -> (f32, f32) {
    let ratio = calculate_ratio(org_width, new_width, org_height, new_height);
    // let ratio = new_width / new_height;
    (org_width * ratio, org_height * ratio)
}

pub fn calculate_ratio(org_width: f32, new_width: f32, org_height: f32, new_height: f32) -> f32 {
    let mut area_size = 0.0;
    let mut image_size = 0.0;

    if new_height * org_width > new_width * org_height {
        area_size = new_height;
        image_size = org_height;
    } else {
        area_size = new_width;
        image_size = org_width;
    }

    let ratio = area_size / image_size;
    ratio
}
