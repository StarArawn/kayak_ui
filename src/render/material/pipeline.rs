use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{
            DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult, RenderPhase,
            SetItemPipeline, TrackedRenderPass,
        },
        render_resource::{
            AsBindGroupError, BindGroup, BindGroupLayout, OwnedBindingResource, PipelineCache,
            RenderPipelineDescriptor, ShaderRef, SpecializedRenderPipeline,
            SpecializedRenderPipelines,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
        Extract,
    },
    utils::{HashMap, HashSet},
};
use kayak_font::bevy::FontTextureCache;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::render::{
    extract::UIExtractedView,
    opacity_layer::OpacityLayerManager,
    svg::RenderSvgs,
    ui_pass::{TransparentOpacityUI, TransparentUI},
    unified::pipeline::{
        queue_quads_inner, DrawUIDraw, ExtractedQuad, ImageBindGroups, QuadBatch, QuadMeta,
        QuadTypeOffsets, SetUIViewBindGroup, UIQuadType, UnifiedPipeline, UnifiedPipelineKey,
    },
};

use super::{key::MaterialUIKey, MaterialUI};

/// Render pipeline data for a given [`MaterialUI`]
#[derive(Resource)]
pub struct MaterialUIPipeline<M: MaterialUI> {
    pub unified_pipeline: UnifiedPipeline,
    pub material_ui_layout: BindGroupLayout,
    pub vertex_shader: Option<Handle<Shader>>,
    pub fragment_shader: Option<Handle<Shader>>,
    marker: PhantomData<M>,
}

impl<M: MaterialUI> Clone for MaterialUIPipeline<M> {
    fn clone(&self) -> Self {
        Self {
            unified_pipeline: self.unified_pipeline.clone(),
            material_ui_layout: self.material_ui_layout.clone(),
            vertex_shader: self.vertex_shader.clone(),
            fragment_shader: self.fragment_shader.clone(),
            marker: PhantomData,
        }
    }
}

impl<M: MaterialUI> SpecializedRenderPipeline for MaterialUIPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = MaterialUIKey<M>;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let mut descriptor = self.unified_pipeline.specialize(key.unified_key);
        if let Some(vertex_shader) = &self.vertex_shader {
            descriptor.vertex.shader = vertex_shader.clone();
        }

        if let Some(fragment_shader) = &self.fragment_shader {
            descriptor.fragment.as_mut().unwrap().shader = fragment_shader.clone();
        }
        descriptor.layout = vec![
            self.unified_pipeline.view_layout.clone(),
            self.unified_pipeline.image_layout.clone(),
            self.unified_pipeline.types_layout.clone(),
            self.material_ui_layout.clone(),
        ];

        M::specialize(&mut descriptor, key);
        descriptor
    }
}

impl<M: MaterialUI> FromWorld for MaterialUIPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let render_device = world.resource::<RenderDevice>();
        let material_ui_layout = M::bind_group_layout(render_device);

        MaterialUIPipeline {
            unified_pipeline: world.resource::<UnifiedPipeline>().clone(),
            material_ui_layout,
            vertex_shader: match M::vertex_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            fragment_shader: match M::fragment_shader() {
                ShaderRef::Default => None,
                ShaderRef::Handle(handle) => Some(handle),
                ShaderRef::Path(path) => Some(asset_server.load(path)),
            },
            marker: PhantomData,
        }
    }
}

/// Data prepared for a [`MaterialUI`] instance.
pub struct PreparedMaterialUI<T: MaterialUI> {
    pub bindings: Vec<OwnedBindingResource>,
    pub bind_group: BindGroup,
    pub key: T::Data,
}

#[derive(Resource)]
pub struct ExtractedMaterialsUI<M: MaterialUI> {
    extracted: Vec<(Handle<M>, M)>,
    removed: Vec<Handle<M>>,
}

impl<M: MaterialUI> Default for ExtractedMaterialsUI<M> {
    fn default() -> Self {
        Self {
            extracted: Default::default(),
            removed: Default::default(),
        }
    }
}

