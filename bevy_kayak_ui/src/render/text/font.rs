use bevy::reflect::TypeUuid;
use kayak_font::Font;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "4fe4732c-6731-49bb-bafc-4690d636b848"]
pub struct KayakFont {
    pub font: Font,
}
