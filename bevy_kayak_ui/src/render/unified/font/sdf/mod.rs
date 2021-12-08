use self::{atlas::Atlas, glyph::Glyph, metrics::Metrics};
use bevy::math::Vec2;
use serde::Deserialize;

mod atlas;
mod glyph;
mod metrics;

#[derive(Deserialize, Debug, Clone)]
pub struct Sdf {
    pub atlas: Atlas,
    metrics: Metrics,
    pub glyphs: Vec<Glyph>,
    kerning: Vec<KerningData>,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
pub struct KerningData {
    pub unicode1: u32,
    pub unicode2: u32,
    pub advance: f32,
}

impl Sdf {
    pub fn from_string(data: String) -> Sdf {
        let value: Sdf = match serde_path_to_error::deserialize(
            &mut serde_json::Deserializer::from_str(&data),
        ) {
            Ok(v) => v,
            Err(err) => {
                let path = err.path().to_string();
                dbg!(err);
                panic!("failed to deserialize json! path: {}", path);
            }
        };

        value
    }

    pub fn max_glyph_size(&self) -> Vec2 {
        let mut size = Vec2::new(0.0, 0.0);
        self.glyphs.iter().for_each(|glyph| {
            if let Some(atlas_bounds) = glyph.atlas_bounds {
                let atlas_size = atlas_bounds.size();
                if atlas_size.x > size.x {
                    size.x = atlas_size.x;
                }
                if atlas_size.y > size.y {
                    size.y = atlas_size.y;
                }
            }
        });

        size
    }
}

#[test]
fn test_sdf_loader() {
    let sdf = Sdf::from_string(include_str!("../../../../../../msdfgen/test.json").to_string());
    dbg!(sdf.max_glyph_size());
}
