use bevy::{
    prelude::{Assets, HandleUntyped, Plugin},
    reflect::TypeUuid,
    render2::{render_phase::DrawFunctions, render_resource::Shader, RenderApp, RenderStage},
};

use crate::render::{
    quad::pipeline::{DrawQuad, QuadMeta, QuadPipeline},
    ui_pass::TransparentUI,
};

mod pipeline;

pub const QUAD_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 7604018236855288450);

#[derive(Default)]
pub struct QuadRendererPlugin;

impl Plugin for QuadRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        let quad_shader = Shader::from_wgsl(include_str!("shader.wgsl"));
        shaders.set_untracked(QUAD_SHADER_HANDLE, quad_shader);

        let render_app = app.sub_app(RenderApp);
        render_app
            .init_resource::<QuadPipeline>()
            .init_resource::<QuadMeta>()
            .add_system_to_stage(RenderStage::Extract, pipeline::extract_quads)
            .add_system_to_stage(RenderStage::Prepare, pipeline::prepare_quads)
            .add_system_to_stage(RenderStage::Queue, pipeline::queue_quads);

        let draw_quad = DrawQuad::new(&mut render_app.world);

        render_app
            .world
            .get_resource::<DrawFunctions<TransparentUI>>()
            .unwrap()
            .write()
            .add(draw_quad);
    }
}
