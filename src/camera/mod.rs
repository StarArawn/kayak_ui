use bevy::{
    ecs::query::QueryItem,
    prelude::*,
    render::extract_component::{ExtractComponent, ExtractComponentPlugin},
};

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct CameraUIKayak;
impl ExtractComponent for CameraUIKayak {
    type Query = &'static Self;
    type Filter = With<Camera>;
    type Out = CameraUIKayak;

    fn extract_component(item: QueryItem<Self::Query>) -> Option<Self::Out> {
        Some(*item)
    }
}

pub struct KayakUICameraPlugin;
impl Plugin for KayakUICameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<CameraUIKayak>::default());
    }
}
