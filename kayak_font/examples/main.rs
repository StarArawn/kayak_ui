use std::{fs::File, io::BufWriter};

use kayak_font::{
    compute_msdf, recolor_contours, rescale_contours, Angle, FlatPathBuilder, PathCollector, Point,
    Rect,
};

pub fn main() {
    let font_data = include_bytes!("../resources/Roboto-Regular.ttf");
    let face = ttf_parser::Face::from_slice(font_data, 0).unwrap();

    let char_dim: u32 = 64;

    let c = 'A';
    if let Some(glyph) = face.glyph_index(c) {
        let mut path_collector = PathCollector::new();
        path_collector.scale = 1.0; //0.0001; //1024 as f32 / face.units_per_em() as f32;
        let rect = face.outline_glyph(glyph, &mut path_collector).unwrap();
        let contours = path_collector.build();
        let uv_rect = Rect::new(Point::new(0.0, 0.0), lyon_geom::math::Size::new(1.0, 1.0));

        let font_rect = Rect::new(
            Point::new(rect.x_min as f32, rect.y_min as f32),
            lyon_geom::math::Size::new(rect.width() as f32, rect.height() as f32),
        );

        let (contours, _) = rescale_contours(contours, font_rect, uv_rect, 0);

        let contours = recolor_contours(contours, Angle::degrees(3.0), 1);
        let msdf = compute_msdf(&contours, char_dim as usize);

        let file = File::create(format!("./test-{}.png", c)).unwrap();
        let ref mut w = BufWriter::new(file);

        let pixels: Vec<u8> = msdf
            .iter()
            .flat_map(|y| {
                y.iter().flat_map(|pixel| {
                    vec![
                        (pixel.0 * 255.0) as u8,
                        (pixel.1 * 255.0) as u8,
                        (pixel.2 * 255.0) as u8,
                        255u8,
                    ]
                })
            })
            .collect();

        let mut encoder = png::Encoder::new(w, char_dim, char_dim);
        encoder.set_color(png::ColorType::RGBA);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }
}
