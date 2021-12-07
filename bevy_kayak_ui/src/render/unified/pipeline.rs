use bevy::{
    core::FloatOrd,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemState,
    },
    math::{const_vec3, Mat4, Quat, Vec2, Vec3, Vec4},
    prelude::{Bundle, Component, Entity, FromWorld, Handle, Query, Res, ResMut, World},
    render2::{
        color::Color,
        render_asset::RenderAssets,
        render_phase::{Draw, DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BlendComponent, BlendFactor, BlendOperation, BlendState, BufferBindingType, BufferSize,
            BufferUsages, BufferVec, CachedPipelineId, ColorTargetState, ColorWrites,
            DynamicUniformVec, Extent3d, FragmentState, FrontFace, MultisampleState, PolygonMode,
            PrimitiveState, PrimitiveTopology, RenderPipelineCache, RenderPipelineDescriptor,
            SamplerDescriptor, Shader, ShaderStages, TextureDescriptor, TextureDimension,
            TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor,
            TextureViewDimension, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage, Image},
        view::{ViewUniformOffset, ViewUniforms},
    },
    sprite2::Rect,
    utils::HashMap,
};
use bytemuck::{Pod, Zeroable};
use crevice::std140::AsStd140;

use super::font::{FontTextureCache, KayakFont};
use super::UNIFIED_SHADER_HANDLE;
use crate::render::ui_pass::TransparentUI;

pub struct UnifiedPipeline {
    view_layout: BindGroupLayout,
    types_layout: BindGroupLayout,
    pub(crate) font_image_layout: BindGroupLayout,
    image_layout: BindGroupLayout,
    pipeline: CachedPipelineId,
    empty_font_texture: (GpuImage, BindGroup),
    default_image: (GpuImage, BindGroup),
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

        // Used by fonts
        let font_image_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

        let image_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
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
            label: Some("image_layout"),
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 60,
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
                    format: VertexFormat::Float32x4,
                    offset: 28,
                    shader_location: 2,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 44,
                    shader_location: 3,
                },
            ],
        };

        let empty_font_texture = FontTextureCache::get_empty(&render_device, &font_image_layout);

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
                font_image_layout.clone(),
                types_layout.clone(),
                image_layout.clone(),
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
            label: Some("unified_pipeline".into()),
        };

        let texture_descriptor = TextureDescriptor {
            label: Some("font_texture_array"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };

        let sampler_descriptor = SamplerDescriptor::default();

        let texture = render_device.create_texture(&texture_descriptor);
        let sampler = render_device.create_sampler(&sampler_descriptor);

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("font_texture_array_view"),
            format: None,
            dimension: Some(TextureViewDimension::D2),
            aspect: bevy::render2::render_resource::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            mip_level_count: None,
            array_layer_count: None,
        });

        let image = GpuImage {
            texture,
            sampler,
            texture_view,
        };

        let binding = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("text_image_bind_group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&image.texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&image.sampler),
                },
            ],
            layout: &image_layout,
        });

        UnifiedPipeline {
            pipeline: pipeline_cache.queue(pipeline_desc),
            view_layout,
            font_image_layout,
            empty_font_texture,
            types_layout,
            image_layout,
            default_image: (image, binding),
        }
    }
}

#[derive(Debug, Bundle)]
pub struct ExtractQuadBundle {
    pub(crate) extracted_quad: ExtractedQuad,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UIQuadType {
    Quad,
    Text,
    Image,
}

#[derive(Debug, Component, Clone)]
pub struct ExtractedQuad {
    pub rect: Rect,
    pub color: Color,
    pub vertex_index: usize,
    pub char_id: usize,
    pub z_index: f32,
    pub font_handle: Option<Handle<KayakFont>>,
    pub quad_type: UIQuadType,
    pub type_index: u32,
    pub border_radius: (f32, f32, f32, f32),
    pub image: Option<Handle<Image>>,
    pub uv_min: Option<Vec2>,
    pub uv_max: Option<Vec2>,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct QuadVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 4],
    pub pos_size: [f32; 4],
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

#[derive(Default)]
pub struct ImageBindGroups {
    values: HashMap<Handle<Image>, BindGroup>,
}

pub fn prepare_quads(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut sprite_meta: ResMut<QuadMeta>,
    mut extracted_quads: Query<&mut ExtractedQuad>,
) {
    let extracted_sprite_len = extracted_quads.iter_mut().len();
    // don't create buffers when there are no quads
    if extracted_sprite_len == 0 {
        return;
    }

    sprite_meta.types_buffer.clear();
    sprite_meta.types_buffer.reserve(2, &render_device);
    let quad_type_offset = sprite_meta.types_buffer.push(QuadType { t: 0 });
    let text_type_offset = sprite_meta.types_buffer.push(QuadType { t: 1 });
    let image_type_offset = sprite_meta.types_buffer.push(QuadType { t: 2 });
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
            UIQuadType::Image => extracted_sprite.type_index = image_type_offset,
        };

