use bevy::{
    prelude::{Assets, Commands, HandleUntyped, Plugin, Res},
    reflect::TypeUuid,
    render::{render_phase::DrawFunctions, render_resource::Shader, RenderApp, RenderStage},
    window::Windows,
};

use crate::{
    render::{
        ui_pass::TransparentUI,
        unified::pipeline::{DrawUI, QuadMeta, UnifiedPipeline},
    },
    WindowSize,
};

use self::pipeline::ImageBindGroups;

pub mod pipeline;

pub const UNIFIED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub struct UnifiedRenderPlugin;

impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let unified_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(UNIFIED_SHADER_HANDLE, unified_shader);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<ImageBindGroups>()
            .init_resource::<UnifiedPipeline>()
            .init_resource::<QuadMeta>()
            .add_system_to_stage(RenderStage::Extract, extract_baseline)
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_quads)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_quads);

        let draw_quad = DrawUI::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_quad);
    }
}

pub struct Dpi(f32);

pub fn extract_baseline(
    mut commands: Commands,
    windows: Res<Windows>,
    window_size: Res<WindowSize>,
) {
    let dpi = if let Some(window) = windows.get_primary() {
        window.scale_factor() as f32
    } else {
        1.0
    };

    commands.insert_resource(*window_size);
    commands.insert_resource(Dpi(dpi));
}
