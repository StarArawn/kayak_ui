use bevy::{
    prelude::*,
    render::{
        extract_component::ExtractComponentPlugin, render_asset::PrepareAssetSet,
        render_phase::AddRenderCommand, render_resource::SpecializedRenderPipelines, Render,
        RenderApp, RenderSet,
    },
};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::render::ui_pass::{TransparentOpacityUI, TransparentUI};

use super::{
    pipeline::{
        extract_materials_ui, prepare_materials_ui, queue_material_ui_quads, DrawMaterialUI,
        ExtractedMaterialsUI, MaterialUIPipeline, RenderMaterialsUI,
    },
    DrawMaterialUITransparent, MaterialUI,
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
            .add_plugins(ExtractComponentPlugin::<Handle<M>>::extract_visible());

        app.sub_app_mut(RenderApp)
            .add_render_command::<TransparentUI, DrawMaterialUI<M>>()
            .add_render_command::<TransparentOpacityUI, DrawMaterialUITransparent<M>>()
            .init_resource::<ExtractedMaterialsUI<M>>()
            .init_resource::<RenderMaterialsUI<M>>()
            .init_resource::<SpecializedRenderPipelines<MaterialUIPipeline<M>>>()
            .add_systems(ExtractSchedule, extract_materials_ui::<M>)
            .add_systems(
                Render,
                (
                    prepare_materials_ui::<M>
                        .in_set(RenderSet::Prepare)
                        .after(PrepareAssetSet::PreAssetPrepare),
                    queue_material_ui_quads::<M>
                        .in_set(RenderSet::Queue)
                        .after(crate::render::unified::pipeline::queue_quads),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        app.sub_app_mut(RenderApp)
            .init_resource::<MaterialUIPipeline<M>>();
    }
}
