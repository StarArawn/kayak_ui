use crate::{
    context::{KayakRootContext, WidgetName},
    node::Node,
    CameraUIKayak,
};
use bevy::{
    prelude::*,
    render::{
        render_phase::RenderPhase,
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
        view::ColorGrading,
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
    window::{PrimaryWindow, Window, WindowRef},
};
use kayak_font::KayakFont;

use super::{font::FontMapping, ui_pass::TransparentUI, unified::pipeline::ExtractedQuads};

// mod nine_patch;
// mod texture_atlas;

pub struct BevyKayakUIExtractPlugin;

impl Plugin for BevyKayakUIExtractPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .init_resource::<UIViewUniforms>()
            .add_systems(
                ExtractSchedule,
                (
                    extract,
                    extract_default_ui_camera_view::<Camera2d>,
                    extract_default_ui_camera_view::<Camera3d>,
                ),
            )
            .add_systems(Render, prepare_view_uniforms.in_set(RenderSet::Prepare));
    }
}

pub fn extract(
    mut commands: Commands,
    context_query: Extract<Query<(Entity, &KayakRootContext)>>,
    fonts: Extract<Res<Assets<KayakFont>>>,
    font_mapping: Extract<Res<FontMapping>>,
    node_query: Extract<Query<&Node>>,
    widget_names: Extract<Query<&WidgetName>>,
    images: Extract<Res<Assets<Image>>>,
    primary_window: Extract<Query<&Window, With<PrimaryWindow>>>,
    cameras: Extract<Query<&Camera>>,
    mut extracted_quads: ResMut<ExtractedQuads>,
) {
    extracted_quads.quads.clear();

    for (_entity, context) in context_query.iter() {
        let dpi = if let Ok(camera) = cameras.get(context.camera_entity) {
            if let bevy::render::camera::RenderTarget::Window(WindowRef::Primary) = &camera.target {
                if let Ok(window) = primary_window.get_single() {
                    window.scale_factor() as f32
                } else {
                    1.0
                }
            } else {
                1.0
            }
        } else {
            1.0
        };

        context.build_render_primitives(
            &mut commands,
            context.camera_entity,
            dpi,
            &node_query,
            &widget_names,
            &fonts,
            &font_mapping,
            &images,
            &mut extracted_quads,
        );
    }
}

const UI_CAMERA_TRANSFORM_OFFSET: f32 = -0.1;

#[derive(Component)]
pub struct UIExtractedView {
    pub projection: Mat4,
    pub transform: GlobalTransform,
    // The view-projection matrix. When provided it is used instead of deriving it from
    // `projection` and `transform` fields, which can be helpful in cases where numerical
    // stability matters and there is a more direct way to derive the view-projection matrix.
    pub view_projection: Option<Mat4>,
    pub hdr: bool,
    // uvec4(origin.x, origin.y, width, height)
    pub viewport: UVec4,
    pub color_grading: ColorGrading,
}

pub fn extract_default_ui_camera_view<T: Component>(
    mut commands: Commands,
    query: Extract<Query<(Entity, &Camera, &CameraUIKayak), With<T>>>,
) {
    for (entity, camera, _camera_ui) in &query {
        if let (Some(logical_size), Some((physical_origin, _)), Some(physical_size)) = (
            camera.logical_viewport_size(),
            camera.physical_viewport_rect(),
            camera.physical_viewport_size(),
        ) {
            // use a projection matrix with the origin in the top left instead of the bottom left that comes with OrthographicProjection
            let projection_matrix =
                Mat4::orthographic_rh(0.0, logical_size.x, logical_size.y, 0.0, 0.0, 1000.0);
            commands.get_or_spawn(entity).insert((
                UIExtractedView {
                    projection: projection_matrix,
                    transform: GlobalTransform::from_xyz(
                        0.0,
                        0.0,
                        1000.0 + UI_CAMERA_TRANSFORM_OFFSET,
                    ),
                    hdr: camera.hdr,
                    viewport: UVec4::new(
                        physical_origin.x,
                        physical_origin.y,
                        physical_size.x,
                        physical_size.y,
                    ),
                    view_projection: None,
                    color_grading: ColorGrading::default(),
                },
                RenderPhase::<TransparentUI>::default(),
            ));
        }
    }
}

#[derive(Resource, Default)]
pub struct UIViewUniforms {
    pub uniforms: DynamicUniformBuffer<UIViewUniform>,
}

#[derive(Clone, ShaderType)]
pub struct UIViewUniform {
    pub view_proj: Mat4,
    pub unjittered_view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub view: Mat4,
    pub inverse_view: Mat4,
    pub projection: Mat4,
    pub inverse_projection: Mat4,
    pub world_position: Vec3,
    // viewport(x_origin, y_origin, width, height)
    pub viewport: Vec4,
    pub color_grading: ColorGrading,
    pub mip_bias: f32,
}

#[derive(Component, Debug)]
pub struct UIViewUniformOffset {
    pub offset: u32,
}

use bevy::math::Vec4Swizzles;
use bevy::render::camera::{MipBias, TemporalJitter};

pub fn prepare_view_uniforms(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut view_uniforms: ResMut<UIViewUniforms>,
    views: Query<(
        Entity,
        &UIExtractedView,
        Option<&TemporalJitter>,
        Option<&MipBias>,
    )>,
) {
    view_uniforms.uniforms.clear();
    for (entity, camera, temporal_jitter, mip_bias) in &views {
        let viewport = camera.viewport.as_vec4();
        let unjittered_projection = camera.projection;
        let mut projection = unjittered_projection;

        if let Some(temporal_jitter) = temporal_jitter {
            temporal_jitter.jitter_projection(&mut projection, viewport.zw());
        }

        let inverse_projection = projection.inverse();
        let view = camera.transform.compute_matrix();
        let inverse_view = view.inverse();
        let view_uniforms = UIViewUniformOffset {
            offset: view_uniforms.uniforms.push(UIViewUniform {
                view_proj: camera
                    .view_projection
                    .unwrap_or_else(|| projection * inverse_view),
                unjittered_view_proj: unjittered_projection * inverse_view,
                inverse_view_proj: view * inverse_projection,
                view,
                inverse_view,
                projection,
                inverse_projection,
                world_position: camera.transform.translation(),
                viewport,
                color_grading: camera.color_grading,
                mip_bias: mip_bias.unwrap_or(&MipBias(0.0)).0,
            }),
        };
        commands.entity(entity).insert(view_uniforms);
    }

    view_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}
