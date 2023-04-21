use std::ops::Range;

use bevy::ecs::prelude::*;
use bevy::prelude::{Color, Image};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{BatchedPhaseItem, DrawFunctionId, PhaseItem};
use bevy::render::render_resource::{CachedRenderPipelineId, RenderPassColorAttachment};
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_phase::{DrawFunctions, RenderPhase},
    render_resource::{LoadOp, Operations, RenderPassDescriptor},
    renderer::RenderContext,
    view::{ExtractedView, ViewTarget},
};
use bevy::utils::FloatOrd;

use crate::CameraUIKayak;

use super::extract::DefaultCameraView;
use super::opacity_layer::{OpacityLayerManager, MAX_OPACITY_LAYERS};
use super::unified::pipeline::UIQuadType;

pub struct TransparentUI {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub quad_type: UIQuadType,
    pub rect: bevy::math::Rect,
    pub type_index: u32,
    pub batch_range: Option<Range<u32>>,
}

impl PhaseItem for TransparentUI {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

impl BatchedPhaseItem for TransparentUI {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}

pub struct TransparentOpacityUI {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub quad_type: UIQuadType,
    pub rect: bevy::math::Rect,
    pub type_index: u32,
    pub batch_range: Option<Range<u32>>,
    pub opacity_layer: u32,
}

impl PhaseItem for TransparentOpacityUI {
    type SortKey = FloatOrd;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        self.sort_key
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }

    fn entity(&self) -> Entity {
        self.entity
    }
}

impl BatchedPhaseItem for TransparentOpacityUI {
    fn batch_range(&self) -> &Option<Range<u32>> {
        &self.batch_range
    }

    fn batch_range_mut(&mut self) -> &mut Option<Range<u32>> {
        &mut self.batch_range
    }
}

pub struct MainPassUINode {
    query: QueryState<
        (
            &'static RenderPhase<TransparentUI>,
            &'static RenderPhase<TransparentOpacityUI>,
            &'static ViewTarget,
            &'static CameraUIKayak,
        ),
        With<ExtractedView>,
    >,
    default_camera_view_query: QueryState<&'static DefaultCameraView>,
}

impl MainPassUINode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
            default_camera_view_query: world.query(),
        }
    }
}

impl Node for MainPassUINode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(MainPassUINode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
        self.default_camera_view_query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let input_view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        // adapted from bevy itself;
        // see: <https://github.com/bevyengine/bevy/commit/09a3d8abe062984479bf0e99fcc1508bb722baf6>
        let (transparent_phase, transparent_opacity_phase, target, _camera_ui) =
            match self.query.get_manual(world, input_view_entity) {
                Ok(it) => it,
                _ => return Ok(()),
            };

        let view_entity = if let Ok(default_view) = self
            .default_camera_view_query
            .get_manual(world, input_view_entity)
        {
            default_view.0
        } else {
            input_view_entity
        };

        // Opacity passes first..
        {
            let opacity_layer_manager = world.get_resource::<OpacityLayerManager>().unwrap();
            if let Some(opacity_layer_manager) =
                opacity_layer_manager.camera_layers.get(&input_view_entity)
            {
                let draw_functions = world
                    .get_resource::<DrawFunctions<TransparentOpacityUI>>()
                    .unwrap();
                let mut draw_functions = draw_functions.write();

                for layer_id in 1..MAX_OPACITY_LAYERS {
                    let layer_draw_calls = transparent_opacity_phase
                        .items
                        .iter()
                        .filter(|item| item.opacity_layer == layer_id)
                        .collect::<Vec<_>>();

                    if layer_draw_calls.is_empty() {
                        continue;
                    }

                    // Start new render pass.
                    let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();
                    let image_handle = opacity_layer_manager.get_image_handle(layer_id);
                    let gpu_image = gpu_images.get(&image_handle).unwrap();
                    // bevy::prelude::info!("Attaching opacity layer with index: {} with view: {:?}", layer_id, gpu_image.texture_view);
                    let pass_descriptor = RenderPassDescriptor {
                        label: Some("opacity_ui_layer_pass"),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &gpu_image.texture_view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::rgba(0.0, 0.0, 0.0, 0.0).into()),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    };

                    let mut tracked_pass =
                        render_context.begin_tracked_render_pass(pass_descriptor);

                    for item in layer_draw_calls {
                        let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
                        draw_function.draw(world, &mut tracked_pass, view_entity, item);
                    }
                }
            }
        }

        // Regular pass
        {
            let pass_descriptor = RenderPassDescriptor {
                label: Some("main_transparent_pass_UI"),
                color_attachments: &[Some(target.get_unsampled_color_attachment(Operations {
                    load: LoadOp::Load,
                    store: true,
                }))],
                depth_stencil_attachment: None,
            };

            let draw_functions = world
                .get_resource::<DrawFunctions<TransparentUI>>()
                .unwrap();

            let mut tracked_pass = render_context.begin_tracked_render_pass(pass_descriptor);
            let mut draw_functions = draw_functions.write();
            for item in transparent_phase.items.iter() {
                let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
                draw_function.draw(world, &mut tracked_pass, view_entity, item);
            }
        }

        Ok(())
    }
}
