use bevy::ecs::prelude::*;
use bevy::render::render_phase::{DrawFunctionId, PhaseItem};
use bevy::render::render_resource::CachedRenderPipelineId;
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

    fn entity(&self) -> Entity {
        self.entity
    }
}

pub struct MainPassUINode {
    query: QueryState<
        (
            &'static RenderPhase<TransparentUI>,
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
        let (transparent_phase, target, _camera_ui) =
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
        // let clear_color = world.get_resource::<ClearColor>().unwrap();
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
