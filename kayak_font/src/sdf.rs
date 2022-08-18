use crate::{atlas::Atlas, glyph::Glyph, metrics::Metrics};
use nanoserde::DeJson;

#[derive(DeJson, Debug, Clone, PartialEq)]
pub struct Sdf {
    pub atlas: Atlas,
    metrics: Metrics,
    pub glyphs: Vec<Glyph>,
    kerning: Vec<KerningData>,
}

#[derive(DeJson, Debug, Clone, Copy, PartialEq)]
pub struct KerningData {
    pub unicode1: u32,
    pub unicode2: u32,
    pub advance: f32,
}

impl Sdf {
    pub fn from_string(data: String) -> Sdf {
        let value: Sdf = match DeJson::deserialize_json(data.as_str()) {
            Ok(v) => v,
            Err(err) => {
                panic!("{}", dbg!(err));
            }
        };

        value
    }

    pub fn from_bytes(data: &[u8]) -> Sdf {
        let value: Sdf = match DeJson::deserialize_json(std::str::from_utf8(data).unwrap()) {
            Ok(v) => v,
            Err(err) => {
                panic!("{}", dbg!(err));
            }
        };

        value
    }

    pub fn max_glyph_size(&self) -> (f32, f32) {
        let mut size = (0.0, 0.0);

        self.glyphs.iter().for_each(|glyph| {
            if let Some(atlas_bounds) = glyph.atlas_bounds {
                let atlas_size = atlas_bounds.size();
                if atlas_size.0 > size.0 {
                    size.0 = atlas_size.0;
                }
                if atlas_size.1 > size.1 {
                    size.1 = atlas_size.1;
                }
            }
        });

        size
    }
}

#[test]
fn test_sdf_loader() {
    use crate::SDFType;
    let sdf = Sdf::from_string(include_str!("../assets/roboto.kayak_font").to_string());
    assert!(sdf.max_glyph_size() == (30.0, 36.0));
    assert!(sdf.atlas.width == 212);
    assert!(sdf.atlas.height == 212);
    assert!(matches!(sdf.atlas.sdf_type, SDFType::Msdf));
}
