#define_import_path kayak_ui::bindings

#import bevy_render::globals::Globals

struct View {
    view_proj: mat4x4<f32>,
    unjittered_view_proj: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    inverse_view: mat4x4<f32>,
    projection: mat4x4<f32>,
    inverse_projection: mat4x4<f32>,
    world_position: vec3<f32>,
    exposure: f32,
    // viewport(x_origin, y_origin, width, height)
    viewport: vec4<f32>,
    frustum: array<vec4<f32>, 6>,
    // color_grading: ColorGrading,
    mip_bias: f32,
    render_layers: u32,
};

@group(0) @binding(0)
var<uniform> view: View;

@group(0) @binding(1)
var<uniform> globals: Globals;

struct QuadType {
    t: i32,
    _padding_a: i32,
    _padding_b: i32,
    _padding_c: i32,
};

@group(2) @binding(0)
var<uniform> quad_type: QuadType;

@group(1) @binding(0)
var image_texture: texture_2d<f32>;
@group(1) @binding(1)
var image_sampler: sampler;

@group(1) @binding(2)
var font_texture: texture_2d_array<f32>;
@group(1) @binding(3)
var font_sampler: sampler;
