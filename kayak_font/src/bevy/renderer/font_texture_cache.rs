use crate::{ImageType, KayakFont, Sdf};
use bevy::{
    math::Vec2,
    prelude::{Handle, Res, Resource},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, BindGroupLayout, CommandEncoderDescriptor, Extent3d, FilterMode,
            ImageCopyTexture, Origin3d, SamplerDescriptor, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{GpuImage, Image},
    },
    utils::HashMap,
};

pub trait FontRenderingPipeline {
    fn get_font_image_layout(&self) -> &BindGroupLayout;
}

pub const MAX_CHARACTERS: u32 = 500;

#[derive(Resource)]
pub struct FontTextureCache {
    images: HashMap<Handle<KayakFont>, GpuImage>,
    fonts: HashMap<Handle<KayakFont>, KayakFont>,
    new_fonts: Vec<Handle<KayakFont>>,
    updated_fonts: Vec<Handle<KayakFont>>,
}

impl Default for FontTextureCache {
    fn default() -> Self {
        Self::new()
    }
}

impl FontTextureCache {
    pub fn new() -> Self {
        Self {
            images: HashMap::default(),
            fonts: HashMap::default(),
            new_fonts: Vec::new(),
            updated_fonts: Vec::new(),
        }
    }

    pub fn add(&mut self, kayak_font_handle: Handle<KayakFont>, font: KayakFont) {
        if !self.fonts.contains_key(&kayak_font_handle) {
            self.fonts.insert(kayak_font_handle.clone(), font);
            self.new_fonts.push(kayak_font_handle);
        } else if let Some(old_font) = self.fonts.get_mut(&kayak_font_handle) {
            *old_font = font;
            self.updated_fonts.push(kayak_font_handle);
        }
    }

    pub fn get_gpu_image<'s>(
        &'s self,
        handle: &Handle<KayakFont>,
        render_images: &'s RenderAssets<Image>,
    ) -> Option<&'s GpuImage> {
        if let Some(gpu_image) = self.images.get(handle) {
            Some(gpu_image)
        } else if let Some(font) = self.fonts.get(handle) {
            render_images.get(font.image.get())
        } else {
            None
        }
    }

    pub fn process_new(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        render_images: &Res<RenderAssets<Image>>,
    ) {
        let new_fonts: Vec<_> = self.new_fonts.drain(..).collect();
        for kayak_font_handle in new_fonts {
            let mut was_processed = true;
            if let Some(font) = self.fonts.get(&kayak_font_handle) {
                if matches!(font.image, ImageType::Array(..)) {
                    if render_images.get(font.image.get()).is_none() {
                        was_processed = false;
                    }
                } else if let Some(atlas_texture) = render_images.get(font.image.get()) {
                    Self::create_from_atlas(
                        &mut self.images,
                        &font.sdf,
                        kayak_font_handle.clone_weak(),
                        device,
                        queue,
                        atlas_texture,
                        font.sdf.max_glyph_size().into(),
                    );
                } else {
                    was_processed = false;
                }
            }
            if !was_processed {
                self.new_fonts.push(kayak_font_handle.clone_weak());
            }
        }
    }

    fn create_texture(
        images: &mut HashMap<Handle<KayakFont>, GpuImage>,
        font_handle: Handle<KayakFont>,
        size: (u32, u32),
        device: &RenderDevice,
        format: TextureFormat,
    ) {
        let texture_descriptor = TextureDescriptor {
            label: Some("font_texture_array"),
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: MAX_CHARACTERS,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            view_formats: &[format],
            format,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };

        let sampler_descriptor = SamplerDescriptor {
            label: Some("font_texture_array_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: std::f32::MAX,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        };

        let texture = device.create_texture(&texture_descriptor);
        let sampler = device.create_sampler(&sampler_descriptor);

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("font_texture_array_view"),
            format: None,
            dimension: Some(TextureViewDimension::D2Array),
            aspect: bevy::render::render_resource::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            mip_level_count: None,
            array_layer_count: std::num::NonZeroU32::new(MAX_CHARACTERS),
        });

        let image = GpuImage {
            texture,
            sampler,
            texture_view,
            mip_level_count: 1,
            size: Vec2 {
                x: size.0 as f32,
                y: size.1 as f32,
            },
            texture_format: format,
        };

        images.insert(font_handle, image);
    }

    pub fn get_empty(device: &RenderDevice) -> GpuImage {
        let texture_descriptor = TextureDescriptor {
            label: Some("font_texture_array"),
            size: Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: MAX_CHARACTERS,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            view_formats: &[TextureFormat::Rgba8Unorm],
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        };

        let sampler_descriptor = SamplerDescriptor::default();

        let texture = device.create_texture(&texture_descriptor);
        let sampler = device.create_sampler(&sampler_descriptor);

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("font_texture_array_view"),
            format: None,
            dimension: Some(TextureViewDimension::D2Array),
            aspect: bevy::render::render_resource::TextureAspect::All,
            base_mip_level: 0,
            base_array_layer: 0,
            mip_level_count: None,
            array_layer_count: std::num::NonZeroU32::new(MAX_CHARACTERS),
        });

        GpuImage {
            texture,
            sampler,
            texture_view,
            mip_level_count: 1,
            size: Vec2 { x: 1.0, y: 1.0 },
            texture_format: TextureFormat::Rgba8Unorm,
        }
    }

    pub fn create_from_atlas(
        images: &mut HashMap<Handle<KayakFont>, GpuImage>,
        sdf: &Sdf,
        font_handle: Handle<KayakFont>,
        device: &RenderDevice,
        queue: &RenderQueue,
        atlas_texture: &GpuImage,
        size: Vec2,
    ) {
        Self::create_texture(
            images,
            font_handle.clone_weak(),
            (size.x as u32, size.y as u32),
            device,
            TextureFormat::Rgba8Unorm,
        );

        let mut command_encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("create_sdf_from_atlas_encoder"),
        });

        let gpu_image = images.get(&font_handle).unwrap();

        // Now fill the texture data.

        let _atlas_width = sdf.atlas.width;
        let atlas_height = sdf.atlas.height;

        for (i, glyph) in sdf.glyphs.iter().enumerate() {
            if let Some(atlas_bounds) = glyph.atlas_bounds {
                let glyph_size = atlas_bounds.size();
                command_encoder.copy_texture_to_texture(
                    ImageCopyTexture {
                        texture: &atlas_texture.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: atlas_bounds.left as u32,
                            y: atlas_height - atlas_bounds.top as u32,
                            z: 0,
                        },
                        aspect: TextureAspect::All,
                    },
                    ImageCopyTexture {
                        texture: &gpu_image.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
                        aspect: TextureAspect::All,
                    },
                    Extent3d {
                        width: glyph_size.0 as u32,
                        height: glyph_size.1 as u32,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        let command_buffer = command_encoder.finish();
        queue.submit(vec![command_buffer]);
    }
}
