[[block]]
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] uv: vec3<f32>;
    [[location(2)]] pos: vec2<f32>;
    [[location(3)]] size: vec2<f32>;
    [[location(4)]] screen_position: vec2<f32>;
    [[location(5)]] border_radius: f32;
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_color: vec4<f32>,
    [[location(2)]] vertex_uv: vec4<f32>,
    [[location(3)]] vertex_pos_size: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex_color;
    out.pos = vertex_pos_size.xy;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    out.screen_position = (view.view_proj * vec4<f32>(vertex_position, 1.0)).xy;
    out.uv = vertex_uv.xyz;
    out.size = vertex_pos_size.zw;
    out.border_radius = vertex_uv.w;
    return out;
}

[[group(1), binding(0)]]
var font_texture: texture_2d_array<f32>;
[[group(1), binding(1)]]
var font_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var px_range = 2.5;
    var tex_dimensions = textureDimensions(font_texture);
    var msdf_unit = vec2<f32>(px_range, px_range) / vec2<f32>(f32(tex_dimensions.x), f32(tex_dimensions.y));
    var x = textureSample(font_texture, font_sampler, vec2<f32>(in.uv.x, in.uv.y), i32(in.uv.z)); 
    var v = max(min(x.r, x.g), min(max(x.r, x.g), x.b));
    var sig_dist = (v - 0.5) * dot(msdf_unit, 0.5 / fwidth(in.uv.xy));
    var a = clamp(sig_dist + 0.5, 0.0, 1.0);
    return vec4<f32>(in.color.rgb, a);
}