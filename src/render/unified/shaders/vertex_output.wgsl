#define_import_path kayak_ui::vertex_output

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec3<f32>,
    @location(2) pos: vec2<f32>,
    @location(3) size: vec2<f32>,
    @location(4) border_radius: f32,
    @location(5) pixel_position: vec2<f32>,
};