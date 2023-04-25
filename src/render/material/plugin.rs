use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::PrepareAssetSet,
        render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, RenderApp,
        RenderSet,
    },
};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::render::ui_pass::{TransparentUI, TransparentOpacityUI};

use super::{
    pipeline::{
        extract_materials_ui, prepare_materials_ui, queue_material_ui_quads, DrawMaterialUI,
        ExtractedMaterialsUI, MaterialUIPipeline, RenderMaterialsUI,
    },
    MaterialUI, DrawMaterialUITransparent,
};

/// Adds the necessary ECS resources and render logic to enable rendering entities using the given [`MaterialUI`]
/// asset type (which includes [`MaterialUI`] types).
pub struct MaterialUIPlugin<M: MaterialUI>(PhantomData<M>);

impl<M: MaterialUI> Default for MaterialUIPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: MaterialUI> Plugin for MaterialUIPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        app.add_asset::<M>()
            .add_plugin(ExtractComponentPlugin::<Handle<M>>::extract_visible());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<TransparentUI, DrawMaterialUI<M>>()
                .add_render_command::<TransparentOpacityUI, DrawMaterialUITransparent<M>>()
                .init_resource::<MaterialUIPipeline<M>>()
                .init_resource::<ExtractedMaterialsUI<M>>()
                .init_resource::<RenderMaterialsUI<M>>()
                .init_resource::<SpecializedRenderPipelines<MaterialUIPipeline<M>>>()
                .add_system(extract_materials_ui::<M>.in_schedule(ExtractSchedule))
                .add_systems((
                    prepare_materials_ui::<M>
                        .in_set(RenderSet::Prepare)
                        .after(PrepareAssetSet::PreAssetPrepare),
                    queue_material_ui_quads::<M>
                        .in_set(RenderSet::Queue)
                        .after(crate::render::unified::pipeline::queue_quads),
                ));
        }
    }
}
