use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssets,
        render_graph::{RenderGraph, RunGraphOnViewNode},
        render_phase::{batch_phase_system, sort_phase_system, DrawFunctions, RenderPhase},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, Window, WindowRef},
};

use crate::{
    render::{ui_pass::MainPassUINode, unified::UnifiedRenderPlugin},
    CameraUIKayak,
};

use self::{
    extract::BevyKayakUIExtractPlugin,
    opacity_layer::OpacityLayerManager,
    ui_pass::{TransparentOpacityUI, TransparentUI},
};

mod extract;
pub(crate) mod font;
pub(crate) mod image;
pub mod material;
pub(crate) mod nine_patch;
mod opacity_layer;
pub(crate) mod quad;
pub(crate) mod svg;
pub(crate) mod texture_atlas;
mod ui_pass;
pub mod unified;

pub use opacity_layer::MAX_OPACITY_LAYERS;

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
        app.init_resource::<OpacityLayerManager>()
            .add_systems(Update, update_opacity_layer_cameras);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<DrawFunctions<TransparentUI>>()
            .init_resource::<DrawFunctions<TransparentOpacityUI>>()
            .add_systems(ExtractSchedule, extract_core_pipeline_camera_phases)
            .add_systems(
                Render,
                prepare_opacity_layers
                    .in_set(RenderSet::Queue)
                    .before(unified::pipeline::queue_quads),
            )
            .add_systems(
                Render,
                (
                    batch_phase_system::<TransparentUI>.after(sort_phase_system::<TransparentUI>),
                    batch_phase_system::<TransparentOpacityUI>
                        .after(sort_phase_system::<TransparentOpacityUI>),
                )
                    .in_set(RenderSet::PhaseSort),
            );

        // let pass_node_ui = MainPassUINode::new(&mut render_app.world);
        // let mut graph = render_app.world.get_resource_mut::<RenderGraph>().unwrap();

        // let mut draw_ui_graph = RenderGraph::default();
        // draw_ui_graph.add_node(draw_ui_graph::node::MAIN_PASS, pass_node_ui);
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
            graph_2d.add_node_edge(
                bevy::core_pipeline::core_2d::graph::node::MAIN_PASS,
                draw_ui_graph::node::MAIN_PASS,
            );
            graph_2d.add_node_edge(
                bevy::core_pipeline::core_2d::graph::node::TONEMAPPING,
                draw_ui_graph::node::MAIN_PASS,
            );
            graph_2d.add_node_edge(
                draw_ui_graph::node::MAIN_PASS,
                bevy::core_pipeline::core_2d::graph::node::UPSCALING,
            );
        }

        if let Some(graph_3d) = graph.get_sub_graph_mut(bevy::core_pipeline::core_3d::graph::NAME) {
            graph_3d.add_sub_graph(draw_ui_graph::NAME, ui_graph_3d);
            graph_3d.add_node(
                draw_ui_graph::node::MAIN_PASS,
                RunGraphOnViewNode::new(draw_ui_graph::NAME),
            );
            graph_3d.add_node_edge(
                bevy::core_pipeline::core_3d::graph::node::END_MAIN_PASS,
                draw_ui_graph::node::MAIN_PASS,
            );
            graph_3d.add_node_edge(
                bevy::core_pipeline::core_3d::graph::node::TONEMAPPING,
                draw_ui_graph::node::MAIN_PASS,
            );
            graph_3d.add_node_edge(
                draw_ui_graph::node::MAIN_PASS,
                bevy::core_pipeline::core_3d::graph::node::UPSCALING,
            );
        }

        app.add_plugins((
            font::TextRendererPlugin,
            UnifiedRenderPlugin,
            BevyKayakUIExtractPlugin,
        ));
    }
}

fn get_ui_graph(render_app: &mut App) -> RenderGraph {
    let ui_pass_node = MainPassUINode::new(&mut render_app.world);
    let mut ui_graph = RenderGraph::default();
    ui_graph.add_node(draw_ui_graph::node::MAIN_PASS, ui_pass_node);
    ui_graph
}

pub fn update_opacity_layer_cameras(
    windows: Query<&Window>,
    cameras: Query<(Entity, &Camera), With<CameraUIKayak>>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
    mut opacity_layers: ResMut<OpacityLayerManager>,
    mut images: ResMut<Assets<Image>>,
) {
    for (camera_entity, camera) in cameras.iter() {
        if let RenderTarget::Window(window_ref) = &camera.target {
            let window_entity = match window_ref {
                WindowRef::Entity(entity) => *entity,
                WindowRef::Primary => primary_window.get_single().unwrap(),
            };
            if let Ok(camera_window) = windows.get(window_entity) {
                opacity_layers.add_or_update(&camera_entity, camera_window, &mut images);
            }
        }
    }
}

pub fn extract_core_pipeline_camera_phases(
    mut commands: Commands,
    active_cameras: Extract<Query<(Entity, &Camera), With<CameraUIKayak>>>,
    opacity_layers: Extract<Res<OpacityLayerManager>>,
) {
    for (entity, camera) in &active_cameras {
        if camera.is_active {
            commands
                .get_or_spawn(entity)
                .insert(RenderPhase::<TransparentOpacityUI>::default())
                .insert(RenderPhase::<TransparentUI>::default());
        }
    }

    let opacity_layers = opacity_layers.clone();
    commands.insert_resource(opacity_layers);
}

fn prepare_opacity_layers(
    mut opacity_layers: ResMut<OpacityLayerManager>,
    gpu_images: Res<RenderAssets<Image>>,
) {
    for (_, layer) in opacity_layers.camera_layers.iter_mut() {
        layer.set_texture_views(&gpu_images);
    }
}
