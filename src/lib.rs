#![feature(doc_auto_cfg)]

pub mod core {
    pub use kayak_core::*;
    pub use kayak_render_macros::{
        constructor, render, rsx, use_effect, use_state, widget, WidgetProps,
    };
}

#[cfg(feature = "bevy_renderer")]
pub mod font {
    pub use kayak_font::*;
}

#[cfg(feature = "bevy_renderer")]
pub mod bevy {
    pub use bevy_kayak_ui::*;
}

pub mod widgets;
