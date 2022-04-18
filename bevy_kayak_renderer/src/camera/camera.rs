use bevy::{
    prelude::{Bundle, Component, GlobalTransform, Transform},
    render::{
        camera::{Camera, CameraProjection, DepthCalculation, WindowOrigin},
        primitives::Frustum,
        view::VisibleEntities,
    },
};

use super::ortho::UIOrthographicProjection;

#[derive(Component, Default)]
pub struct CameraUiKayak;

#[derive(Bundle)]
pub struct UICameraBundle {
    pub camera: Camera,
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
            camera: Default::default(),
            orthographic_projection,
            frustum,
            visible_entities: VisibleEntities::default(),
            transform,
            global_transform: Default::default(),
            marker: CameraUiKayak,
        }
    }
}
