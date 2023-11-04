use bevy::{
    asset::embedded_asset,
    prelude::{Commands, IntoSystemConfigs, Plugin, Query, Res, ResMut, Resource, With},
    render::{
        render_phase::AddRenderCommand,
        render_resource::{SpecializedRenderPipelines},
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

pub struct UnifiedRenderPlugin;
impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "svg")]
        app.add_asset::<Svg>().add_plugins(text::TextRendererPlugin);

        embedded_asset!(app, "shaders/bindings.wgsl");
        embedded_asset!(app, "shaders/sample_quad.wgsl");
        embedded_asset!(app, "shaders/vertex_output.wgsl");
        embedded_asset!(app, "shaders/shader.wgsl");

        let render_app = app.sub_app_mut(RenderApp);
        #[cfg(feature = "svg")]
        render_app
            .init_resource::<RenderSvgs>()
            .add_systems(ExtractSchedule, super::svg::extract_svg_asset);
        render_app
            .init_resource::<QuadTypeOffsets>()
            .init_resource::<ExtractedQuads>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<QuadMeta>()
            .init_resource::<PreviousClip>()
            .init_resource::<PreviousIndex>()
            .add_systems(ExtractSchedule, extract_baseline)
            .add_systems(
                Render,
                (
                    queue_quad_types,
                    (pipeline::queue_quads, queue_ui_view_bind_groups),
                )
                    .chain()
                    .in_set(RenderSet::Queue),
            )
            .add_systems(Render, queue_vertices.in_set(RenderSet::QueueFlush));

        render_app.add_render_command::<TransparentUI, DrawUI>();
        render_app.add_render_command::<TransparentOpacityUI, DrawUITransparent>();
    }

    fn finish(&self, app: &mut bevy::prelude::App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<UnifiedPipeline>()
            .init_resource::<SpecializedRenderPipelines<UnifiedPipeline>>();
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
