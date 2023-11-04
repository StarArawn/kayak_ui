use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
            TextureView,
        },
        texture::BevyDefault,
        view::ViewTarget,
    },
    utils::HashMap,
    window::Window,
};

/// Keeps track of opacity layer textures.
#[derive(Resource, Clone, Default)]
pub struct OpacityLayerManager {
    pub camera_layers: HashMap<Entity, OpacityCamera>,
}

impl OpacityLayerManager {
    pub(crate) fn add_or_update(
        &mut self,
        camera_entity: &Entity,
        camera: &Camera,
        window: &Window,
        images: &mut Assets<Image>,
    ) {
        if let Some(opacity_camera) = self.camera_layers.get_mut(camera_entity) {
            opacity_camera.update_images(window, camera, images);
        } else {
            self.camera_layers
                .insert(*camera_entity, OpacityCamera::new(window, camera, images));
        }
    }
}

#[derive(Clone, Debug)]
pub struct OpacityCamera {
    layers: HashMap<u32, (Extent3d, Handle<Image>)>,
    views: HashMap<u32, TextureView>,
}

pub const MAX_OPACITY_LAYERS: u32 = 5;

impl OpacityCamera {
    /// Creates as new opacity layer render target manager
    pub(crate) fn new(window: &Window, camera: &Camera, images: &mut Assets<Image>) -> Self {
        let main_texture_format = if camera.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let mut layers = HashMap::default();
        for layer in 1..MAX_OPACITY_LAYERS {
            let size = Extent3d {
                width: window.resolution.physical_width(),
                height: window.resolution.physical_height(),
                ..Default::default()
            };
            // This is the texture that will be rendered to.
            let mut image = Image {
                texture_descriptor: TextureDescriptor {
                    label: None,
                    size,
                    dimension: TextureDimension::D2,
                    format: main_texture_format,
                    mip_level_count: 1,
                    sample_count: 1,
                    usage: TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                },
                ..Default::default()
            };
            image.resize(size);
            let image_handle = images.add(image);

            layers.insert(layer, (size, image_handle));
        }

        Self {
            layers,
            views: HashMap::default(),
        }
    }

    pub(crate) fn update_images(
        &mut self,
        window: &Window,
        camera: &Camera,
        images: &mut Assets<Image>,
    ) {
        let main_texture_format = if camera.hdr {
            ViewTarget::TEXTURE_FORMAT_HDR
        } else {
            TextureFormat::bevy_default()
        };

        let new_size = Extent3d {
            width: window.resolution.physical_width(),
            height: window.resolution.physical_height(),
            ..Default::default()
        };
        for (size, layer_handle) in self.layers.values_mut() {
            if *size != new_size {
                let layer_image = images.get_mut(layer_handle).unwrap();
                layer_image.texture_descriptor.format = main_texture_format;
                layer_image.resize(new_size);
                *size = new_size;
            }
        }
    }

    pub(crate) fn get_image_handle(&self, layer_id: u32) -> Handle<Image> {
        self.layers.get(&layer_id).unwrap().1.clone_weak()
    }

    pub(crate) fn set_texture_views(&mut self, gpu_images: &RenderAssets<Image>) {
        for (layer, image) in self.layers.iter() {
            if let Some(gpu_image) = gpu_images.get(&image.1) {
                self.views.insert(*layer, gpu_image.texture_view.clone());
            }
        }
    }
}
