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
pub use text_box::{TextBoxBundle, TextBoxProps};
pub use texture_atlas::{TextureAtlas, TextureAtlasBundle};
pub use window::{KWindow, WindowBundle};

use app::app_update;
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
use text::text_update;
use text_box::update_text_box;
use texture_atlas::update_texture_atlas;
use window::window_update;

use crate::{context::Context, widget::Widget};

pub struct KayakWidgets;

impl Plugin for KayakWidgets {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system_to_stage(StartupStage::PostStartup, add_widget_systems);
    }
}

fn add_widget_systems(mut context: ResMut<Context>) {
    context.add_widget_system(KayakApp::default().get_name(), app_update);
    context.add_widget_system(KButton::default().get_name(), button_update);
    context.add_widget_system(TextProps::default().get_name(), text_update);
    context.add_widget_system(KWindow::default().get_name(), window_update);
    context.add_widget_system(Background::default().get_name(), update_background);
    context.add_widget_system(Clip::default().get_name(), update_clip);
    context.add_widget_system(Image::default().get_name(), update_image);
    context.add_widget_system(TextureAtlas::default().get_name(), update_texture_atlas);
    context.add_widget_system(NinePatch::default().get_name(), update_nine_patch);
    context.add_widget_system(Element::default().get_name(), update_element);
    context.add_widget_system(ScrollBarProps::default().get_name(), update_scroll_bar);
    context.add_widget_system(
        ScrollContentProps::default().get_name(),
        update_scroll_content,
    );
    context.add_widget_system(ScrollBoxProps::default().get_name(), update_scroll_box);
    context.add_widget_system(
        ScrollContextProvider::default().get_name(),
        update_scroll_context,
    );
    context.add_widget_system(TextBoxProps::default().get_name(), update_text_box);
}
