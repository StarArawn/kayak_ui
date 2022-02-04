//! This example demonstrates how one might create a tab system
//!
//! Additionally, it showcases focus navigation. Press `Tab` and `Shift + Tab` to move
//! between focusable widgets. This example also sets it up so that `Enter` or `Space`
//! can be used in place of a normal click.

use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};

use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        constructor, render, rsx, WidgetProps,
        styles::{Style, StyleProp, Units},
        widget, Color, Index,
    },
    widgets::{App, Text, Window},
};

use crate::theming::{ColorState, TabTheme, TabThemeProvider};
use tab_box::TabBox;
use tab_box::TabData;

mod tab;
mod tab_bar;
mod tab_box;
mod tab_content;
mod theming;

#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct TabDemoProps {}

#[widget]
fn TabDemo(props: TabDemoProps) {
    let text_style = Style {
        width: StyleProp::Value(Units::Percentage(75.0)),
        top: StyleProp::Value(Units::Stretch(0.5)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    // TODO: This is not the most ideal way to generate tabs. For one, the `content` has no access to its actual context
    // (i.e. where it actually exists in the hierarchy). Additionally, it would be better if tabs were created as
    // children of `TabBox`. These are issues that will be addressed in the future, so for now, this will work.
    let tabs = vec![
        TabData {
            name: "Tab 1".to_string(),
            content: {
                let text_style = text_style.clone();
                constructor! {
                    <>
                        <Text content={"Welcome to Tab 1!".to_string()} size={48.0} styles={Some(text_style)} />
                    </>
                }
            },
        },
        TabData {
            name: "Tab 2".to_string(),
            content: {
                let text_style = text_style.clone();
                constructor! {
                    <>
                        <Text content={"Welcome to Tab 2!".to_string()} size={48.0} styles={Some(text_style)} />
                    </>
                }
            },
        },
        TabData {
            name: "Tab 3".to_string(),
            content: {
                let text_style = text_style.clone();
                constructor! {
                    <>
                        <Text content={"Welcome to Tab 3!".to_string()} size={48.0} styles={Some(text_style)} />
                    </>
                }
            },
        },
    ];

    rsx! {
        <TabBox tabs={tabs} />
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let theme = TabTheme {
        primary: Default::default(),
        bg: Color::new(0.176, 0.227, 0.255, 1.0),
        fg: Color::new(0.286, 0.353, 0.392, 1.0),
        focus: Color::new(0.388, 0.474, 0.678, 0.5),
        text: ColorState {
            normal: Color::new(0.949, 0.956, 0.968, 1.0),
            hovered: Color::new(0.650, 0.574, 0.669, 1.0),
            active: Color::new(0.949, 0.956, 0.968, 1.0),
        },
        active_tab: ColorState {
            normal: Color::new(0.286, 0.353, 0.392, 1.0),
            hovered: Color::new(0.246, 0.323, 0.352, 1.0),
            active: Color::new(0.196, 0.283, 0.312, 1.0),
        },
        inactive_tab: ColorState {
            normal: Color::new(0.176, 0.227, 0.255, 1.0),
            hovered: Color::new(0.16, 0.21, 0.23, 1.0),
            active: Color::new(0.196, 0.283, 0.312, 1.0),
        },
        tab_height: 22.0,
    };

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <Window position={(50.0, 50.0)} size={(600.0, 300.0)} title={"Tabs Example".to_string()}>
                    <TabThemeProvider initial_theme={theme}>
                        <TabDemo />
                    </TabThemeProvider>
                </Window>
            </App>
        }
    });

    commands.insert_resource(context);
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
