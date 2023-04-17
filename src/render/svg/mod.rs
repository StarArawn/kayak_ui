use bevy::{
    prelude::*,
    render::Extract,
    utils::{HashMap, HashSet},
};
use bevy_svg::prelude::Svg;

mod extract;
pub use extract::extract_svg;

#[derive(Resource, Default, Debug, Clone, Deref, DerefMut)]
pub struct RenderSvgs(pub HashMap<Handle<Svg>, (Svg, Mesh)>);

pub fn extract_svg_asset(
    mut events: Extract<EventReader<AssetEvent<Svg>>>,
    svg_assets: Extract<Res<Assets<Svg>>>,
    mesh_assets: Extract<Res<Assets<Mesh>>>,
    mut render_assets: ResMut<RenderSvgs>,
) {
    let mut changed_assets = HashSet::default();
    for event in events.iter() {
        match event {
            AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
                changed_assets.insert(handle.clone_weak());
            }
            AssetEvent::Removed { handle } => {
                changed_assets.remove(handle);
                render_assets.remove(handle);
            }
        }
    }

    for handle in changed_assets.drain() {
        if let Some(asset) = svg_assets.get(&handle) {
            if let Some(mesh) = mesh_assets.get(&asset.mesh) {
                render_assets.insert(handle, (asset.clone(), mesh.clone()));
            }
        }
    }
}
