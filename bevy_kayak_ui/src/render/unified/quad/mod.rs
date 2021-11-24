use bevy::{
    prelude::Plugin,
    render2::{RenderApp, RenderStage},
};

mod extract;

pub struct QuadRendererPlugin;

impl Plugin for QuadRendererPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app(RenderApp);
        render_app.add_system_to_stage(RenderStage::Extract, extract::extract_quads);
    }
}
