use bevy::{
    prelude::*,
    render::Extract,
    utils::{HashMap, HashSet},
};
use bevy_svg::prelude::Svg;

mod extract;
pub use extract::extract_svg;

#[derive(Resource, Default, Debug, Clone, Deref, DerefMut)]
pub struct RenderSvgs(pub HashMap<AssetId<Svg>, (Svg, Mesh)>);

pub fn extract_svg_asset(
    mut events: Extract<EventReader<AssetEvent<Svg>>>,
    svg_assets: Extract<Res<Assets<Svg>>>,
    mesh_assets: Extract<Res<Assets<Mesh>>>,
    mut render_assets: ResMut<RenderSvgs>,
) {
    let mut changed_assets = HashSet::default();
    for event in events.read() {
        match event {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => {
                changed_assets.insert(*id);
            }
            AssetEvent::Removed { id } => {
                changed_assets.remove(id);
                render_assets.remove(id);
            }
        }
    }

    for handle in changed_assets.drain() {
        if let Some(asset) = svg_assets.get(handle) {
            if let Some(mesh) = mesh_assets.get(&asset.mesh) {
                render_assets.insert(handle, (asset.clone(), mesh.clone()));
            }
        }
    }
}
