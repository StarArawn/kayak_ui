use std::{fs::File, io::BufWriter};

use kayak_font::Font;

fn main() {
    let font_bytes = include_bytes!("../resources/Roboto-Regular.ttf");
    let mut font = Font::new(font_bytes, 64);

    font.add_character('A');
    font.add_character('B');
    font.add_character('C');
    font.add_character('!');
    font.add_character('&');

    // Characters that have already been calculated wont be calculated again!
    for _ in 0..1000000 {
        font.add_character('A');
    }

    let dimensions = font.cache.dimensions;
    for (c, _, pixels) in font.get_data_to_process() {
        let file = File::create(format!("./test-{}.png", c)).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, dimensions, dimensions);
        encoder.set_color(png::ColorType::RGBA);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixels).unwrap();
    }
}
