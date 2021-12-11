use std::collections::HashMap;


use bevy::{prelude::Handle, reflect::TypeUuid, render2::texture::Image};

use crate::Sdf;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Sdf,
    pub atlas_image: Handle<Image>,
    char_ids: HashMap<char, u32>,
}

impl KayakFont {
    pub fn new(sdf: Sdf, atlas_image: Handle<Image>) -> Self {
        Self {
            sdf,
            atlas_image,
            char_ids: HashMap::default(),
        }
    }

    pub fn generate_char_ids(&mut self) {
        let mut count = 0;
        for glyph in self.sdf.glyphs.iter() {
            self.char_ids.insert(glyph.unicode, count);
            count += 1;
        }
    }

    pub fn get_char_id(&self, c: char) -> Option<u32> {
        self.char_ids.get(&c).and_then(|id| Some(*id))
    }
}
