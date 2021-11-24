use bevy::{
    core::FloatOrd,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemState,
    },
    math::{const_vec3, Mat4, Quat, Vec3},
    prelude::{Bundle, Entity, FromWorld, Handle, Query, Res, ResMut, World},
    render2::{
        color::Color,
        render_phase::{Draw, DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent,
            BlendFactor, BlendOperation, BlendState, BufferBindingType, BufferSize, BufferUsages,
            BufferVec, CachedPipelineId, ColorTargetState, ColorWrites, DynamicUniformVec,
            FragmentState, FrontFace, MultisampleState, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor, Shader, ShaderStages,
            TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
        view::{ViewUniformOffset, ViewUniforms},
    },
    sprite2::Rect,
};
use bytemuck::{Pod, Zeroable};
use crevice::std140::AsStd140;

use super::font::{FontTextureCache, KayakFont};
use super::UNIFIED_SHADER_HANDLE;
use crate::render::ui_pass::TransparentUI;

pub struct UnifiedPipeline {
    view_layout: BindGroupLayout,
    types_layout: BindGroupLayout,
    pub(crate) image_layout: BindGroupLayout,
    pipeline: CachedPipelineId,
    empty_font_texture: (GpuImage, BindGroup),
}

const QUAD_VERTEX_POSITIONS: &[Vec3] = &[
    const_vec3!([0.0, 0.0, 0.0]),
    const_vec3!([1.0, 1.0, 0.0]),
    const_vec3!([0.0, 1.0, 0.0]),
    const_vec3!([0.0, 0.0, 0.0]),
    const_vec3!([1.0, 0.0, 0.0]),
    const_vec3!([1.0, 1.0, 0.0]),
];

impl FromWorld for UnifiedPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let mut pipeline_cache = world.get_resource_mut::<RenderPipelineCache>().unwrap();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // TODO: change this to ViewUniform::std140_size_static once crevice fixes this!
                    // Context: https://github.com/LPGhatguy/crevice/issues/29
                    min_binding_size: BufferSize::new(144),
                },
                count: None,
            }],
            label: Some("ui_view_layout"),
        });

        let types_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    // TODO: change this to ViewUniform::std140_size_static once crevice fixes this!
                    // Context: https://github.com/LPGhatguy/crevice/issues/29
                    min_binding_size: BufferSize::new(4),
                },
                count: None,
            }],
            label: Some("ui_types_layout"),
        });

        let image_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2Array,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
            label: Some("text_image_layout"),
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 40,
            step_mode: VertexStepMode::Vertex,
            attributes: vec![
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 12,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 28,
                    shader_location: 2,
                },
            ],
        };

        let empty_font_texture = FontTextureCache::get_empty(&render_device, &image_layout);

        let pipeline_desc = RenderPipelineDescriptor {
            vertex: VertexState {
                shader: UNIFIED_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: UNIFIED_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::OneMinusSrcAlpha,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent {
                            src_factor: BlendFactor::One,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                    }),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            layout: Some(vec![
                view_layout.clone(),
                image_layout.clone(),
                types_layout.clone(),
            ]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("quad_pipeline".into()),
        };

        UnifiedPipeline {
            pipeline: pipeline_cache.queue(pipeline_desc),
            view_layout,
            image_layout,
            empty_font_texture,
            types_layout,
        }
    }
}

#[derive(Bundle)]
pub struct ExtractQuadBundle {
    pub(crate) extracted_quad: ExtractedQuad,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIQuadType {
    Quad,
    Text,
}

pub struct ExtractedQuad {
    pub rect: Rect,
    pub color: Color,
    pub vertex_index: usize,
    pub char_id: usize,
    pub z_index: f32,
    pub font_handle: Option<Handle<KayakFont>>,
    pub quad_type: UIQuadType,
    pub type_index: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct QuadVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, AsStd140)]
struct QuadType {
    pub t: i32,
}

pub struct QuadMeta {
    vertices: BufferVec<QuadVertex>,
    view_bind_group: Option<BindGroup>,
    types_buffer: DynamicUniformVec<QuadType>,
    types_bind_group: Option<BindGroup>,
}

impl Default for QuadMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
            types_buffer: DynamicUniformVec::default(),
            types_bind_group: None,
        }
    }
}

