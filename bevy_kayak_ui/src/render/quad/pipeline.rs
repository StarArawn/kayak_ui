use bevy::{
    core::FloatOrd,
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemState,
    },
    math::{const_vec3, Mat4, Quat, Vec2, Vec3},
    prelude::{Bundle, Color, Commands, Entity, FromWorld, Query, Res, ResMut, World},
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
            TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState,
            VertexStepMode,
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
    render::{quad::QUAD_SHADER_HANDLE, ui_pass::TransparentUI},
    to_bevy_color, BevyContext,
};

pub struct QuadPipeline {
    view_layout: BindGroupLayout,
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

impl FromWorld for QuadPipeline {
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
            label: Some("quad_view_layout"),
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: 28,
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
            ],
        };

        let pipeline_desc = RenderPipelineDescriptor {
            vertex: VertexState {
                shader: QUAD_SHADER_HANDLE.typed::<Shader>(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: vec![vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                shader: QUAD_SHADER_HANDLE.typed::<Shader>(),
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
            layout: Some(vec![view_layout.clone()]),
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
                depth_write_enabled: true,
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
            label: Some("quad_pipeline".into()),
        };

        QuadPipeline {
            pipeline: pipeline_cache.queue(pipeline_desc),
            view_layout,
        }
    }
}

#[derive(Bundle)]
pub struct ExtractQuadBundle {
    extracted_quad: ExtractedQuad,
}

pub struct ExtractedQuad {
    rect: Rect,
    background_color: Color,
    vertex_index: usize,
}

pub fn extract_quads(mut commands: Commands, context: Res<BevyContext>) {
    let render_commands = if let Ok(context) = context.kayak_context.read() {
        context.widget_manager.build_render_primitives()
    } else {
        vec![]
    };

    let quad_commands: Vec<&RenderPrimitive> = render_commands
        .iter()
        .filter(|command| matches!(command, RenderPrimitive::Quad { .. }))
        .collect::<Vec<_>>();

    let mut extracted_quads = Vec::new();
    for render_primitive in quad_commands {
        let (background_color, layout) = match render_primitive {
            RenderPrimitive::Quad {
                background_color,
                layout,
            } => (background_color, layout),
            _ => panic!(""),
        };

        extracted_quads.push(ExtractQuadBundle {
            extracted_quad: ExtractedQuad {
                rect: Rect {
                    min: Vec2::new(layout.posx, layout.posy),
                    max: Vec2::new(layout.width, layout.height),
                },
                background_color: to_bevy_color(background_color),
                vertex_index: 0,
            },
        });
    }
    commands.spawn_batch(extracted_quads);
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct QuadVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
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
    mut extracted_sprites: Query<&mut ExtractedQuad>,
) {
    let extracted_sprite_len = extracted_sprites.iter_mut().len();
    // dont create buffers when there are no quads
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

        extracted_sprite.vertex_index = i;
        for vertex_position in QUAD_VERTEX_POSITIONS.iter() {
            let world = Mat4::from_scale_rotation_translation(
                sprite_rect.size().extend(1.0),
                Quat::default(),
                sprite_rect.min.extend(0.0),
            );
            let final_position = (world * Vec3::from(*vertex_position).extend(1.0)).truncate();
            sprite_meta.vertices.push(QuadVertex {
                position: final_position.into(),
                color,
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
    quad_pipeline: Res<QuadPipeline>,
    mut extracted_sprites: Query<(Entity, &ExtractedQuad)>,
    mut views: Query<&mut RenderPhase<TransparentUI>>,
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
        let draw_quad = draw_functions.read().get_id::<DrawQuad>().unwrap();
        for mut transparent_phase in views.iter_mut() {
            for (entity, quad) in extracted_sprites.iter_mut() {
                transparent_phase.add(TransparentUI {
                    draw_function: draw_quad,
                    pipeline: quad_pipeline.pipeline,
                    entity,
                    sort_key: FloatOrd(0.0),
                });
            }
        }
    }
}

pub struct DrawQuad {
    params: SystemState<(
        SRes<QuadMeta>,
        SRes<RenderPipelineCache>,
        SQuery<Read<ViewUniformOffset>>,
        SQuery<Read<ExtractedQuad>>,
    )>,
}

impl DrawQuad {
    pub fn new(world: &mut World) -> Self {
        Self {
            params: SystemState::new(world),
        }
    }
}

impl Draw<TransparentUI> for DrawQuad {
    fn draw<'w>(
        &mut self,
        world: &'w World,
        pass: &mut TrackedRenderPass<'w>,
        view: Entity,
        item: &TransparentUI,
    ) {
        let (quad_meta, pipelines, views, quads) = self.params.get(world);
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

            pass.draw(
                (extracted_quad.vertex_index * QUAD_VERTEX_POSITIONS.len()) as u32
                    ..((extracted_quad.vertex_index + 1) * QUAD_VERTEX_POSITIONS.len()) as u32,
                0..1,
            );
        }
    }
}
