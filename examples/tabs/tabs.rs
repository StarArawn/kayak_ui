use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};

use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        styles::{Style, StyleProp, Units},
        Children, Color, constructor, Handler, Index, render, rsx, use_state, widget
    },
    widgets::{App, Text, Window},
};

use tab_box::TabBox;
use tab_box::TabData;
use crate::theming::{ColorState, TabTheme, TabThemeProvider};

mod tab_bar;
mod tab_box;
mod tab_content;
mod tab;
mod theming;

#[widget]
fn TabDemo() {
    let text_style = Style {
        width: StyleProp::Value(Units::Percentage(75.0)),
        top: StyleProp::Value(Units::Stretch(0.5)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        bottom: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let (count, set_count, ..) = use_state!(0);
    let (tabs, set_tabs, ..) = use_state!(vec![
        TabData {
            name: "Tab 1".to_string(),
            content: {
                let children = Children::None;
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
                let children = Children::None;
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
                let children = Children::None;
                let text_style = text_style.clone();
                constructor! {
                    <>
                        <Text content={"Welcome to Tab 3!".to_string()} size={48.0} styles={Some(text_style)} />
                    </>
                }
            },
        },
    ]);

    let tab_clone = tabs.clone();
    let set_added_tabs = set_tabs.clone();
    let on_add_tab = Handler::new(move |_| {
        let mut tab_clone = (&tab_clone).clone();
        tab_clone.push(TabData {
            name: format!("Tab {}", count),
            content: {
                let children = Children::None;
                constructor! {
                    <>
                        <Text content={"Hello".to_string()} size={12.0} />
                    </>
                }
            },
        }, );
        set_count(count + 1);
        set_added_tabs(tab_clone);
    });

    let tab_clone = tabs.clone();
    let on_remove_tab = Handler::new(move |index: usize| {
        let mut tab_clone = (&tab_clone).clone();
        tab_clone.remove(index);
        set_tabs(tab_clone);
    });

    rsx! {
        <TabBox tabs={tabs} on_add_tab={on_add_tab} on_remove_tab={on_remove_tab} />
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    let theme = TabTheme {
        primary: Default::default(),
        bg: Color::new(0.176, 0.227, 0.255, 1.0),
        fg: Color::new(0.286, 0.353, 0.392, 1.0),
        focus: Color::new(0.388, 0.474, 0.678, 0.5),
        text: ColorState {
            normal: Color::new(0.949, 0.956, 0.968, 1.0),
            hovered: Color::new(0.650, 0.574, 0.669, 1.0),
            active: Color::new(0.949, 0.956, 0.968, 1.0),
            disabled: Color::new(0.662, 0.678, 0.694, 1.0),
        },
        active_tab: ColorState {
            normal: Color::new(0.286, 0.353, 0.392, 1.0),
            hovered: Color::new(0.246, 0.323, 0.352, 1.0),
            active: Default::default(),
            disabled: Color::new(0.474, 0.486, 0.505, 1.0),
        },
        inactive_tab: ColorState {
            normal: Color::new(0.176, 0.227, 0.255, 1.0),
            hovered: Color::new(0.16, 0.21, 0.23, 1.0),
            active: Default::default(),
            disabled: Color::new(0.474, 0.486, 0.505, 1.0),
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