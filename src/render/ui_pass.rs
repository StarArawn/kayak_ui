use std::ops::Range;

use bevy::ecs::prelude::*;
use bevy::prelude::{Color, Image};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_phase::{
    CachedRenderPipelinePhaseItem, DrawFunctionId, DrawFunctions, PhaseItem, TrackedRenderPass,
};
use bevy::render::render_resource::{CachedRenderPipelineId, RenderPassColorAttachment};
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext},
    render_resource::{LoadOp, Operations, RenderPassDescriptor},
    renderer::RenderContext,
    view::{ExtractedView, ViewTarget},
};
use bevy::utils::nonmax::NonMaxU32;
use bevy::utils::FloatOrd;

use crate::CameraUIKayak;

use super::opacity_layer::{OpacityLayerManager, MAX_OPACITY_LAYERS};
use super::unified::pipeline::UIQuadType;

pub trait TransparentUIGeneric {
    fn get_entity(&self) -> Entity;
    fn get_quad_type(&self) -> UIQuadType;
    fn get_rect(&self) -> bevy::math::Rect;
    fn get_type_index(&self) -> u32;
}

#[derive(Debug)]
pub struct TransparentUI {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
    pub quad_type: UIQuadType,
    pub rect: bevy::math::Rect,
    pub type_index: u32,
    pub batch_range: Option<Range<u32>>,
    pub dynamic_offset: Option<NonMaxU32>,
}

impl TransparentUIGeneric for TransparentUI {
    fn get_entity(&self) -> Entity {
        self.entity
    }

    fn get_quad_type(&self) -> UIQuadType {
        self.quad_type
    }

    fn get_rect(&self) -> bevy::math::Rect {
        self.rect
    }

    fn get_type_index(&self) -> u32 {
        self.type_index
    }
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

    fn batch_range(&self) -> &Range<u32> {
        self.batch_range.as_ref().unwrap()
    }

    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        self.batch_range.as_mut().unwrap()
    }

    fn dynamic_offset(&self) -> Option<bevy::utils::nonmax::NonMaxU32> {
        self.dynamic_offset
    }

    fn dynamic_offset_mut(&mut self) -> &mut Option<bevy::utils::nonmax::NonMaxU32> {
        &mut self.dynamic_offset
    }
}

impl CachedRenderPipelinePhaseItem for TransparentUI {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

#[derive(Debug)]
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
    pub dynamic_offset: Option<NonMaxU32>,
}

impl TransparentUIGeneric for TransparentOpacityUI {
    fn get_entity(&self) -> Entity {
        self.entity
    }

    fn get_quad_type(&self) -> UIQuadType {
        self.quad_type
    }

    fn get_rect(&self) -> bevy::math::Rect {
        self.rect
    }

    fn get_type_index(&self) -> u32 {
        self.type_index
    }
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

    fn batch_range(&self) -> &Range<u32> {
        self.batch_range.as_ref().unwrap()
    }

    fn batch_range_mut(&mut self) -> &mut Range<u32> {
        self.batch_range.as_mut().unwrap()
    }

    fn dynamic_offset(&self) -> Option<bevy::utils::nonmax::NonMaxU32> {
        self.dynamic_offset
    }

    fn dynamic_offset_mut(&mut self) -> &mut Option<bevy::utils::nonmax::NonMaxU32> {
        &mut self.dynamic_offset
    }
}

impl CachedRenderPipelinePhaseItem for TransparentOpacityUI {
    #[inline]
    fn cached_pipeline(&self) -> CachedRenderPipelineId {
        self.pipeline
    }
}

pub struct MainPassUINode {
    query: QueryState<
        (
            &'static UIRenderPhase<TransparentUI>,
            &'static UIRenderPhase<TransparentOpacityUI>,
            &'static ViewTarget,
            &'static CameraUIKayak,
        ),
        With<ExtractedView>,
    >,
}

impl MainPassUINode {
    pub fn new(world: &mut World) -> Self {
        Self {
            query: world.query_filtered(),
        }
    }
}

