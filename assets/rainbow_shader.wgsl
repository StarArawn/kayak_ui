#import kayak_ui::bindings
#import kayak_ui::sample_quad
#import kayak_ui::vertex_output

fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var output_color = sample_quad(in);
    let hsv = vec3(abs(sin(globals.time)), 1.0, 1.0);
    return vec4(hsv2rgb(hsv), output_color.a);
}
