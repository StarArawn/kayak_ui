/// The core of Kayak UI, containing all required code
pub mod core {
    pub use kayak_core::*;
    pub use kayak_render_macros::{
        constructor, render, rsx, use_effect, use_state, widget, WidgetProps,
    };
}

/// Contains code related to loading and reading fonts in Kayak UI
#[cfg(feature = "bevy_renderer")]
pub mod font {
    pub use kayak_font::*;
}

/// Bevy-specific code for Bevy integration
#[cfg(feature = "bevy_renderer")]
pub mod bevy {
    pub use bevy_kayak_ui::*;
}

/// A convenient collection of built-in widgets
pub mod widgets;
