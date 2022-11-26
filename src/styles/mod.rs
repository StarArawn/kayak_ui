use bevy::{prelude::Component, reflect::Reflect};

mod corner;
mod edge;
mod options_ref;
mod render_command;
mod style;
mod units;

pub use corner::Corner;
pub use edge::Edge;
pub use options_ref::AsRefOption;
pub use render_command::RenderCommand;
pub use style::*;
pub use units::*;

#[derive(Component, Reflect, Debug, Default, Clone, PartialEq)]
pub struct ComputedStyles(pub KStyle);

impl Into<ComputedStyles> for KStyle {
    fn into(self) -> ComputedStyles {
        ComputedStyles(self)
    }
}
