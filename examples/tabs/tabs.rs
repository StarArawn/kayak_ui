use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, ImageSettings, Res, ResMut, Vec2},
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, *};

mod tab;
mod tab_button;
mod tab_context;
use tab::{tab_update, Tab, TabBundle};
use tab_button::{tab_button_update, TabButton, TabButtonBundle};
use tab_context::{tab_context_update, TabContextProvider, TabContextProviderBundle};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
<<<<<<< HEAD
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

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
=======
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_system(Tab::default().get_name(), tab_update);
    widget_context.add_widget_system(TabContextProvider::default().get_name(), tab_context_update);
    widget_context.add_widget_system(TabButton::default().get_name(), tab_button_update);
    let parent_id = None;

    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "Tabs".into(),
                    draggable: true,
                    position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(300.0, 250.0),
                    ..KWindow::default()
                }}
            >
                <TabContextProviderBundle tab_provider={TabContextProvider { initial_index: 0 }}>
                    <ElementBundle
                        styles={KStyle {
                            layout_type: StyleProp::Value(LayoutType::Row),
                            height: StyleProp::Value(Units::Auto),
                            width: StyleProp::Value(Units::Stretch(1.0)),
                            ..Default::default()
                        }}
                    >
                        <TabButtonBundle tab_button={TabButton { index: 0, title: "Tab 1".into() }} />
                        <TabButtonBundle tab_button={TabButton { index: 1, title: "Tab 2".into() }} />
                    </ElementBundle>
                    <TabBundle tab={Tab { index: 0 }}>
                        <TextWidgetBundle text={TextProps { content: "Tab 1".into(), ..Default::default() }} />
                    </TabBundle>
                    <TabBundle tab={Tab { index: 1 }}>
                        <TextWidgetBundle text={TextProps { content: "Tab 2".into(), ..Default::default() }} />
                    </TabBundle>
                </TabContextProviderBundle>
            </WindowBundle>
        </KayakAppBundle>
    }

    commands.insert_resource(widget_context);
>>>>>>> exp/main
}

fn main() {
    BevyApp::new()
<<<<<<< HEAD
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
=======
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
>>>>>>> exp/main
}
