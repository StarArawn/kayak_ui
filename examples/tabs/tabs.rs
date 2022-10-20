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
}

fn main() {
    BevyApp::new()
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
