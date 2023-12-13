#define_import_path kayak_ui::bindings
#import bevy_render::view::View

#import bevy_render::globals::Globals

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
