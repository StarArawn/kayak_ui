[[block]]
struct View {
    view_proj: mat4x4<f32>;
    world_position: vec3<f32>;
};
[[group(0), binding(0)]]
var<uniform> view: View;

[[block]]
struct QuadType {
    t: i32;
};
[[group(2), binding(0)]]
var<uniform> quad_type: QuadType;

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

[[group(3), binding(0)]]
var image_texture: texture_2d<f32>;
[[group(3), binding(1)]]
var image_sampler: sampler;

let RADIUS: f32 = 0.1;

fn sd_box_rounded(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32,
) -> f32 {
    var inner_size: vec2<f32> = size - radius * 2.0;
    var top_left: vec2<f32> = position + radius;
    var bottom_right: vec2<f32> = top_left + inner_size;

    var top_left_distance: vec2<f32> = top_left - frag_coord;
    var bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    var dist = max(max(top_left_distance, bottom_right_distance), vec2<f32>(0.0));

    return length(dist);
}

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    if (quad_type.t == 0) {
        var dist = sd_box_rounded(
            in.position.xy,
            in.pos,
            in.size,
            in.border_radius,
        );
        dist = 1.0 - smoothStep(
            max(in.border_radius - 0.5, 0.0),
            in.border_radius + 0.5,
            dist);

        return vec4<f32>(in.color.rgb, dist);
    }
    if (quad_type.t == 1) {
        var px_range = 2.5;
        var tex_dimensions = textureDimensions(font_texture);
        var msdf_unit = vec2<f32>(px_range, px_range) / vec2<f32>(f32(tex_dimensions.x), f32(tex_dimensions.y));
        var x = textureSample(font_texture, font_sampler, vec2<f32>(in.uv.x, 1.0 - in.uv.y), i32(in.uv.z)); 
        var v = max(min(x.r, x.g), min(max(x.r, x.g), x.b));
        var sig_dist = (v - 0.5) * dot(msdf_unit, 0.5 / fwidth(in.uv.xy));
        var a = clamp(sig_dist + 0.5, 0.0, 1.0);

        return vec4<f32>(in.color.rgb, a);
    }
    if (quad_type.t == 2) {
        var color = textureSample(image_texture, image_sampler, vec2<f32>(in.uv.x, in.uv.y));
        return vec4<f32>(color.rgb * in.color.rgb, color.a * in.color.a);
    }
    return in.color;
}