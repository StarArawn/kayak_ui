use bevy::{ecs::world::World, render::camera::ActiveCamera};
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotValue},
    renderer::RenderContext,
};

use crate::CameraUiKayak;

pub struct UIPassDriverNode;

impl Node for UIPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if let Some(camera_ui) = world.resource::<ActiveCamera<CameraUiKayak>>().get() {
            graph.run_sub_graph(
                super::draw_ui_graph::NAME,
                vec![SlotValue::Entity(camera_ui)],
            )?;
        }

        Ok(())
    }
}
