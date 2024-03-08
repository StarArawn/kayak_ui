#[cfg(feature = "svg")]
use bevy::prelude::AssetApp;
use bevy::{
    asset::{load_internal_asset, Handle},
    prelude::{Commands, IntoSystemConfigs, Plugin, Query, Res, ResMut, Resource, With},
    render::{
        render_phase::AddRenderCommand,
        render_resource::{Shader, SpecializedRenderPipelines},
        renderer::{RenderDevice, RenderQueue},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, Window},
};
#[cfg(feature = "svg")]
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
    ImageBindGroups, PreviousClip, PreviousIndex, QuadTypeOffsets,
};

#[cfg(feature = "svg")]
use super::svg::RenderSvgs;
use super::ui_pass::TransparentOpacityUI;

pub mod pipeline;
pub mod text;

pub const UNIFIED_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(7604018236855288450);

pub const UNIFIED_BINDINGS_HANDLE: Handle<Shader> = Handle::weak_from_u128(13885898746900949245);

pub const SAMPLE_QUAD_HANDLE: Handle<Shader> = Handle::weak_from_u128(5975018398368429820);

pub const VERTEX_OUTPUT_HANDLE: Handle<Shader> = Handle::weak_from_u128(8828896277688845893);

pub struct UnifiedRenderPlugin;
impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "svg")]
        app.init_asset::<Svg>();
        app.add_plugins(text::TextRendererPlugin);

        load_internal_asset!(
            app,
            UNIFIED_BINDINGS_HANDLE,
            "shaders/bindings.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            SAMPLE_QUAD_HANDLE,
            "shaders/sample_quad.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            VERTEX_OUTPUT_HANDLE,
            "shaders/vertex_output.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            UNIFIED_SHADER_HANDLE,
            "shaders/shader.wgsl",
            Shader::from_wgsl
        );
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<UnifiedPipeline>()
            .init_resource::<SpecializedRenderPipelines<UnifiedPipeline>>()
            .init_resource::<QuadTypeOffsets>()
            .init_resource::<ExtractedQuads>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<QuadMeta>();
        #[cfg(feature = "svg")]
        render_app.init_resource::<RenderSvgs>();
        render_app
            .init_resource::<PreviousClip>()
            .init_resource::<PreviousIndex>()
            .add_systems(
                ExtractSchedule,
                (
                    #[cfg(feature = "svg")]
                    super::svg::extract_svg_asset,
                    extract_baseline,
                ),
            )
            .add_systems(
                Render,
                (
                    queue_quad_types,
                    (pipeline::queue_quads, queue_ui_view_bind_groups),
                )
                    .chain()
                    .in_set(RenderSet::Queue),
            )
            .add_systems(Render, queue_vertices.in_set(RenderSet::QueueMeshes));

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
