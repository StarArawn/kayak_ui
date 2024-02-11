pub mod scroll_bar;
pub mod scroll_box;
pub mod scroll_content;
pub mod scroll_context;

/// Maps a value from one range to another range
fn map_range(value: f32, from_range: (f32, f32), to_range: (f32, f32)) -> f32 {
    let from_diff = from_range.1 - from_range.0;
    if from_diff <= f32::EPSILON {
        value
    } else {
        to_range.0 + (value - from_range.0) * (to_range.1 - to_range.0) / from_diff
    }
}
