//! A small collection of default widgets
//! These widgets can be useful as default widgets for debugging purposes.
//! Kayak recommends that you use these widgets as a guide for building your own widgets.
//! Some of the widgets are useful regardless. A list:
//!
//! - KayakApp
//! - Background
//! - Clip
//! - Element
//! - Image
//! - NinePatch
//! - TextBox
//! - Text
//! - Texture Atlas
//! - Scroll
//!
//! Widgets like:
//! - Window
//! - Button
//!
//! Should be a guide for creating your own set of widgets.

use bevy::prelude::*;

mod accordion;
mod app;
mod background;
mod button;
mod clip;
mod element;
mod icons;
mod image;
mod modal;
mod nine_patch;
mod scroll;
mod svg;
mod text;
mod text_box;
mod texture_atlas;
mod transition;
mod window;
mod window_context_provider;

pub use accordion::*;
pub use app::{KayakApp, KayakAppBundle};
pub use background::{Background, BackgroundBundle};
pub use button::{ButtonState, KButton, KButtonBundle};
pub use clip::{Clip, ClipBundle};
pub use element::{Element, ElementBundle};
pub use icons::*;
pub use image::{KImage, KImageBundle};
pub use modal::{Modal, ModalBundle};
pub use nine_patch::{NinePatch, NinePatchBundle};
pub use scroll::{
    scroll_bar::{ScrollBarBundle, ScrollBarProps},
    scroll_box::{ScrollBoxBundle, ScrollBoxProps},
    scroll_content::{ScrollContentBundle, ScrollContentProps},
    scroll_context::{
        ScrollContext, ScrollContextProvider, ScrollContextProviderBundle, ScrollMode,
    },
};
pub use svg::{KSvg, KSvgBundle, Svg};
pub use text::{TextProps, TextWidgetBundle};
pub use text_box::{TextBoxBundle, TextBoxProps, TextBoxState};
pub use texture_atlas::{TextureAtlasBundle, TextureAtlasProps};
pub use transition::{
    create_transition, Transition, TransitionBundle, TransitionEasing, TransitionProps,
    TransitionState,
};
pub use window::{KWindow, KWindowState, WindowBundle};
pub use window_context_provider::{
    WindowContext, WindowContextProvider, WindowContextProviderBundle,
};

use app::{app_render, app_update};
use background::background_render;
use button::button_render;
use clip::clip_render;
use element::element_render;
use image::image_render;
use nine_patch::nine_patch_render;
use scroll::{
    scroll_bar::scroll_bar_render, scroll_box::scroll_box_render,
    scroll_content::scroll_content_render, scroll_context::scroll_context_render,
};
use svg::svg_render;
use text::text_render;
use text_box::text_box_render;
use texture_atlas::texture_atlas_render;
use window::window_render;

use crate::{
    context::{update_widgets_sys, KayakRootContext},
    widget::{widget_update, widget_update_with_context, EmptyState, Widget},
    KayakUIPlugin,
};

use self::window_context_provider::window_context_render;

pub struct KayakWidgets;

impl Plugin for KayakWidgets {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(icons::IconsPlugin);
        app.add_system(
            transition::update_transitions
                .after(update_widgets_sys)
                .in_base_set(CoreSet::PostUpdate),
        )
        .add_system(text_box::cursor_animation_system);
    }
}

pub struct KayakWidgetsContextPlugin;

impl KayakUIPlugin for KayakWidgetsContextPlugin {
    fn build(&self, context: &mut KayakRootContext) {
        context.add_plugin(AccordionPlugin);
        context.add_widget_data::<KayakApp, EmptyState>();
        context.add_widget_data::<KButton, ButtonState>();
        context.add_widget_data::<TextProps, EmptyState>();
        context.add_widget_data::<KWindow, KWindowState>();
        context.add_widget_data::<WindowContextProvider, EmptyState>();
        context.add_widget_data::<Background, EmptyState>();
        context.add_widget_data::<Clip, EmptyState>();
        context.add_widget_data::<KImage, EmptyState>();
        context.add_widget_data::<TextureAtlasProps, EmptyState>();
        context.add_widget_data::<NinePatch, EmptyState>();
        context.add_widget_data::<KSvg, EmptyState>();
        context.add_widget_data::<Element, EmptyState>();
        context.add_widget_data::<ScrollBarProps, EmptyState>();
        context.add_widget_data::<ScrollContentProps, EmptyState>();
        context.add_widget_data::<ScrollBoxProps, EmptyState>();
        context.add_widget_data::<ScrollContextProvider, EmptyState>();
        context.add_widget_data::<TextBoxProps, TextBoxState>();
        context.add_widget_data::<TransitionProps, TransitionState>();
        context.add_widget_data::<Modal, TransitionState>();

        context.add_widget_system(KayakApp::default().get_name(), app_update, app_render);
        context.add_widget_system(
            KButton::default().get_name(),
            widget_update::<KButton, ButtonState>,
            button_render,
        );
        context.add_widget_system(
            TextProps::default().get_name(),
            widget_update::<TextProps, EmptyState>,
            text_render,
        );
        context.add_widget_system(
            WindowContextProvider::default().get_name(),
            widget_update::<WindowContextProvider, EmptyState>,
            window_context_render,
        );
        context.add_widget_system(
            KWindow::default().get_name(),
            widget_update_with_context::<KWindow, KWindowState, WindowContext>,
            window_render,
        );
        context.add_widget_system(
            Background::default().get_name(),
            widget_update::<Background, EmptyState>,
            background_render,
        );
        context.add_widget_system(
            Clip::default().get_name(),
            widget_update::<Clip, EmptyState>,
            clip_render,
        );
        context.add_widget_system(
            KImage::default().get_name(),
            widget_update::<KImage, EmptyState>,
            image_render,
        );
        context.add_widget_system(
            TextureAtlasProps::default().get_name(),
            widget_update::<TextureAtlasProps, EmptyState>,
            texture_atlas_render,
        );
        context.add_widget_system(
            NinePatch::default().get_name(),
            widget_update::<NinePatch, EmptyState>,
            nine_patch_render,
        );
        context.add_widget_system(
            KSvg::default().get_name(),
            widget_update::<KSvg, EmptyState>,
            svg_render,
        );
        context.add_widget_system(
            Element::default().get_name(),
            widget_update::<Element, EmptyState>,
            element_render,
        );
        context.add_widget_system(
            ScrollBarProps::default().get_name(),
            widget_update_with_context::<ScrollBarProps, EmptyState, ScrollContext>,
            scroll_bar_render,
        );
        context.add_widget_system(
            ScrollContentProps::default().get_name(),
            widget_update_with_context::<ScrollContentProps, EmptyState, ScrollContext>,
            scroll_content_render,
        );
        context.add_widget_system(
            ScrollBoxProps::default().get_name(),
            widget_update_with_context::<ScrollBoxProps, EmptyState, ScrollContext>,
            scroll_box_render,
        );
        context.add_widget_system(
            ScrollContextProvider::default().get_name(),
            widget_update::<ScrollContextProvider, EmptyState>,
            scroll_context_render,
        );
        context.add_widget_system(
            TextBoxProps::default().get_name(),
            widget_update::<TextBoxProps, TextBoxState>,
            text_box_render,
        );
        context.add_widget_system(
            TransitionProps::default().get_name(),
            widget_update::<TransitionProps, TransitionState>,
            transition::render,
        );
        context.add_widget_system(
            Modal::default().get_name(),
            widget_update::<Modal, TransitionState>,
            modal::render,
        );
    }
}
