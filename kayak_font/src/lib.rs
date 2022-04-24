mod atlas;
mod font;
mod glyph;
mod layout;
mod metrics;
mod sdf;
mod utility;

pub use atlas::*;
pub use font::*;
pub use glyph::*;
pub use layout::*;
pub use metrics::*;
pub use sdf::*;

#[cfg(feature = "bevy_renderer")]
pub mod bevy;

#[cfg(test)]
mod tests {
    use crate::{Alignment, KayakFont, Sdf, TextProperties};

    fn make_font() -> KayakFont {
        let bytes = std::fs::read("assets/roboto.kayak_font")
            .expect("a `roboto.kayak_font` file in the `assets/` directory of this crate");

        #[cfg(feature = "bevy_renderer")]
        return KayakFont::new(Sdf::from_bytes(&bytes), bevy::asset::Handle::default());

        #[cfg(not(feature = "bevy_renderer"))]
        return KayakFont::new(Sdf::from_bytes(&bytes));
    }

    fn make_properties() -> TextProperties {
        TextProperties {
            line_height: 14.0 * 1.2,
            font_size: 14.0,
            alignment: Alignment::Start,
            max_size: (200.0, 300.0),
            tab_size: 4,
        }
    }

    #[test]
    fn should_contain_correct_number_of_chars() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        assert_eq!(content.len(), layout.total_chars())
    }

    #[test]
    fn should_contain_correct_number_of_glyphs() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        // Since this string is pure ascii, we can just get the total characters - total whitespace
        let expected = content.split_whitespace().collect::<String>().len();

        assert_eq!(expected, layout.total_glyphs())
    }

    #[test]
    fn should_contain_correct_number_of_graphemes() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        // Since this string is pure ascii, we can just get the total characters
        let expected = content.len();

        assert_eq!(expected, layout.total_graphemes())
    }

    #[test]
    fn should_contain_correct_number_of_lines() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        assert_eq!(4, layout.total_lines())
    }

    #[test]
    fn should_find_line_containing_grapheme() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        let lines = [
            (content.find("Hello").unwrap(), content.rfind('\n').unwrap()),
            (
                content.find("How").unwrap(),
                content.rfind("this ").unwrap(),
            ),
            (
                content.find("super").unwrap(),
                content.rfind("doing ").unwrap(),
            ),
            (content.find("today").unwrap(), content.rfind('?').unwrap()),
        ];

        for (line_index, (line_start, line_end)) in lines.into_iter().enumerate() {
            let result = layout.find_grapheme(line_start).unwrap().row;
            assert_eq!(line_index, result);
            let result = layout.find_grapheme(line_end).unwrap().row;
            assert_eq!(line_index, result);
        }
    }

    #[test]
    fn grapheme_should_be_correct_position() {
        let content = "Hello world!\nHow is everyone on this super-awesome rock doing today?";
        let font = make_font();
        let properties = make_properties();
        let layout = font.measure(content, properties);

        for (line_index, line) in layout.lines().iter().enumerate() {
            let mut expected_x = 0.0;
            let expected_y = properties.line_height * line_index as f32;

            for grapheme in line.graphemes() {
                assert_eq!(expected_x, grapheme.position.0);
                assert_eq!(expected_y, grapheme.position.1);
                expected_x += grapheme.size.0;
            }
        }
    }
}
