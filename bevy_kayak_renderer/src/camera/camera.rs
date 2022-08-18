use bevy::{
    ecs::query::QueryItem,
    prelude::{Bundle, Component, GlobalTransform, Transform, With},
    render::{
        camera::{Camera, CameraProjection, CameraRenderGraph, DepthCalculation, WindowOrigin},
        extract_component::ExtractComponent,
        primitives::Frustum,
        view::VisibleEntities,
    },
};

use super::ortho::UIOrthographicProjection;

#[derive(Component, Clone, Default)]
pub struct CameraUiKayak;

impl ExtractComponent for CameraUiKayak {
    type Query = &'static Self;
    type Filter = With<Camera>;

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

#[derive(Bundle)]
pub struct UICameraBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub orthographic_projection: UIOrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub marker: CameraUiKayak,
}

impl UICameraBundle {
    pub const UI_CAMERA: &'static str = "KAYAK_UI_CAMERA";
    pub fn new() -> Self {
        // we want 0 to be "closest" and +far to be "farthest" in 2d, so we offset
        // the camera's translation by far and use a right handed coordinate system
        let far = 1000.0;

        let orthographic_projection = UIOrthographicProjection {
            far,
            depth_calculation: DepthCalculation::ZDifference,
            window_origin: WindowOrigin::BottomLeft,
            ..Default::default()
        };

        let transform = Transform::from_xyz(0.0, 0.0, far - 0.1);

        let view_projection =
            orthographic_projection.get_projection_matrix() * transform.compute_matrix().inverse();
        let frustum = Frustum::from_view_projection(
            &view_projection,
            &transform.translation,
            &transform.back(),
            orthographic_projection.far(),
        );
        UICameraBundle {
            camera: Camera {
                priority: isize::MAX - 1,
                ..Default::default()
            },
            camera_render_graph: CameraRenderGraph::new(crate::render::draw_ui_graph::NAME),
            orthographic_projection,
            frustum,
            visible_entities: VisibleEntities::default(),
            transform,
            global_transform: Default::default(),
            marker: CameraUiKayak,
        }
    }
}
