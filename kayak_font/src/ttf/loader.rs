#![allow(clippy::needless_question_mark, clippy::question_mark)]
use bevy::{
    asset::{
        io::{AssetSourceBuilders, Reader},
        AssetLoader, AssetServer, AsyncReadExt, LoadContext,
    },
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureFormat},
    },
    utils::{BoxedFuture, HashMap},
};

use thiserror::Error;

use image::{EncodableLayout, RgbaImage};
use nanoserde::DeJson;

use crate::{
    msdf::{self, bitmap::FloatRGBBmp, shape::Shape, ttf_parser::ShapeBuilder, vector::Vector2},
    Glyph, ImageType, KayakFont, Rect, Sdf,
};
#[derive(Default)]
pub struct TTFLoader;

/// Possible errors that can be produced by [`CustomAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TTFLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(DeJson, Default, Debug, Clone)]
pub struct Kttf {
    file: String,
    char_range_start: String,
    char_range_end: String,
    offset_x: Option<f32>,
    offset_y: Option<f32>,
}

impl AssetLoader for TTFLoader {
    type Asset = KayakFont;

    type Settings = ();

    type Error = TTFLoaderError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a (),
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let kttf: Kttf =
                nanoserde::DeJson::deserialize_json(std::str::from_utf8(&bytes).unwrap()).unwrap();

            let char_range_start =
                u32::from_str_radix(kttf.char_range_start.trim_start_matches("0x"), 16).unwrap();
            let char_range_end =
                u32::from_str_radix(kttf.char_range_end.trim_start_matches("0x"), 16).unwrap();
            let font_bytes = load_context
                .read_asset_bytes(kttf.file.clone())
                .await
                .unwrap();

            let mut cache_path = std::path::PathBuf::from(load_context.path());
            let file_name = load_context
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            cache_path.set_file_name(format!("{}-cached.png", file_name));
            let cache_image = load_context.read_asset_bytes(cache_path.clone()).await;

            let font_range = char_range_start..char_range_end;
            let char_count = font_range.len() as u32;

            let size_x = 64usize;
            let size_y = 128usize;
            let face = ttf_parser::Face::parse(&font_bytes, 0).unwrap();
            let image_height = size_y as u32 * char_count;
            let mut image_builder: RgbaImage = image::ImageBuffer::new(size_x as u32, image_height);
            let mut yy = 0u32;
            let mut glyphs = vec![];

            // Build char to glyph mapping..
            let mut glyph_to_char: HashMap<ttf_parser::GlyphId, char> =
                HashMap::with_capacity(face.number_of_glyphs() as usize);
            let mut char_to_glyph: HashMap<char, ttf_parser::GlyphId> =
                HashMap::with_capacity(face.number_of_glyphs() as usize);
            if let Some(subtable) = face.tables().cmap {
                for subtable in subtable.subtables {
                    subtable.codepoints(|codepoint| {
                        if let Some(mapping) = subtable.glyph_index(codepoint) {
                            glyph_to_char.insert(mapping, std::char::from_u32(codepoint).unwrap());
                            char_to_glyph.insert(std::char::from_u32(codepoint).unwrap(), mapping);
                        }
                    })
                }
            }

            let loaded_file = &kttf;

