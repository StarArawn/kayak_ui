use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::Component;
use bevy::{
    math::Mat4,
    reflect::Reflect,
    render::camera::{CameraProjection, ScalingMode, WindowOrigin},
};

/// Kayak UI's default orthographic projection matrix
/// This matrix uses top left as 0, 0
/// and bottom right as width, height.
/// This projection layout is typical for most UI systems.
#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component)]
pub struct UIOrthographicProjection {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
    pub window_origin: WindowOrigin,
    pub scaling_mode: ScalingMode,
    pub scale: f32,
}

impl CameraProjection for UIOrthographicProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh(
            self.left * self.scale,
            self.right * self.scale,
            self.bottom * self.scale,
            self.top * self.scale,
            // NOTE: near and far are swapped to invert the depth range from [0,1] to [1,0]
            // This is for interoperability with pipelines using infinite reverse perspective projections.
            self.far,
            self.near,
        )
    }

    fn update(&mut self, width: f32, height: f32) {
        match (&self.scaling_mode, &self.window_origin) {
            (ScalingMode::WindowSize, WindowOrigin::BottomLeft) => {
                self.left = 0.0;
                self.right = width;
                self.top = 0.0;
                self.bottom = height;
            }
            _ => {}
        }
    }

    fn far(&self) -> f32 {
        self.far
    }
}

impl Default for UIOrthographicProjection {
    fn default() -> Self {
        UIOrthographicProjection {
            left: -1.0,
            right: 1.0,
            bottom: -1.0,
            top: 1.0,
            near: 0.0,
            far: 1000.0,
            window_origin: WindowOrigin::Center,
            scaling_mode: ScalingMode::WindowSize,
            scale: 1.0,
        }
    }
}
