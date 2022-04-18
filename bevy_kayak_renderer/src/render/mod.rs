use bevy::{
    core_pipeline::node::MAIN_PASS_DRIVER,
    prelude::{Commands, Plugin, Res},
    render::{
        camera::ActiveCamera,
        render_graph::{EmptyNode, RenderGraph, SlotInfo, SlotType},
        render_phase::{DrawFunctions, RenderPhase},
        RenderApp, RenderStage,
    },
};

use crate::{
    render::{
        ui_pass::MainPassUINode, ui_pass_driver::UIPassDriverNode, unified::UnifiedRenderPlugin,
    },
    CameraUiKayak,
};

use self::ui_pass::TransparentUI;

mod ui_pass;
mod ui_pass_driver;
pub mod unified;

pub mod node {
    pub const UI_PASS_DEPENDENCIES: &str = "kayak_ui_pass_dependencies";
    pub const UI_PASS_DRIVER: &str = "kayak_ui_pass_driver";
}

pub mod draw_ui_graph {
    pub const NAME: &str = "kayak_draw_ui";
    pub mod input {
        pub const VIEW_ENTITY: &str = "kayak_view_entity";
    }
    pub mod node {
        pub const MAIN_PASS: &str = "kayak_ui_pass";
    }
}

pub struct BevyKayakUIRenderPlugin;

impl Plugin for BevyKayakUIRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<DrawFunctions<TransparentUI>>()
            .add_system_to_stage(RenderStage::Extract, extract_core_pipeline_camera_phases);
        // .add_system_to_stage(RenderStage::PhaseSort, sort_phase_system::<TransparentUI>);

        let pass_node_ui = MainPassUINode::new(&mut render_app.world);
        let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        let mut draw_ui_graph = RenderGraph::default();
        draw_ui_graph.add_node(draw_ui_graph::node::MAIN_PASS, pass_node_ui);
        let input_node_id = draw_ui_graph.set_input(vec![SlotInfo::new(
            draw_ui_graph::input::VIEW_ENTITY,
            SlotType::Entity,
        )]);
        draw_ui_graph
            .add_slot_edge(
                input_node_id,
                draw_ui_graph::input::VIEW_ENTITY,
                draw_ui_graph::node::MAIN_PASS,
                MainPassUINode::IN_VIEW,
            )
            .unwrap();
        graph.add_sub_graph(draw_ui_graph::NAME, draw_ui_graph);

        graph.add_node(node::UI_PASS_DEPENDENCIES, EmptyNode);
        graph.add_node(node::UI_PASS_DRIVER, UIPassDriverNode);
        graph
            .add_node_edge(node::UI_PASS_DEPENDENCIES, node::UI_PASS_DRIVER)
            .unwrap();
        graph
            .add_node_edge(MAIN_PASS_DRIVER, node::UI_PASS_DRIVER)
            .unwrap();

        app.add_plugin(UnifiedRenderPlugin);
    }
}

pub fn extract_core_pipeline_camera_phases(
    mut commands: Commands,
    active_camera: Res<ActiveCamera<CameraUiKayak>>,
) {
    if let Some(entity) = active_camera.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<TransparentUI>::default());
    }
}
