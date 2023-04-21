use crate::{
    context::{KayakRootContext, WidgetName},
    node::Node,
    render_primitive::RenderPrimitive,
    styles::Corner,
    CameraUIKayak,
};
use bevy::{
    prelude::{
        Assets, Camera, Camera2d, Camera3d, Color, Commands, Component, Entity, GlobalTransform,
        Image, IntoSystemAppConfig, IntoSystemAppConfigs, Mat4, Plugin, Query, Rect, Res, ResMut,
        UVec4, Vec2, With,
    },
    render::{
        render_phase::RenderPhase,
        view::{ColorGrading, ExtractedView},
        Extract, ExtractSchedule, RenderApp,
    },
    window::{PrimaryWindow, Window, WindowRef},
};
use kayak_font::KayakFont;

use super::{
    font::{self, FontMapping},
    image, nine_patch, texture_atlas,
    ui_pass::TransparentUI,
    unified::pipeline::{ExtractedQuad, ExtractedQuads, UIQuadType},
};

// mod nine_patch;
// mod texture_atlas;

pub struct BevyKayakUIExtractPlugin;

impl Plugin for BevyKayakUIExtractPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system(extract.in_schedule(ExtractSchedule));
        render_app.add_systems(
            (
                extract_default_ui_camera_view::<Camera2d>,
                extract_default_ui_camera_view::<Camera3d>,
            )
                .in_schedule(ExtractSchedule),
        );
    }
}

pub fn extract(
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
    let mut render_primitives = Vec::new();
    for (_entity, context) in context_query.iter() {
        let dpi = if let Ok(camera) = cameras.get(context.camera_entity) {
            match &camera.target {
                bevy::render::camera::RenderTarget::Window(window_ref) => match window_ref {
                    WindowRef::Primary => {
                        if let Ok(window) = primary_window.get_single() {
                            window.scale_factor() as f32
                        } else {
                            1.0
                        }
                    }
                    _ => 1.0,
                },
                _ => 1.0,
            }
        } else {
            1.0
        };
        let mut new_render_primitives = context.build_render_primitives(&node_query, &widget_names);
        render_primitives.extend(
            new_render_primitives
                .drain(..)
                .map(|r| (context.camera_entity, dpi, r)),
        );
    }

    for (camera_entity, dpi, render_primitive) in render_primitives {
        match render_primitive {
            RenderPrimitive::Text { .. } => {
                let text_quads = font::extract_texts(
                    camera_entity,
                    &render_primitive,
                    &fonts,
                    &font_mapping,
                    dpi,
                );
                extracted_quads.quads.extend(text_quads);
            }
            RenderPrimitive::Image { .. } => {
                let image_quads = image::extract_images(camera_entity, &render_primitive, dpi);
                extracted_quads.quads.extend(image_quads);
            }
            RenderPrimitive::Quad { .. } => {
                let quad_quads = super::quad::extract_quads(camera_entity, &render_primitive, 1.0);
                extracted_quads.quads.extend(quad_quads);
            }
            RenderPrimitive::NinePatch { .. } => {
                let nine_patch_quads =
                    nine_patch::extract_nine_patch(camera_entity, &render_primitive, &images, dpi);
                extracted_quads.quads.extend(nine_patch_quads);
            }
            RenderPrimitive::Svg { .. } => {
                extracted_quads.quads.push(super::svg::extract_svg(
                    camera_entity,
                    &render_primitive,
                    dpi,
                ));
            }
            RenderPrimitive::TextureAtlas { .. } => {
                let texture_atlas_quads = texture_atlas::extract_texture_atlas(
                    camera_entity,
                    &render_primitive,
                    &images,
                    dpi,
                );
                extracted_quads.quads.extend(texture_atlas_quads);
            }
            RenderPrimitive::Clip {
                layout,
                opacity_layer,
            } => {
                extracted_quads.quads.push(ExtractedQuad {
                    camera_entity,
                    rect: Rect {
                        min: Vec2::new(layout.posx, layout.posy) * dpi,
                        max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height)
                            * dpi,
                    },
                    color: Color::default(),
                    char_id: 0,
                    z_index: layout.z_index,
                    font_handle: None,
                    quad_type: UIQuadType::Clip,
                    type_index: 0,
                    border_radius: Corner::default(),
                    image: None,
                    uv_min: None,
                    uv_max: None,
                    opacity_layer,
                    ..Default::default()
                });
            }
            RenderPrimitive::OpacityLayer { index, z } => {
                extracted_quads.quads.push(ExtractedQuad {
                    camera_entity,
                    z_index: z,
                    quad_type: UIQuadType::OpacityLayer,
                    opacity_layer: index,
                    ..Default::default()
                });
            }
            RenderPrimitive::DrawOpacityLayer {
                opacity,
                index,
                z,
                layout,
            } => {
                extracted_quads.quads.push(ExtractedQuad {
                    camera_entity,
                    z_index: z,
                    color: Color::rgba(1.0, 1.0, 1.0, opacity),
                    opacity_layer: index,
                    quad_type: UIQuadType::DrawOpacityLayer,
                    rect: Rect {
                        min: Vec2::new(layout.posx, layout.posy),
                        max: Vec2::new(layout.posx + layout.width, layout.posy + layout.height),
                    },
                    ..Default::default()
                });
            }
            _ => {}
        }
    }
}

#[derive(Component)]
pub struct DefaultCameraView(pub Entity);

const UI_CAMERA_TRANSFORM_OFFSET: f32 = -0.1;

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
            let default_camera_view = commands
                .spawn(ExtractedView {
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
                })
                .id();
            commands.get_or_spawn(entity).insert((
                DefaultCameraView(default_camera_view),
                RenderPhase::<TransparentUI>::default(),
            ));
        }
    }
}