            for char_u in font_range {
                let c = char::from_u32(char_u).unwrap();
                let glyph_id = char_to_glyph.get(&c);
                if glyph_id.is_none() {
                    continue;
                }
                let glyph_id = *glyph_id.unwrap();
                let mut output = FloatRGBBmp::new(size_x, size_y);
                let mut builder = ShapeBuilder::default();
                let pixel_scale = size_x as f64 / face.units_per_em() as f64;
                builder.pixel_scale = pixel_scale;
                let _result = face.outline_glyph(glyph_id, &mut builder);

                let char_bounds = face
                    .glyph_bounding_box(glyph_id)
                    .unwrap_or(ttf_parser::Rect {
                        x_min: 0,
                        x_max: size_x as i16,
                        y_min: 0,
                        y_max: size_y as i16,
                    });

                let mut shape = builder.build();
                shape.inverse_y_axis = true;
                // let (left, bottom, right, top) = shape.get_bounds();

                let scale = Vector2::new(1.0, 1.0);
                let px_range = 8.0;
                let range = px_range / scale.x.min(scale.y);

                let (translation, plane) = calculate_plane(
                    loaded_file,
                    &mut shape,
                    pixel_scale as f32,
                    1.0,
                    px_range as f32,
                    1.0,
                );
                let advance = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f32 / size_x as f32;
                let c = *glyph_to_char.get(&glyph_id).unwrap();
                glyphs.push(Glyph {
                    unicode: c,
                    advance: advance * pixel_scale as f32,
                    atlas_bounds: Some(Rect {
                        left: 0.0,
                        bottom: 0.0,
                        right: size_x as f32,
                        top: size_y as f32,
                    }),
                    plane_bounds: Some(plane),
                });

                // let frame = Vector2::new(size_x as f64, size_y as f64);

                // dbg!((left, right, top, bottom));

                // left = (left - (size_x as f64 / 8.0)).max(0.0);
                // right = (right + (size_x as f64 / 8.0)).min(size_x as f64);
                // top = (top + (size_y as f64 / 8.0)).min(size_y as f64);
                // bottom = (bottom - (size_y as f64 / 8.0)).max(0.0);

                // dbg!((left, right, top, bottom));

                // let dims = Vector2::new(right - left, top - bottom);

                // let translate = Vector2::new(-left + (frame.x - dims.x), (frame.y - (bottom + dims.y)) - 1.0);
                if cache_image.is_err() {
                    msdf::edge_coloring::simple(&mut shape, 3.0, 0);
                    msdf::gen::generate_msdf(
                        &mut output,
                        &shape,
                        range,
                        scale,
                        translation + Vector2::new(0.0, size_x as f64 * 1.25),
                        1.111_111_111_111_111_2,
                    );

                    // let left = (translation.x - char_bounds.x_min as f64 * pixel_scale).max(0.0).floor() as u32;
                    let right =
                        (translation.x + char_bounds.x_max as f64 * pixel_scale).floor() as u32;
                    // let top = (translation.y - char_bounds.y_min as f64 * pixel_scale).max(0.0).floor() as u32;
                    let bottom =
                        (translation.y + char_bounds.y_max as f64 * pixel_scale).floor() as u32;

                    for x in 0..(right + 2).min(64) {
                        for y in 0..bottom + 48 {
                            // for x in 0..size_x as u32 {
                            //     for y  in 0..size_y as u32 {
                            let pixel = output.get_pixel(x as usize, y as usize);
                            image_builder.put_pixel(
                                x,
                                yy + y,
                                image::Rgba([
                                    (pixel.r * 255.0) as u8,
                                    (pixel.g * 255.0) as u8,
                                    (pixel.b * 255.0) as u8,
                                    255,
                                ]),
                            );
                        }
                    }
                }
                // if c == '\"' {
                //     image_builder.save("test.png").unwrap();
                //     panic!("");
                // }
                yy += size_y as u32;
            }

            let image_bytes = match cache_image {
                Ok(cache_image) => {
                    let image = image::load_from_memory(&cache_image).unwrap();
                    image.as_bytes().to_vec()
                }
                Err(_) => {
                    #[cfg(not(target_family = "wasm"))]
                    {
                        let mut sources = AssetSourceBuilders::default();
                        sources.init_default_source("assets", None);
                        let fake_server = AssetServer::new(
                            sources.build_sources(false, false),
                            bevy::asset::AssetServerMode::Unprocessed,
                            false,
                        );
                        let writer = fake_server
                            .get_source(load_context.asset_path().source().clone())
                            .unwrap()
                            .writer()
                            .unwrap();
                        let mut cursor = std::io::Cursor::new(Vec::new());
                        image_builder
                            .write_to(&mut cursor, image::ImageOutputFormat::Png)
                            .unwrap();

                        writer
                            .write_bytes(cache_path.as_path(), cursor.get_ref())
                            .await
                            .unwrap();
                    }
                    image_builder.as_bytes().to_vec()
                }
            };

