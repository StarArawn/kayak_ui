use bevy::ecs::reflect::ReflectComponent;
use bevy::{
    math::Mat4,
    reflect::Reflect,
    render2::camera::{CameraProjection, DepthCalculation, ScalingMode, WindowOrigin},
};

#[derive(Debug, Clone, Reflect)]
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
    pub depth_calculation: DepthCalculation,
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

    fn depth_calculation(&self) -> DepthCalculation {
        self.depth_calculation
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
            depth_calculation: DepthCalculation::Distance,
        }
    }
}