/// Stores all prepared representations of [`MaterialUI`] assets for as long as they exist.
#[derive(Resource, Deref, DerefMut)]
pub struct RenderMaterialsUI<T: MaterialUI>(HashMap<Handle<T>, PreparedMaterialUI<T>>);

impl<T: MaterialUI> Default for RenderMaterialsUI<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// This system extracts all created or modified assets of the corresponding [`Material2d`] type
/// into the "render world".
pub fn extract_materials_ui<M: MaterialUI>(
    mut commands: Commands,
    mut events: Extract<EventReader<AssetEvent<M>>>,
    assets: Extract<Res<Assets<M>>>,
) {
    let mut changed_assets = HashSet::default();
    let mut removed = Vec::new();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                removed.push(handle.clone_weak());
            }
        }
    }

    let mut extracted_assets = Vec::new();
    for handle in changed_assets.drain() {
        if let Some(asset) = assets.get(&handle) {
            extracted_assets.push((handle, asset.clone()));
        }
    }

    commands.insert_resource(ExtractedMaterialsUI {
        extracted: extracted_assets,
        removed,
    });
}

/// All [`MaterialUI`] values of a given type that should be prepared next frame.
pub struct PrepareNextFrameMaterials<M: MaterialUI> {
    assets: Vec<(Handle<M>, M)>,
}

impl<M: MaterialUI> Default for PrepareNextFrameMaterials<M> {
    fn default() -> Self {
        Self {
            assets: Default::default(),
        }
    }
}

/// This system prepares all assets of the corresponding [`MaterialUI`] type
/// which where extracted this frame for the GPU.
pub fn prepare_materials_ui<M: MaterialUI>(
    mut prepare_next_frame: Local<PrepareNextFrameMaterials<M>>,
    mut extracted_assets: ResMut<ExtractedMaterialsUI<M>>,
    mut render_materials: ResMut<RenderMaterialsUI<M>>,
    render_device: Res<RenderDevice>,
    images: Res<RenderAssets<Image>>,
    fallback_image: Res<FallbackImage>,
    pipeline: Res<MaterialUIPipeline<M>>,
) {
    let queued_assets = std::mem::take(&mut prepare_next_frame.assets);
    for (handle, material) in queued_assets {
        match prepare_materialui(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }

    for removed in std::mem::take(&mut extracted_assets.removed) {
        render_materials.remove(&removed);
    }

    for (handle, material) in std::mem::take(&mut extracted_assets.extracted) {
        match prepare_materialui(
            &material,
            &render_device,
            &images,
            &fallback_image,
            &pipeline,
        ) {
            Ok(prepared_asset) => {
                render_materials.insert(handle, prepared_asset);
            }
            Err(AsBindGroupError::RetryNextUpdate) => {
                prepare_next_frame.assets.push((handle, material));
            }
        }
    }
}

pub fn prepare_materialui<M: MaterialUI>(
    material: &M,
    render_device: &RenderDevice,
    images: &RenderAssets<Image>,
    fallback_image: &FallbackImage,
    pipeline: &MaterialUIPipeline<M>,
) -> Result<PreparedMaterialUI<M>, AsBindGroupError> {
    let prepared = material.as_bind_group(
        &pipeline.material_ui_layout,
        render_device,
        images,
        fallback_image,
    )?;
    Ok(PreparedMaterialUI {
        bindings: prepared.bindings,
        bind_group: prepared.bind_group,
        key: prepared.data,
    })
}

pub type DrawMaterialUI<M> = (
    SetItemPipeline,
    SetUIViewBindGroup<TransparentUI, 0>,
    SetMaterialBindGroup<M, 3>,
    DrawUIDraw<TransparentUI>,
);

pub type DrawMaterialUITransparent<M> = (
    SetItemPipeline,
    SetUIViewBindGroup<TransparentOpacityUI, 0>,
    SetMaterialBindGroup<M, 3>,
    DrawUIDraw<TransparentOpacityUI>,
);

pub struct SetMaterialBindGroup<M: MaterialUI, const I: usize>(PhantomData<M>);
impl<P: PhaseItem, M: MaterialUI, const I: usize> RenderCommand<P> for SetMaterialBindGroup<M, I> {
    type Param = SRes<RenderMaterialsUI<M>>;
    type ViewWorldQuery = ();
    type ItemWorldQuery = Read<Handle<M>>;

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: (),
        material2d_handle: ROQueryItem<'_, Self::ItemWorldQuery>,
        materials: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let material2d = materials.into_inner().get(material2d_handle).unwrap();
        pass.set_bind_group(I, &material2d.bind_group, &[]);
        RenderCommandResult::Success
    }
}

