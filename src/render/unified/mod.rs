use bevy::{
    prelude::{
        AddAsset, Assets, Commands, HandleUntyped, IntoSystemAppConfig, IntoSystemConfig, Plugin,
        Query, Res, Resource, With,
    },
    reflect::TypeUuid,
    render::{
        render_phase::DrawFunctions,
        render_resource::{Shader, SpecializedRenderPipelines},
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

use self::pipeline::{DrawOpacityUI, ExtractedQuads, ImageBindGroups};

use super::{svg::RenderSvgs, ui_pass::TransparentOpacityUI};

pub mod pipeline;
pub mod text;

pub const UNIFIED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub struct UnifiedRenderPlugin;

impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<Svg>().add_plugin(text::TextRendererPlugin);

        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let unified_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(UNIFIED_SHADER_HANDLE, unified_shader);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ExtractedQuads>()
            .init_resource::<ImageBindGroups>()
            .init_resource::<UnifiedPipeline>()
            .init_resource::<SpecializedRenderPipelines<UnifiedPipeline>>()
            .init_resource::<QuadMeta>()
            .init_resource::<RenderSvgs>()
            .add_system(super::svg::extract_svg_asset.in_schedule(ExtractSchedule))
            .add_system(extract_baseline.in_schedule(ExtractSchedule))
            .add_system(pipeline::queue_quads.in_set(RenderSet::Queue));

        let draw_quad = DrawUI::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_quad);

        let draw_quad = DrawOpacityUI::new(&mut render_app.world);
        render_app
            .world
            .get_resource::<DrawFunctions<TransparentOpacityUI>>()
            .unwrap()
            .write()
            .add(draw_quad);
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
