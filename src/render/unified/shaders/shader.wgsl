#import kayak_ui::bindings view

#import kayak_ui::vertex_output VertexOutput

@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_color: vec4<f32>,
    @location(2) vertex_uv: vec4<f32>,
    @location(3) vertex_pos_size: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex_color;
    out.pos = (vertex_position.xy - vertex_pos_size.xy);
    out.position = view.view_proj * vec4<f32>(vertex_position, 1.0);
    out.pixel_position = out.position.xy;
    out.uv = vertex_uv.xyz;
    out.size = vertex_pos_size.zw;
    out.border_radius = vertex_uv.w;
    return out;
}

#import kayak_ui::sample_quad sample_quad

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return sample_quad(in);
}
