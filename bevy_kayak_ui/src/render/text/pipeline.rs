use bevy::{
    core::FloatOrd,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemState,
    },
    math::{const_vec3, Mat4, Quat, Vec2, Vec3},
    prelude::{
        Assets, Bundle, Color, Commands, Entity, FromWorld, Handle, Query, Res, ResMut, World,
    },
    render2::{
        render_phase::{Draw, DrawFunctions, RenderPhase, TrackedRenderPass},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendComponent,
            BlendFactor, BlendOperation, BlendState, BufferBindingType, BufferSize, BufferUsages,
            BufferVec, CachedPipelineId, ColorTargetState, ColorWrites, CompareFunction,
            DepthBiasState, DepthStencilState, FragmentState, FrontFace, MultisampleState,
            PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipelineCache,
            RenderPipelineDescriptor, Shader, ShaderStages, StencilFaceState, StencilState,
            TextureFormat, TextureSampleType, TextureViewDimension, VertexAttribute,
            VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ViewUniformOffset, ViewUniforms},
    },
    sprite2::Rect,
};
use bytemuck::{Pod, Zeroable};
use kayak_core::render_primitive::RenderPrimitive;

use crate::{
    render::{text::TEXT_SHADER_HANDLE, ui_pass::TransparentUI},
    to_bevy_color, BevyContext,
};

use super::{font::KayakFont, font_mapping::FontMapping, font_texture_cache::FontTextureCache};

pub struct TextPipeline {
    view_layout: BindGroupLayout,
    pub(crate) image_layout: BindGroupLayout,
    pipeline: CachedPipelineId,
}

const QUAD_VERTEX_POSITIONS: &[Vec3] = &[
    const_vec3!([0.0, 0.0, 0.0]),
    const_vec3!([1.0, 1.0, 0.0]),
    const_vec3!([0.0, 1.0, 0.0]),
    const_vec3!([0.0, 0.0, 0.0]),
    const_vec3!([1.0, 0.0, 0.0]),
    const_vec3!([1.0, 1.0, 0.0]),
];

impl FromWorld for TextPipeline {
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
            label: Some("text_view_layout"),
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

        let pipeline_desc = RenderPipelineDescriptor {
            vertex: VertexState {
                shader: TEXT_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: TEXT_SHADER_HANDLE.typed::<Shader>(),
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
            layout: Some(vec![view_layout.clone(), image_layout.clone()]),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            label: Some("text_pipeline".into()),
        };

        TextPipeline {
            pipeline: pipeline_cache.queue(pipeline_desc),
            view_layout,
            image_layout,
        }
    }
}



#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct TextVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 3],
}

pub struct TextMeta {
    vertices: BufferVec<TextVertex>,
    view_bind_group: Option<BindGroup>,
}

impl Default for TextMeta {
    fn default() -> Self {
        Self {
            vertices: BufferVec::new(BufferUsages::VERTEX),
            view_bind_group: None,
        }
    }
}

pub fn prepare_texts(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut sprite_meta: ResMut<TextMeta>,
    mut extracted_sprites: Query<&mut ExtractedText>,
) {
    let extracted_sprite_len = extracted_sprites.iter_mut().len();
    // dont create buffers when there are no texts
    if extracted_sprite_len == 0 {
        return;
    }

    sprite_meta.vertices.clear();
    sprite_meta.vertices.reserve(
        extracted_sprite_len * QUAD_VERTEX_POSITIONS.len(),
        &render_device,
    );

    for (i, mut extracted_sprite) in extracted_sprites.iter_mut().enumerate() {
        let sprite_rect = extracted_sprite.rect;
        let color = extracted_sprite.background_color.as_linear_rgba_f32();

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
                sprite_rect.min.extend(extracted_sprite.z_index),
            );
            let final_position = (world * Vec3::from(*vertex_position).extend(1.0)).truncate();
            sprite_meta.vertices.push(TextVertex {
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

pub fn queue_texts(
    draw_functions: Res<DrawFunctions<TransparentUI>>,
    render_device: Res<RenderDevice>,
    mut sprite_meta: ResMut<TextMeta>,
    view_uniforms: Res<ViewUniforms>,
    text_pipeline: Res<TextPipeline>,
    mut extracted_sprites: Query<(Entity, &ExtractedText)>,
    mut views: Query<&mut RenderPhase<TransparentUI>>,
) {
    if let Some(view_binding) = view_uniforms.uniforms.binding() {
        sprite_meta.view_bind_group = Some(render_device.create_bind_group(&BindGroupDescriptor {
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_binding,
            }],
            label: Some("text_view_bind_group"),
            layout: &text_pipeline.view_layout,
        }));
        let draw_text = draw_functions.read().get_id::<DrawText>().unwrap();
        for mut transparent_phase in views.iter_mut() {
            for (entity, _) in extracted_sprites.iter_mut() {
                transparent_phase.add(TransparentUI {
                    draw_function: draw_text,
                    pipeline: text_pipeline.pipeline,
                    entity,
                    sort_key: FloatOrd(0.0),
                });
            }
        }
    }
}

pub struct DrawText {
    params: SystemState<(
        SRes<TextMeta>,
        SRes<RenderPipelineCache>,
        SRes<FontTextureCache>,
        SQuery<Read<ViewUniformOffset>>,
        SQuery<Read<ExtractedText>>,
    )>,
}

impl DrawText {
    pub fn new(world: &mut World) -> Self {
        Self {
            params: SystemState::new(world),
        }
    }
}

impl Draw<TransparentUI> for DrawText {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &TransparentUI,
    ) {
        let (text_meta, pipelines, font_texture_cache, views, texts) = self.params.get(world);
        let view_uniform = views.get(view).unwrap();
        let text_meta = text_meta.into_inner();
        let extracted_text = texts.get(item.entity).unwrap();
        if let Some(pipeline) = pipelines.into_inner().get(item.pipeline) {
            pass.set_render_pipeline(pipeline);
            pass.set_vertex_buffer(0, text_meta.vertices.buffer().unwrap().slice(..));
            pass.set_bind_group(
                0,
                text_meta.view_bind_group.as_ref().unwrap(),
                &[view_uniform.offset],
            );

            if let Some(image_bindings) = font_texture_cache
                .into_inner()
                .bind_groups
                .get(&extracted_text.font_handle)
            {
                pass.set_bind_group(1, image_bindings, &[]);

                pass.draw(
                    (extracted_text.vertex_index * QUAD_VERTEX_POSITIONS.len()) as u32
                        ..((extracted_text.vertex_index + 1) * QUAD_VERTEX_POSITIONS.len()) as u32,
                    0..1,
                );
            }
        }
    }
}
