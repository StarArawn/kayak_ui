use bevy::{
    prelude::{
        AddAsset, Assets, Commands, HandleUntyped, IntoSystemAppConfig, IntoSystemConfig, Plugin,
        Query, Res, ResMut, Resource, With,
    },
    reflect::TypeUuid,
    render::{
        render_phase::AddRenderCommand,
        render_resource::{Shader, SpecializedRenderPipelines},
        renderer::{RenderDevice, RenderQueue},
        Extract, ExtractSchedule, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, Window},
};
use bevy_svg::prelude::Svg;

use crate::{
    render::{
        ui_pass::TransparentUI,
        unified::pipeline::{DrawUI, QuadMeta, UnifiedPipeline},
    },
    WindowSize,
};

use self::pipeline::{
    queue_quad_types, queue_ui_view_bind_groups, DrawUITransparent, ExtractedQuads,
    ImageBindGroups, PreviousClip, QuadTypeOffsets, PreviousIndex,
};

use super::{svg::RenderSvgs, ui_pass::TransparentOpacityUI};

pub mod pipeline;
pub mod text;

pub const UNIFIED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub const UNIFIED_BINDINGS_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 13885898746900949245);

pub const SAMPLE_QUAD_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5975018398368429820);

pub const VERTEX_OUTPUT_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 8828896277688845893);

pub struct UnifiedRenderPlugin;
impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<Svg>().add_plugin(text::TextRendererPlugin);

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let bindings_include = Shader::from_wgsl(include_str!("shaders/bindings.wgsl"));
        shaders.set_untracked(UNIFIED_BINDINGS_HANDLE, bindings_include);
        let sample_quad_include = Shader::from_wgsl(include_str!("shaders/sample_quad.wgsl"));
        shaders.set_untracked(SAMPLE_QUAD_HANDLE, sample_quad_include);
        let vertex_output_include = Shader::from_wgsl(include_str!("shaders/vertex_output.wgsl"));
        shaders.set_untracked(VERTEX_OUTPUT_HANDLE, vertex_output_include);
        let unified_shader = Shader::from_wgsl(include_str!("shaders/shader.wgsl"));
        shaders.set_untracked(UNIFIED_SHADER_HANDLE, unified_shader);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<QuadTypeOffsets>()
            .init_resource::<ExtractedQuads>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<UnifiedPipeline>()
            .init_resource::<SpecializedRenderPipelines<UnifiedPipeline>>()
            .init_resource::<QuadMeta>()
            .init_resource::<RenderSvgs>()
            .init_resource::<PreviousClip>()
            .init_resource::<PreviousIndex>()
            .add_system(super::svg::extract_svg_asset.in_schedule(ExtractSchedule))
            .add_system(extract_baseline.in_schedule(ExtractSchedule))
            .add_system(queue_quad_types.in_set(RenderSet::Queue))
            .add_system(
                pipeline::queue_quads
                    .in_set(RenderSet::Queue)
                    .after(queue_quad_types),
            )
            .add_system(
                queue_ui_view_bind_groups
                    .in_set(RenderSet::Queue)
                    .after(pipeline::queue_quads),
            )
            .add_system(queue_vertices.in_set(RenderSet::QueueFlush));

        render_app.add_render_command::<TransparentUI, DrawUI>();
        render_app.add_render_command::<TransparentOpacityUI, DrawUITransparent>();
    }
}

#[derive(Resource)]
pub struct Dpi(f32);

pub fn extract_baseline(
    mut commands: Commands,
    windows: Extract<Query<&Window, With<PrimaryWindow>>>,
    window_size: Extract<Res<WindowSize>>,
) {
    let dpi = if let Ok(window) = windows.get_single() {
        window.scale_factor() as f32
    } else {
        1.0
    };

    commands.insert_resource(**window_size);
    commands.insert_resource(Dpi(dpi));
}

pub fn queue_vertices(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut quad_meta: ResMut<QuadMeta>,
) {
    quad_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}
