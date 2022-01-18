use bevy::{
    prelude::{Assets, Handle, Res},
    utils::HashMap,
};
use kayak_font::KayakFont;

use crate::BevyContext;

pub struct FontMapping {
    font_ids: HashMap<Handle<KayakFont>, String>,
    font_handles: HashMap<String, Handle<KayakFont>>,
    new_fonts: Vec<String>,
}

impl Default for FontMapping {
    fn default() -> Self {
        Self {
            font_ids: HashMap::default(),
            font_handles: HashMap::default(),
            new_fonts: Vec::new(),
        }
    }
}

impl FontMapping {
    pub fn add(&mut self, key: impl Into<String>, handle: Handle<KayakFont>) {
        let key = key.into();
        if !self.font_ids.contains_key(&handle) {
            self.font_ids.insert(handle.clone(), key.clone());
            self.new_fonts.push(key.clone());
            self.font_handles.insert(key, handle);
        }
    }

    pub fn get_handle(&self, id: String) -> Option<Handle<KayakFont>> {
        self.font_handles
            .get(&id)
            .and_then(|item| Some(item.clone()))
    }

    pub fn get(&self, font: &Handle<KayakFont>) -> Option<String> {
        self.font_ids
            .get(font)
            .and_then(|font_id| Some(font_id.clone()))
    }

    pub(crate) fn add_loaded_to_kayak(
        &mut self,
        fonts: &Res<Assets<KayakFont>>,
        context: &BevyContext,
    ) {
        if let Ok(mut kayak_context) = context.kayak_context.write() {
            let new_fonts = self.new_fonts.drain(..).collect::<Vec<_>>();
            for font_key in new_fonts {
                let font_handle = self.font_handles.get(&font_key).unwrap();
                if let Some(font) = fonts.get(font_handle) {
                    kayak_context.set_asset(font_key, font.clone());
                } else {
                    self.new_fonts.push(font_key);
                }
            }
        }
    }
}