pub fn prepare_quads(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut sprite_meta: ResMut<QuadMeta>,
    mut extracted_quads: Query<&mut ExtractedQuad>,
) {
    let extracted_sprite_len = extracted_quads.iter_mut().len();
    // dont create buffers when there are no quads
    if extracted_sprite_len == 0 {
        return;
    }

    sprite_meta.types_buffer.clear();
    sprite_meta.types_buffer.reserve(2, &render_device);
    let quad_type_offset = sprite_meta.types_buffer.push(QuadType { t: 0 });
    let text_type_offset = sprite_meta.types_buffer.push(QuadType { t: 1 });
    sprite_meta
        .types_buffer
        .write_buffer(&render_device, &render_queue);

    sprite_meta.vertices.clear();
    sprite_meta.vertices.reserve(
        extracted_sprite_len * QUAD_VERTEX_POSITIONS.len(),
        &render_device,
    );

    for (i, mut extracted_sprite) in extracted_quads.iter_mut().enumerate() {
        let sprite_rect = extracted_sprite.rect;
        let color = extracted_sprite.color.as_linear_rgba_f32();

        match extracted_sprite.quad_type {
            UIQuadType::Quad => extracted_sprite.type_index = quad_type_offset,
            UIQuadType::Text => extracted_sprite.type_index = text_type_offset,
        };

        let bottom_left = Vec3::new(0.0, 1.0, extracted_sprite.char_id as f32);
        let top_left = Vec3::new(0.0, 0.0, extracted_sprite.char_id as f32);
        let top_right = Vec3::new(1.0, 0.0, extracted_sprite.char_id as f32);
        let bottom_right = Vec3::new(1.0, 1.0, extracted_sprite.char_id as f32);

        let uvs: [[f32; 3]; 6] = [
            bottom_left.into(),
            top_right.into(),
            top_left.into(),
            bottom_left.into(),
            bottom_right.into(),
            top_right.into(),
        ];

        extracted_sprite.vertex_index = i;
        for (index, vertex_position) in QUAD_VERTEX_POSITIONS.iter().enumerate() {
            let world = Mat4::from_scale_rotation_translation(
                sprite_rect.size().extend(1.0),
                Quat::default(),
                sprite_rect.min.extend(0.0),
            );
            let final_position = (world * Vec3::from(*vertex_position).extend(1.0)).truncate();
            sprite_meta.vertices.push(QuadVertex {
                position: final_position.into(),
                color,
                uv: uvs[index],
            });
        }
    }
    sprite_meta
        .vertices
        .write_buffer(&render_device, &render_queue);
}

pub fn queue_quads(
    draw_functions: Res<DrawFunctions<TransparentUI>>,
    render_device: Res<RenderDevice>,
    mut sprite_meta: ResMut<QuadMeta>,
    view_uniforms: Res<ViewUniforms>,
    quad_pipeline: Res<UnifiedPipeline>,
    mut extracted_sprites: Query<(Entity, &ExtractedQuad)>,
    mut views: Query<&mut RenderPhase<TransparentUI>>,
) {
    if let Some(type_binding) = sprite_meta.types_buffer.binding() {
        sprite_meta.types_bind_group =
            Some(render_device.create_bind_group(&BindGroupDescriptor {
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: type_binding,
                }],
                label: Some("quad_type_bind_group"),
                layout: &quad_pipeline.types_layout,
            }));
    }

    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        sprite_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("quad_view_bind_group"),
            layout: &quad_pipeline.view_layout,
        }));
        let draw_quad = draw_functions.read().get_id::<DrawUI>().unwrap();
        for mut transparent_phase in views.iter_mut() {
            for (entity, quad) in extracted_sprites.iter_mut() {
                transparent_phase.add(TransparentUI {
                    draw_function: draw_quad,
                    pipeline: quad_pipeline.pipeline,
                    entity,
                    sort_key: FloatOrd(quad.z_index),
                });
            }
        }
    }
}

pub struct DrawUI {
    params: SystemState<(
        SRes<QuadMeta>,
        SRes<UnifiedPipeline>,
        SRes<RenderPipelineCache>,
        SRes<FontTextureCache>,
        SQuery<Read<ViewUniformOffset>>,
        SQuery<Read<ExtractedQuad>>,
    )>,
}

impl DrawUI {
    pub fn new(world: &mut World) -> Self {
        Self {
            params: SystemState::new(world),
        }
    }
}

impl Draw<TransparentUI> for DrawUI {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &TransparentUI,
    ) {
        let (quad_meta, unified_pipeline, pipelines, font_texture_cache, views, quads) =
            self.params.get(world);
        let view_uniform = views.get(view).unwrap();
        let quad_meta = quad_meta.into_inner();
        let extracted_quad = quads.get(item.entity).unwrap();
        if let Some(pipeline) = pipelines.into_inner().get(item.pipeline) {
            pass.set_render_pipeline(pipeline);
            pass.set_vertex_buffer(0, quad_meta.vertices.buffer().unwrap().slice(..));
            pass.set_bind_group(
                0,
                quad_meta.view_bind_group.as_ref().unwrap(),
                &[view_uniform.offset],
            );

            pass.set_bind_group(
                2,
                quad_meta.types_bind_group.as_ref().unwrap(),
                &[extracted_quad.type_index],
            );

            if let Some(font_handle) = extracted_quad.font_handle.as_ref() {
                if let Some(image_bindings) =
                    font_texture_cache.into_inner().bind_groups.get(font_handle)
                {
                    pass.set_bind_group(1, image_bindings, &[]);
                } else {
                    pass.set_bind_group(
                        1,
                        &unified_pipeline.into_inner().empty_font_texture.1,
                        &[],
                    );
                }
            } else {
                pass.set_bind_group(1, &unified_pipeline.into_inner().empty_font_texture.1, &[]);
            }

            pass.draw(
                (extracted_quad.vertex_index * QUAD_VERTEX_POSITIONS.len()) as u32
                    ..((extracted_quad.vertex_index + 1) * QUAD_VERTEX_POSITIONS.len()) as u32,
                0..1,
            );
        }
    }
}
