use bevy::core::FloatOrd;
use bevy::ecs::prelude::*;
use bevy::render::render_phase::{DrawFunctionId, PhaseItem};
use bevy::render::render_resource::{CachedRenderPipelineId, RenderPassColorAttachment};
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
    render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass},
    render_resource::{LoadOp, Operations, RenderPassDescriptor},
    renderer::RenderContext,
    view::{ExtractedView, ViewTarget},
};

pub struct TransparentUI {
    pub sort_key: FloatOrd,
    pub entity: Entity,
    pub pipeline: CachedRenderPipelineId,
    pub draw_function: DrawFunctionId,
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
}

pub struct MainPassUINode {
    query:
        QueryState<(&'static RenderPhase<TransparentUI>, &'static ViewTarget), With<ExtractedView>>,
}

impl MainPassUINode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl Node for MainPassUINode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(MainPassUINode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (transparent_phase, target) = self
            .query
            .get_manual(world, view_entity)
            .expect("view entity should exist");
        // let clear_color = world.get_resource::<ClearColor>().unwrap();
        {
            let pass_descriptor = RenderPassDescriptor {
                label: Some("main_transparent_pass_UI"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &target.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load, //Clear(clear_color.0.into()),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            };

            let draw_functions = world
                .get_resource::<DrawFunctions<TransparentUI>>()
                .unwrap();

            let render_pass = render_context
                .command_encoder
                .begin_render_pass(&pass_descriptor);
            let mut draw_functions = draw_functions.write();
            let mut tracked_pass = TrackedRenderPass::new(render_pass);
            for item in transparent_phase.items.iter() {
                let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
                draw_function.draw(world, &mut tracked_pass, view_entity, item);
            }
        }

        Ok(())
    }
}
