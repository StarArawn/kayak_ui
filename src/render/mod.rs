use bevy::{
    prelude::{App, Camera, Commands, Entity, Plugin, Query, With},
    render::{
        render_graph::{RenderGraph, RunGraphOnViewNode, SlotInfo, SlotType},
        render_phase::{DrawFunctions, RenderPhase},
        Extract, RenderApp, RenderStage,
    },
};

use crate::{
    render::{ui_pass::MainPassUINode, unified::UnifiedRenderPlugin},
    CameraUIKayak,
};

use self::{extract::BevyKayakUIExtractPlugin, ui_pass::TransparentUI};

mod extract;
pub(crate) mod font;
pub(crate) mod image;
pub(crate) mod nine_patch;
pub(crate) mod quad;
pub(crate) mod texture_atlas;
mod ui_pass;
pub mod unified;

pub mod draw_ui_graph {
    pub const NAME: &str = "kayak_draw_ui";
    pub mod input {
        pub const VIEW_ENTITY: &str = "kayak_view_entity";
    }
    pub mod node {
        pub const MAIN_PASS: &str = "kayak_ui_pass";
    }
}

/// The default Kayak UI rendering plugin.
/// Use this to render the UI.
/// Or you can write your own renderer.
pub struct BevyKayakUIRenderPlugin;

impl Plugin for BevyKayakUIRenderPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<DrawFunctions<TransparentUI>>()
            .add_system_to_stage(RenderStage::Extract, extract_core_pipeline_camera_phases);
        // .add_system_to_stage(RenderStage::PhaseSort, sort_phase_system::<TransparentUI>);

        // let pass_node_ui = MainPassUINode::new(&mut render_app.world);
        // let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        // let mut draw_ui_graph = RenderGraph::default();
        // draw_ui_graph.add_node(draw_ui_graph::node::MAIN_PASS, pass_node_ui);
        // let input_node_id = draw_ui_graph.set_input(vec![SlotInfo::new(
        //     draw_ui_graph::input::VIEW_ENTITY,
        //     SlotType::Entity,
        // )]);
        // draw_ui_graph
        //     .add_slot_edge(
        //         input_node_id,
        //         draw_ui_graph::input::VIEW_ENTITY,
        //         draw_ui_graph::node::MAIN_PASS,
        //         MainPassUINode::IN_VIEW,
        //     )
        //     .unwrap();
        // graph.add_sub_graph(draw_ui_graph::NAME, draw_ui_graph);

        // // graph.add_node_edge(MAIN_PASS, draw_ui_graph::NAME).unwrap();

        // Render graph
        let ui_graph_2d = get_ui_graph(render_app);
        let ui_graph_3d = get_ui_graph(render_app);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        if let Some(graph_2d) = graph.get_sub_graph_mut(bevy::core_pipeline::core_2d::graph::NAME) {
            graph_2d.add_sub_graph(draw_ui_graph::NAME, ui_graph_2d);
            graph_2d.add_node(
                draw_ui_graph::node::MAIN_PASS,
                RunGraphOnViewNode::new(draw_ui_graph::NAME),
            );
            graph_2d
                .add_node_edge(
                    bevy::core_pipeline::core_2d::graph::node::MAIN_PASS,
                    draw_ui_graph::node::MAIN_PASS,
                )
                .unwrap();
            graph_2d
                .add_slot_edge(
                    graph_2d.input_node().unwrap().id,
                    bevy::core_pipeline::core_2d::graph::input::VIEW_ENTITY,
                    draw_ui_graph::node::MAIN_PASS,
                    RunGraphOnViewNode::IN_VIEW,
                )
                .unwrap();
            graph_2d
                .add_node_edge(
                    bevy::core_pipeline::core_2d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                    draw_ui_graph::node::MAIN_PASS,
                )
                .unwrap();
            graph_2d
                .add_node_edge(
                    draw_ui_graph::node::MAIN_PASS,
                    bevy::core_pipeline::core_2d::graph::node::UPSCALING,
                )
                .unwrap();
        }

        if let Some(graph_3d) = graph.get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME) {
            graph_3d.add_sub_graph(draw_ui_graph::NAME, ui_graph_3d);
            graph_3d.add_node(
                draw_ui_graph::node::MAIN_PASS,
                RunGraphOnViewNode::new(draw_ui_graph::NAME),
            );
            graph_3d
                .add_node_edge(
                    bevy::core_pipeline::core_3d::graph::node::MAIN_PASS,
                    draw_ui_graph::node::MAIN_PASS,
                )
                .unwrap();
            graph_3d
                .add_node_edge(
                    bevy::core_pipeline::core_3d::graph::node::END_MAIN_PASS_POST_PROCESSING,
                    draw_ui_graph::node::MAIN_PASS,
                )
                .unwrap();
            graph_3d
                .add_node_edge(
                    draw_ui_graph::node::MAIN_PASS,
                    bevy::core_pipeline::core_3d::graph::node::UPSCALING,
                )
                .unwrap();
            graph_3d
                .add_slot_edge(
                    graph_3d.input_node().unwrap().id,
                    bevy::core_pipeline::core_3d::graph::input::VIEW_ENTITY,
                    draw_ui_graph::node::MAIN_PASS,
                    RunGraphOnViewNode::IN_VIEW,
                )
                .unwrap();
        }

        app.add_plugin(font::TextRendererPlugin)
            .add_plugin(UnifiedRenderPlugin)
            .add_plugin(BevyKayakUIExtractPlugin);
    }
}

fn get_ui_graph(render_app: &mut App) -> RenderGraph {
    let ui_pass_node = MainPassUINode::new(&mut render_app.world);
    let mut ui_graph = RenderGraph::default();
    ui_graph.add_node(draw_ui_graph::node::MAIN_PASS, ui_pass_node);
    let input_node_id = ui_graph.set_input(vec![SlotInfo::new(
        draw_ui_graph::input::VIEW_ENTITY,
        SlotType::Entity,
    )]);
    ui_graph
        .add_slot_edge(
            input_node_id,
            draw_ui_graph::input::VIEW_ENTITY,
            draw_ui_graph::node::MAIN_PASS,
            MainPassUINode::IN_VIEW,
        )
        .unwrap();
    ui_graph
}

pub fn extract_core_pipeline_camera_phases(
    mut commands: Commands,
    active_cameras: Extract<Query<(Entity, &Camera), With<CameraUIKayak>>>,
) {
    for (entity, camera) in &active_cameras {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<TransparentUI>::default());
        }
    }
}
