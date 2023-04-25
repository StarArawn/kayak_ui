#define_import_path kayak_ui::bindings

// struct View {
//     view_proj: mat4x4<f32>,
//     world_position: vec3<f32>,
// };
// @group(0) @binding(0)
// var<uniform> view: View;

#import bevy_render::view
#import bevy_render::globals

@group(0) @binding(0)
var<uniform> view: View;

@group(0) @binding(1)
var<uniform> globals: Globals;

struct QuadType {
    t: i32,
    _padding_1: i32,
    _padding_2: i32,
    _padding_3: i32,
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