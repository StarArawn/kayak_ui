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

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        item.clone()
    }
}

pub struct KayakUICameraPlugin;
impl Plugin for KayakUICameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ExtractComponentPlugin::<CameraUIKayak>::default());
    }
}
