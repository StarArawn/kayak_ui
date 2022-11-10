use bevy::{
    prelude::{Handle, Resource},
    utils::HashMap,
};
use kayak_font::KayakFont;

// use crate::context::Context;

/// A resource used to manage fonts for use in a `KayakContext`
///
/// # Example
///
/// ```
/// use bevy::prelude::*;
/// use bevy_kayak_ui::FontMapping;
///
/// fn setup_ui(
///   # mut commands: Commands,
///   asset_server: Res<AssetServer>,
///   mut font_mapping: ResMut<FontMapping>
/// ) {
///   # commands.spawn_bundle(UICameraBundle::new());
///   #
///   font_mapping.set_default(asset_server.load("roboto.kayak_font"));
///   // ...
///   #
///   # let context = BevyContext::new(|context| {
///   #   render! {
///   #     <App>
///   #       <Text content={"Hello World!".to_string()} />
///   #     </App>
///   #   }
///   # });
///   #
///   # commands.insert_resource(context);
/// }
/// ```
#[derive(Resource, Default)]
pub struct FontMapping {
    font_ids: HashMap<Handle<KayakFont>, String>,
    font_handles: HashMap<String, Handle<KayakFont>>,
    new_fonts: Vec<String>,
}

impl FontMapping {
    /// Add a `KayakFont` to be tracked
    pub fn add(&mut self, key: impl Into<String>, handle: Handle<KayakFont>) {
        let key = key.into();
        if !self.font_ids.contains_key(&handle) {
            self.font_ids.insert(handle.clone(), key.clone());
            self.new_fonts.push(key.clone());
            self.font_handles.insert(key, handle);
        }
    }

    /// Set a default `KayakFont`
    pub fn set_default(&mut self, handle: Handle<KayakFont>) {
        self.add(crate::DEFAULT_FONT, handle);
    }

    pub(crate) fn mark_all_as_new(&mut self) {
        self.new_fonts.extend(self.font_handles.keys().cloned());
    }

    /// Get the handle for the given font name
    pub fn get_handle(&self, id: String) -> Option<Handle<KayakFont>> {
        self.font_handles.get(&id).cloned()
    }

    /// Get the font name for the given handle
    pub fn get(&self, font: &Handle<KayakFont>) -> Option<String> {
        self.font_ids.get(font).cloned()
    }

    // pub(crate) fn add_loaded_to_kayak(
    //     &mut self,
    //     fonts: &Res<Assets<KayakFont>>,
    //     context: &Context,
    // ) {
    //     if let Ok(mut kayak_context) = context.kayak_context.write() {
    //         let new_fonts = self.new_fonts.drain(..).collect::<Vec<_>>();
    //         for font_key in new_fonts {
    //             let font_handle = self.font_handles.get(&font_key).unwrap();
    //             if let Some(font) = fonts.get(font_handle) {
    //                 kayak_context.set_asset(font_key, font.clone());
    //             } else {
    //                 self.new_fonts.push(font_key);
    //             }
    //         }
    //     }
    // }
}