impl Node for MainPassUINode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.view_entity();
        // adapted from bevy itself;
        // see: <https://github.com/bevyengine/bevy/commit/09a3d8abe062984479bf0e99fcc1508bb722baf6>
        let (transparent_phase, transparent_opacity_phase, target, _camera_ui) =
            match self.query.get_manual(world, view_entity) {
                Ok(it) => it,
                _ => return Ok(()),
            };

        // Opacity passes first..
        {
            let opacity_layer_manager = world.get_resource::<OpacityLayerManager>().unwrap();
            if let Some(opacity_layer_manager) =
                opacity_layer_manager.camera_layers.get(&view_entity)
            {
                let draw_functions = world.resource::<DrawFunctions<TransparentOpacityUI>>();
                let mut draw_functions = draw_functions.write();
                draw_functions.prepare(world);

                for layer_id in 1..MAX_OPACITY_LAYERS {
                    // Start new render pass.
                    let gpu_images = world.get_resource::<RenderAssets<Image>>().unwrap();
                    let image_handle = opacity_layer_manager.get_image_handle(layer_id);
                    let gpu_image = gpu_images.get(&image_handle).unwrap();
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

                    for item in transparent_opacity_phase
                        .items
                        .iter()
                        .filter(|i| i.opacity_layer == layer_id)
                    {
                        let draw_function = draw_functions.get_mut(item.draw_function()).unwrap();
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
            let mut tracked_pass = render_context.begin_tracked_render_pass(pass_descriptor);
            transparent_phase.render(&mut tracked_pass, world, view_entity);
        }

        Ok(())
    }
}

use std::slice::SliceIndex;

/// A collection of all rendering instructions, that will be executed by the GPU, for a
/// single render phase for a single view.
///
/// Each view (camera, or shadow-casting light, etc.) can have one or multiple render phases.
/// They are used to queue entities for rendering.
/// Multiple phases might be required due to different sorting/batching behaviors
/// (e.g. opaque: front to back, transparent: back to front) or because one phase depends on
/// the rendered texture of the previous phase (e.g. for screen-space reflections).
/// All [`PhaseItem`]s are then rendered using a single [`TrackedRenderPass`].
/// The render pass might be reused for multiple phases to reduce GPU overhead.
#[derive(Component, Debug)]
pub struct UIRenderPhase<I: PhaseItem> {
    pub items: Vec<I>,
}

impl<I: PhaseItem + std::fmt::Debug> Default for UIRenderPhase<I> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<I: PhaseItem + std::fmt::Debug> UIRenderPhase<I> {
    /// Adds a [`PhaseItem`] to this render phase.
    #[inline]
    pub fn add(&mut self, item: I) {
        self.items.push(item);
    }

    /// Sorts all of its [`PhaseItem`]s.
    pub fn sort(&mut self) {
        I::sort(&mut self.items);
    }

    /// An [`Iterator`] through the associated [`Entity`] for each [`PhaseItem`] in order.
    #[inline]
    pub fn iter_entities(&'_ self) -> impl Iterator<Item = Entity> + '_ {
        self.items.iter().map(|item| item.entity())
    }

    /// Renders all of its [`PhaseItem`]s using their corresponding draw functions.
    pub fn render<'w>(
        &self,
        render_pass: &mut TrackedRenderPass<'w>,
        world: &'w World,
        view: Entity,
    ) {
        self.render_range(render_pass, world, view, ..);
    }

    /// Renders all [`PhaseItem`]s in the provided `range` (based on their index in `self.items`) using their corresponding draw functions.
    pub fn render_range<'w>(
        &self,
        render_pass: &mut TrackedRenderPass<'w>,
        world: &'w World,
        view: Entity,
        range: impl SliceIndex<[I], Output = [I]>,
    ) {
        let items = self
            .items
            .get(range)
            .expect("`Range` provided to `render_range()` is out of bounds");

        let draw_functions = world.resource::<DrawFunctions<I>>();
        let mut draw_functions = draw_functions.write();
        draw_functions.prepare(world);

        let mut index = 0;
        while index < items.len() {
            let item = &items[index];
            let draw_function = draw_functions.get_mut(item.draw_function()).unwrap();
            draw_function.draw(world, render_pass, view, item);
            index += 1;
        }
    }
}

/// This system sorts the [`PhaseItem`]s of all [`RenderPhase`]s of this type.
pub fn sort_ui_phase_system<I: PhaseItem + std::fmt::Debug>(
    mut render_phases: Query<&mut UIRenderPhase<I>>,
) {
    for mut phase in &mut render_phases {
        phase.sort();
    }
}
