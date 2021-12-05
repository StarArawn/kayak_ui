use bevy::{
    prelude::{Assets, HandleUntyped, Plugin},
    reflect::TypeUuid,
    render2::{render_phase::DrawFunctions, render_resource::Shader, RenderApp, RenderStage},
};

use crate::render::{
    ui_pass::TransparentUI,
    unified::pipeline::{DrawUI, QuadMeta, UnifiedPipeline},
};

use self::pipeline::ImageBindGroups;

pub mod font;
pub mod image;
mod pipeline;
mod quad;

pub const UNIFIED_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

pub struct UnifiedRenderPlugin;

impl Plugin for UnifiedRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let unified_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(UNIFIED_SHADER_HANDLE, unified_shader);

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<ImageBindGroups>()
            .init_resource::<UnifiedPipeline>()
            .init_resource::<QuadMeta>()
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_quads)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_quads);

        let draw_quad = DrawUI::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_quad);

        app.add_plugin(font::TextRendererPlugin)
            .add_plugin(quad::QuadRendererPlugin)
            .add_plugin(image::ImageRendererPlugin);
    }
}
