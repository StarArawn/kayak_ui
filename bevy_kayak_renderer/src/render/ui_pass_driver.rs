use bevy::ecs::world::World;
use bevy::render::{
    camera::ExtractedCameraNames,
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotValue},
    renderer::RenderContext,
};

use crate::UICameraBundle;

pub struct UIPassDriverNode;

impl Node for UIPassDriverNode {
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        _render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let extracted_cameras = world.get_resource::<ExtractedCameraNames>().unwrap();
        if let Some(camera_ui) = extracted_cameras.entities.get(UICameraBundle::UI_CAMERA) {
            graph.run_sub_graph(
                super::draw_ui_graph::NAME,
                vec![SlotValue::Entity(*camera_ui)],
            )?;
        }

        Ok(())
    }
}
