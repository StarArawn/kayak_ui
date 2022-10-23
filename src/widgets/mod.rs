use bevy::prelude::*;

mod app;
mod background;
mod button;
mod clip;
mod element;
mod image;
mod nine_patch;
mod scroll;
mod text;
mod text_box;
mod texture_atlas;
mod window;

pub use app::{KayakApp, KayakAppBundle};
pub use background::{Background, BackgroundBundle};
pub use button::{KButton, KButtonBundle};
pub use clip::{Clip, ClipBundle};
pub use element::{Element, ElementBundle};
pub use image::{Image, ImageBundle};
pub use nine_patch::{NinePatch, NinePatchBundle};
pub use scroll::{
    scroll_bar::{ScrollBarBundle, ScrollBarProps},
    scroll_box::{ScrollBoxBundle, ScrollBoxProps},
    scroll_content::{ScrollContentBundle, ScrollContentProps},
    scroll_context::{
        ScrollContext, ScrollContextProvider, ScrollContextProviderBundle, ScrollMode,
    },
};
pub use text::{TextProps, TextWidgetBundle};
pub use text_box::{TextBoxBundle, TextBoxProps, TextBoxState};
pub use texture_atlas::{TextureAtlas, TextureAtlasBundle};
pub use window::{KWindow, WindowBundle};

use app::{app_render, app_update};
use background::update_background;
use button::button_update;
use clip::update_clip;
use element::update_element;
use image::update_image;
use nine_patch::update_nine_patch;
use scroll::{
    scroll_bar::update_scroll_bar, scroll_box::update_scroll_box,
    scroll_content::update_scroll_content, scroll_context::update_scroll_context,
};
use text::text_render;
use text_box::update_text_box;
use texture_atlas::update_texture_atlas;
use window::window_update;

use crate::{
    context::Context,
    widget::{widget_update, widget_update_with_context, EmptyState, Widget},
};

pub struct KayakWidgets;

impl Plugin for KayakWidgets {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, add_widget_systems);
    }
}

fn add_widget_systems(mut context: ResMut<Context>) {
    context.add_widget_data::<KayakApp, EmptyState>();
    context.add_widget_data::<KButton, EmptyState>();
    context.add_widget_data::<TextProps, EmptyState>();
    context.add_widget_data::<KWindow, EmptyState>();
    context.add_widget_data::<Background, EmptyState>();
    context.add_widget_data::<Clip, EmptyState>();
    context.add_widget_data::<Image, EmptyState>();
    context.add_widget_data::<TextureAtlas, EmptyState>();
    context.add_widget_data::<NinePatch, EmptyState>();
    context.add_widget_data::<Element, EmptyState>();
    context.add_widget_data::<ScrollBarProps, EmptyState>();
    context.add_widget_data::<ScrollContentProps, EmptyState>();
    context.add_widget_data::<ScrollBoxProps, EmptyState>();
    context.add_widget_data::<ScrollContextProvider, EmptyState>();
    context.add_widget_data::<TextBoxProps, TextBoxState>();

    context.add_widget_system(KayakApp::default().get_name(), app_update, app_render);
    context.add_widget_system(
        KButton::default().get_name(),
        widget_update::<KButton, EmptyState>,
        button_update,
    );
    context.add_widget_system(
        TextProps::default().get_name(),
        widget_update::<TextProps, EmptyState>,
        text_render,
    );
    context.add_widget_system(
        KWindow::default().get_name(),
        widget_update::<KWindow, EmptyState>,
        window_update,
    );
    context.add_widget_system(
        Background::default().get_name(),
        widget_update::<Background, EmptyState>,
        update_background,
    );
    context.add_widget_system(
        Clip::default().get_name(),
        widget_update::<Clip, EmptyState>,
        update_clip,
    );
    context.add_widget_system(
        Image::default().get_name(),
        widget_update::<Image, EmptyState>,
        update_image,
    );
    context.add_widget_system(
        TextureAtlas::default().get_name(),
        widget_update::<TextureAtlas, EmptyState>,
        update_texture_atlas,
    );
    context.add_widget_system(
        NinePatch::default().get_name(),
        widget_update::<NinePatch, EmptyState>,
        update_nine_patch,
    );
    context.add_widget_system(
        Element::default().get_name(),
        widget_update::<Element, EmptyState>,
        update_element,
    );
    context.add_widget_system(
        ScrollBarProps::default().get_name(),
        widget_update_with_context::<ScrollBarProps, EmptyState, ScrollContext>,
        update_scroll_bar,
    );
    context.add_widget_system(
        ScrollContentProps::default().get_name(),
        widget_update_with_context::<ScrollContentProps, EmptyState, ScrollContext>,
        update_scroll_content,
    );
    context.add_widget_system(
        ScrollBoxProps::default().get_name(),
        widget_update_with_context::<ScrollBoxProps, EmptyState, ScrollContext>,
        update_scroll_box,
    );
    context.add_widget_system(
        ScrollContextProvider::default().get_name(),
        widget_update::<ScrollContextProvider, EmptyState>,
        update_scroll_context,
    );
    context.add_widget_system(
        TextBoxProps::default().get_name(),
        widget_update::<TextBoxProps, TextBoxState>,
        update_text_box,
    );
}