pub fn queue_material_ui_quads<M: MaterialUI>(
    render_svgs: Res<RenderSvgs>,
    opacity_layers: Res<OpacityLayerManager>,
    mut commands: Commands,
    draw_functions: Res<DrawFunctions<TransparentUI>>,
    draw_functions_opacity: Res<DrawFunctions<TransparentOpacityUI>>,
    render_device: Res<RenderDevice>,
    mut quad_meta: ResMut<QuadMeta>,
    quad_pipeline: Res<UnifiedPipeline>,
    materialui_pipeline: Res<MaterialUIPipeline<M>>,
    mut pipelines: ResMut<SpecializedRenderPipelines<MaterialUIPipeline<M>>>,
    pipeline_cache: ResMut<PipelineCache>,
    mut extracted_quads: Query<(&'static mut ExtractedQuad, &'static Handle<M>)>,
    mut views: Query<(
        Entity,
        &'static mut RenderPhase<TransparentUI>,
        &'static mut RenderPhase<TransparentOpacityUI>,
        &'static UIExtractedView,
    )>,
    mut image_bind_groups: ResMut<ImageBindGroups>,
    (gpu_images, font_texture_cache, quad_types_offsets, render_materials): (
        Res<RenderAssets<Image>>,
        Res<FontTextureCache>,
        Res<QuadTypeOffsets>,
        Res<RenderMaterialsUI<M>>,
    ),
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let mut current_batch = QuadBatch {
        image_handle_id: None,
        font_handle_id: None,
        quad_type: UIQuadType::None,
        type_id: quad_types_offsets.quad_type_offset,
        z_index: -999.0,
    };
    let mut current_batch_entity = Entity::PLACEHOLDER;

    // Vertex buffer indices
    let mut index = 0;

    let mut previous_clip_rect = Rect::default();

    let draw_quad = draw_functions.read().get_id::<DrawMaterialUI<M>>().unwrap();
    let draw_opacity_quad = draw_functions_opacity
        .read()
        .get_id::<DrawMaterialUITransparent<M>>()
        .unwrap();
    for (camera_entity, mut transparent_phase, mut opacity_transparent_phase, view) in
        views.iter_mut()
    {
        let key = UnifiedPipelineKey {
            msaa: 1,
            hdr: view.hdr,
        };

        for (mut quad, material_handle) in extracted_quads.iter_mut() {
            if let Some(materialui) = render_materials.get(material_handle) {
                let pipeline_id = pipelines.specialize(
                    &pipeline_cache,
                    &materialui_pipeline,
                    MaterialUIKey {
                        unified_key: key,
                        bind_group_data: materialui.key.clone(),
                    },
                );

                queue_quads_inner(
                    &mut commands,
                    &render_device,
                    &font_texture_cache,
                    &opacity_layers,
                    &mut image_bind_groups,
                    &gpu_images,
                    &quad_pipeline,
                    &render_svgs,
                    &mut transparent_phase,
                    &mut opacity_transparent_phase,
                    draw_opacity_quad,
                    draw_quad,
                    pipeline_id,
                    &mut quad_meta,
                    &mut quad,
                    camera_entity,
                    &mut previous_clip_rect,
                    *quad_types_offsets,
                    &mut current_batch,
                    &mut current_batch_entity,
                    &mut index,
                );

                if current_batch_entity != Entity::PLACEHOLDER {
                    commands
                        .entity(current_batch_entity)
                        .insert(material_handle.clone_weak());
                }
            }
        }
    }
}