        let uv_min = extracted_sprite.uv_min.unwrap_or(Vec2::ZERO);
        let uv_max = extracted_sprite.uv_max.unwrap_or(Vec2::ONE);

        let bottom_left = Vec4::new(
            uv_min.x,
            uv_max.y,
            extracted_sprite.char_id as f32,
            extracted_sprite.border_radius.0,
        );
        let top_left = Vec4::new(
            uv_min.x,
            uv_min.y,
            extracted_sprite.char_id as f32,
            extracted_sprite.border_radius.1,
        );
        let top_right = Vec4::new(
            uv_max.x,
            uv_min.y,
            extracted_sprite.char_id as f32,
            extracted_sprite.border_radius.2,
        );
        let bottom_right = Vec4::new(
            uv_max.x,
            uv_max.y,
            extracted_sprite.char_id as f32,
            extracted_sprite.border_radius.3,
        );

        let uvs: [[f32; 4]; 6] = [
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
                pos_size: [
                    sprite_rect.min.x,
                    sprite_rect.min.y,
                    sprite_rect.size().x,
                    sprite_rect.size().y,
                ],
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
    mut image_bind_groups: ResMut<ImageBindGroups>,
    unified_pipeline: Res<UnifiedPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
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
                if let Some(image_handle) = quad.image.as_ref() {
                    image_bind_groups
                        .values
                        .entry(image_handle.clone_weak())
                        .or_insert_with(|| {
                            let gpu_image = gpu_images.get(&image_handle).unwrap();
                            render_device.create_bind_group(&BindGroupDescriptor {
                                entries: &[
                                    BindGroupEntry {
                                        binding: 0,
                                        resource: BindingResource::TextureView(
                                            &gpu_image.texture_view,
                                        ),
                                    },
                                    BindGroupEntry {
                                        binding: 1,
                                        resource: BindingResource::Sampler(&gpu_image.sampler),
                                    },
                                ],
                                label: Some("ui_image_bind_group"),
                                layout: &unified_pipeline.image_layout,
                            })
                        });
                }
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
        SRes<ImageBindGroups>,
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
        let (
            quad_meta,
            unified_pipeline,
            pipelines,
            font_texture_cache,
            image_bind_groups,
            views,
            quads,
        ) = self.params.get(world);

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

            let unified_pipeline = unified_pipeline.into_inner();
            if let Some(font_handle) = extracted_quad.font_handle.as_ref() {
                if let Some(image_bindings) =
                    font_texture_cache.into_inner().bind_groups.get(font_handle)
                {
                    pass.set_bind_group(1, image_bindings, &[]);
                } else {
                    pass.set_bind_group(1, &unified_pipeline.empty_font_texture.1, &[]);
                }
            } else {
                pass.set_bind_group(1, &unified_pipeline.empty_font_texture.1, &[]);
            }

            if let Some(image_handle) = extracted_quad.image.as_ref() {
                pass.set_bind_group(
                    3,
                    &image_bind_groups
                        .into_inner()
                        .values
                        .get(image_handle)
                        .unwrap(),
                    &[],
                );
            } else {
                pass.set_bind_group(3, &unified_pipeline.default_image.1, &[]);
            }

            pass.draw(
                (extracted_quad.vertex_index * QUAD_VERTEX_POSITIONS.len()) as u32
                    ..((extracted_quad.vertex_index + 1) * QUAD_VERTEX_POSITIONS.len()) as u32,
                0..1,
            );
        }
    }
}
