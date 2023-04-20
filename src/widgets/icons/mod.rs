use std::path::Path;

use bevy::{prelude::{Plugin, HandleUntyped, Assets, Mesh}, reflect::TypeUuid};
use bevy_svg::prelude::Svg;

pub const EXPAND_LESS_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Svg::TYPE_UUID, 4238701051302568451);

pub const EXPAND_MORE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Svg::TYPE_UUID, 9116091369991258337);

pub struct IconsPlugin;
impl Plugin for IconsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let expand_less_bytes = include_bytes!("expand_less.svg");
        let expand_more_bytes = include_bytes!("expand_more.svg");
        let mut expand_less = Svg::from_bytes(expand_less_bytes, &Path::new("")).unwrap();
        let mut expand_more = Svg::from_bytes(expand_more_bytes, &Path::new("")).unwrap();

        let mut meshes = app.world.get_resource_mut::<Assets<Mesh>>().unwrap();
        expand_less.mesh = meshes.add(expand_less.tessellate());
        expand_more.mesh = meshes.add(expand_more.tessellate());

        let mut svgs = app.world.get_resource_mut::<Assets<Svg>>().unwrap();
        svgs.set_untracked(EXPAND_LESS_HANDLE, expand_less);
        svgs.set_untracked(EXPAND_MORE_HANDLE, expand_more);
    }
}
