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
};

[[stage(vertex)]]
fn vertex(
    [[location(0)]] vertex_position: vec3<f32>,
    [[location(1)]] vertex_color: vec4<f32>,
    [[location(2)]] vertex_uv: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex_color;
    out.uv = vertex_uv;
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    return out;
}

[[group(1), binding(0)]]
var sprite_texture: texture_2d_array<f32>;
[[group(1), binding(1)]]
var sprite_sampler: sampler;

let RADIUS: f32 = 0.5;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var x = textureSample(sprite_texture, sprite_sampler, in.uv.xy, i32(in.uv.z)); 
    var v = max(min(x.r, x.g), min(max(x.r, x.g), x.b));
    var c = v; //remap(v);

    var v2 = c / fwidth( c );
    var a = v2 + RADIUS; //clamp( v2 + RADIUS, 0.0, 1.0 );

    return vec4<f32>(in.color.rgb, a);
}