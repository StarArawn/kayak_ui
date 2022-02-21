use bevy::{prelude::Handle, render::texture::Image, utils::HashMap};

/// A resource used to manage images for use in a `KayakContext`
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_kayak_ui::ImageManager;
///
/// fn setup_ui(
///   # mut commands: Commands,
///   asset_server: Res<AssetServer>,
///   mut image_manager: ResMut<ImageManager>
/// ) {
///   # commands.spawn_bundle(UICameraBundle::new());
///   #
///   let handle: Handle<Image> = asset_server.load("some-image.png");
///   let ui_image_handle = image_manager.get(&handle);
///   // ...
///   #
///   # let context = BevyContext::new(|context| {
///   #   render! {
///   #     <App>
///   #       <Image handle={ui_image_handle} />
///   #     </App>
///   #   }
///   # });
///   #
///   # commands.insert_resource(context);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ImageManager {
    /// The current number of tracked images (used for assigning IDs)
    count: u16,
    /// A map of image IDs to their _strong_ handle
    mapping: HashMap<u16, Handle<Image>>,
    /// A map of _weak_ image handles to their ID
    reverse_mapping: HashMap<Handle<Image>, u16>,
}

impl ImageManager {
    pub fn new() -> Self {
        Self {
            count: 0,
            mapping: HashMap::default(),
            reverse_mapping: HashMap::default(),
        }
    }

    /// Get the ID for the given handle
    ///
    /// If no handle is found, a _strong_ clone is made and added to the current mapping.
    /// The newly created ID is then returned.
    pub fn get(&mut self, image_handle: &Handle<Image>) -> u16 {
        if let Some(id) = self.reverse_mapping.get(image_handle) {
            return *id;
        } else {
            let id = self.count;
            self.count += 1;
            self.mapping.insert(id, image_handle.clone());
            self.reverse_mapping.insert(image_handle.clone_weak(), id);
            return id;
        }
    }

    /// Get the image handle for the given ID
    pub fn get_handle(&self, id: &u16) -> Option<&Handle<Image>> {
        self.mapping.get(id)
    }
}