            let mut sdf = Sdf::default();
            sdf.glyphs = glyphs;
            sdf.atlas.font_size = size_x as f32;

            let mut image = bevy::prelude::Image::new(
                Extent3d {
                    width: size_x as u32,
                    height: image_height,
                    depth_or_array_layers: 1,
                },
                bevy::render::render_resource::TextureDimension::D2,
                image_bytes,
                TextureFormat::Rgba8Unorm,
                RenderAssetUsages::all(),
            );
            image.reinterpret_stacked_2d_as_array(char_count);
            let labeled_asset = load_context.begin_labeled_asset();
            let loaded_image_asset = labeled_asset.finish(image, None);
            let image_asset =
                load_context.add_loaded_labeled_asset("font_image", loaded_image_asset);

            let font = KayakFont::new(sdf, ImageType::Array(image_asset));

            Ok(font)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["kttf"]
    }
}

fn calculate_plane(
    loaded_file: &Kttf,
    shape: &mut Shape,
    geometry_scale: f32,
    scale: f32,
    _range: f32,
    miter_limit: f32,
) -> (Vector2, Rect) {
    let bounds = shape.get_bounds();
    let bounds = Rect {
        left: bounds.0 as f32,
        bottom: bounds.1 as f32,
        right: bounds.2 as f32,
        top: bounds.3 as f32,
    };
    let scale = scale * geometry_scale;
    let range = 1.0 / geometry_scale; //range / geometry_scale;
    let (_w, _h, translation_x, translation_y) =
        if bounds.left < bounds.right && bounds.bottom < bounds.top {
            let mut l = bounds.left as f64;
            let mut b = bounds.bottom as f64;
            let mut r = bounds.right as f64;
            let mut t = bounds.top as f64;

            l -= 0.5 * range as f64;
            b -= 0.5 * range as f64;
            r += 0.5 * range as f64;
            t += 0.5 * range as f64;

            if miter_limit > 0.0 {
                shape.bound_miters(
                    &mut l,
                    &mut b,
                    &mut r,
                    &mut t,
                    0.5 * range as f64,
                    miter_limit as f64,
                    1,
                );
            }

            let w = scale as f64 * (r - l);
            let h = scale as f64 * (t - b);
            let box_w = w.ceil() as i32 + 1;
            let box_h = h.ceil() as i32 + 1;
            (
                box_w,
                box_h,
                -l + 0.5 * (box_w as f64 - w) / scale as f64,
                -b + 0.5 * (box_h as f64 - h) / scale as f64,
            )
        } else {
            (0, 0, 0.0, 0.0)
        };

    // let mut l = 0.0;
    // let mut r = 0.0;
    // let mut b = 0.0;
    // let mut t = 0.0;
    // if w > 0 && h > 0 {
    //     let inv_box_scale = 1.0 / scale as f64;
    //     l = geometry_scale as f64 * (-translation_x + 0.5 * inv_box_scale);
    //     b = geometry_scale as f64 * (-translation_y + 0.5 * inv_box_scale);
    //     r = geometry_scale as f64 * (-translation_x + (w as f64 - 0.5) * inv_box_scale);
    //     t = geometry_scale as f64 * (-translation_y + (h as f64 - 0.5) * inv_box_scale);
    // }

    let left = loaded_file.offset_x.unwrap_or_default();
    let top = loaded_file.offset_y.unwrap_or_default();

    (
        Vector2::new(translation_x, translation_y) * geometry_scale as f64,
        Rect {
            left: left * geometry_scale, // l as f32,
            bottom: 0.0,                 // b as f32,
            right: 0.0,                  // r as f32,
            top: top * geometry_scale,   //0.0, // t as f32,
        },
    )
}
