use bevy::{
    core_pipeline::core_2d::Transparent2d,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemState,
    },
    math::{Mat4, Quat, Vec2, Vec3, Vec4},
    prelude::{Bundle, Component, Entity, FromWorld, Handle, Query, Res, ResMut, World},
    render::{
        color::Color,
        render_phase::{Draw, DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent,
            BlendFactor, BlendOperation, BlendState, BufferBindingType, BufferSize, BufferUsages,
            BufferVec, CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState,
            FrontFace, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPipelineDescriptor, SamplerBindingType, Shader, ShaderStages,
            ShaderType, TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
        view::{ViewUniformOffset, ViewUniforms},
    },
    sprite::Rect,
    utils::FloatOrd,
};
use bytemuck::{Pod, Zeroable};
use kayak_font::{
    bevy::{FontRenderingPipeline, FontTextureCache},
    KayakFont,
};

use super::FONT_SHADER_HANDLE;

pub struct FontPipeline {
    view_layout: BindGroupLayout,
    pub(crate) font_image_layout: BindGroupLayout,
    pipeline: CachedRenderPipelineId,
    empty_font_texture: (GpuImage, BindGroup),
}

const QUAD_VERTEX_POSITIONS: &[Vec3] = &[
    Vec3::from_array([0.0, 0.0, 0.0]),
    Vec3::from_array([1.0, 1.0, 0.0]),
    Vec3::from_array([0.0, 1.0, 0.0]),
    Vec3::from_array([0.0, 0.0, 0.0]),
    Vec3::from_array([1.0, 0.0, 0.0]),
    Vec3::from_array([1.0, 1.0, 0.0]),
];

impl FontRenderingPipeline for FontPipeline {
    fn get_font_image_layout(&self) -> &BindGroupLayout {
        &self.font_image_layout
    }
}

impl FromWorld for FontPipeline {
    fn from_world(world: &mut World) -> Self {
        let world = world.cell();
        let render_device = world.get_resource::<RenderDevice>().unwrap();
        let mut pipeline_cache = world.get_resource_mut::<PipelineCache>().unwrap();

        let view_layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(144),
                },
                count: None,
            }],
            label: Some("ui_view_layout"),
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
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2Array,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("text_image_layout"),
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
                shader: FONT_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: FONT_SHADER_HANDLE.typed::<Shader>(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
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
                })],
            }),
            layout: Some(vec![view_layout.clone(), font_image_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                unclipped_depth: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("font_pipeline".into()),
        };

        FontPipeline {
            pipeline: pipeline_cache.queue_render_pipeline(pipeline_desc),
            view_layout,
            font_image_layout,
            empty_font_texture,
        }
    }
}

#[derive(Debug, Bundle)]
pub struct ExtractCharBundle {
    pub(crate) extracted_quad: ExtractedChar,
}

#[derive(Debug, Component, Clone)]
pub struct ExtractedChar {
    pub rect: Rect,
    pub color: Color,
    pub vertex_index: usize,
    pub char_id: u32,
    pub z_index: f32,
    pub font_handle: Option<Handle<KayakFont>>,
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
#[derive(Copy, Clone, ShaderType)]
struct QuadType {
    pub t: i32,
}

pub struct QuadMeta {
    vertices: BufferVec<QuadVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for QuadMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

pub fn prepare_quads(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut sprite_meta: ResMut<QuadMeta>,
    mut extracted_quads: Query<&mut ExtractedChar>,
) {
    let extracted_sprite_len = extracted_quads.iter_mut().len();
    // don't create buffers when there are no quads
    if extracted_sprite_len == 0 {
        return;
    }

    sprite_meta.vertices.clear();
    sprite_meta.vertices.reserve(
        extracted_sprite_len * QUAD_VERTEX_POSITIONS.len(),
        &render_device,
    );

    for (i, mut extracted_sprite) in extracted_quads.iter_mut().enumerate() {
        let sprite_rect = extracted_sprite.rect;
        let color = extracted_sprite.color.as_linear_rgba_f32();

        let uv_min = Vec2::ZERO;
        let uv_max = Vec2::ONE;

        let bottom_left = Vec4::new(uv_min.x, uv_max.y, extracted_sprite.char_id as f32, 0.0);
        let top_left = Vec4::new(uv_min.x, uv_min.y, extracted_sprite.char_id as f32, 0.0);
        let top_right = Vec4::new(uv_max.x, uv_min.y, extracted_sprite.char_id as f32, 0.0);
        let bottom_right = Vec4::new(uv_max.x, uv_max.y, extracted_sprite.char_id as f32, 0.0);

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
    draw_functions: Res<DrawFunctions<Transparent2d>>,
    render_device: Res<RenderDevice>,
    mut sprite_meta: ResMut<QuadMeta>,
    view_uniforms: Res<ViewUniforms>,
    quad_pipeline: Res<FontPipeline>,
    mut extracted_sprites: Query<(Entity, &ExtractedChar)>,
    mut views: Query<&mut RenderPhase<Transparent2d>>,
) {
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
                transparent_phase.add(Transparent2d {
                    draw_function: draw_quad,
                    pipeline: quad_pipeline.pipeline,
                    entity,
                    sort_key: FloatOrd(quad.z_index),
                    batch_range: None,
                });
            }
        }
    }
}

pub struct DrawUI {
    params: SystemState<(
        SRes<QuadMeta>,
        SRes<FontPipeline>,
        SRes<PipelineCache>,
        SRes<FontTextureCache>,
        SQuery<Read<ViewUniformOffset>>,
        SQuery<Read<ExtractedChar>>,
    )>,
}

impl DrawUI {
    pub fn new(world: &mut World) -> Self {
        Self {
            params: SystemState::new(world),
        }
    }
}

impl Draw<Transparent2d> for DrawUI {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &Transparent2d,
    ) {
        let (quad_meta, unified_pipeline, pipelines, font_texture_cache, views, quads) =
            self.params.get(world);

        let view_uniform = views.get(view).unwrap();
        let quad_meta = quad_meta.into_inner();
        let extracted_quad = quads.get(item.entity).unwrap();
        if let Some(pipeline) = pipelines.into_inner().get_render_pipeline(item.pipeline) {
            pass.set_render_pipeline(pipeline);
            pass.set_vertex_buffer(0, quad_meta.vertices.buffer().unwrap().slice(..));
            pass.set_bind_group(
                0,
                quad_meta.view_bind_group.as_ref().unwrap(),
                &[view_uniform.offset],
            );

            let unified_pipeline = unified_pipeline.into_inner();
            if let Some(font_handle) = extracted_quad.font_handle.as_ref() {
                if let Some(image_bindings) =
                    font_texture_cache.into_inner().get_binding(font_handle)
                {
                    pass.set_bind_group(1, image_bindings, &[]);
                } else {
                    pass.set_bind_group(1, &unified_pipeline.empty_font_texture.1, &[]);
                }
            } else {
                pass.set_bind_group(1, &unified_pipeline.empty_font_texture.1, &[]);
            }

            pass.draw(
                (extracted_quad.vertex_index * QUAD_VERTEX_POSITIONS.len()) as u32
                    ..((extracted_quad.vertex_index + 1) * QUAD_VERTEX_POSITIONS.len()) as u32,
                0..1,
            );
        }
    }
}
