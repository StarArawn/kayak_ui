use bevy::{prelude::Handle, reflect::TypeUuid, render2::texture::Image};
use kayak_font::Font;

use super::sdf::Sdf;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub sdf: Option<Sdf>,
    pub atlas_image: Option<Handle<Image>>,
    pub font: Font,
}

// impl KayakFont {
//     pub fn from_atlas(atlas: Texture, )
// }
