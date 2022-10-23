use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

mod tab;
mod tab_button;
mod tab_context;
use tab::{tab_render, Tab, TabBundle};
use tab_button::{tab_button_render, TabButton, TabButtonBundle};
use tab_context::{tab_context_render, TabContextProvider, TabContextProviderBundle};

use crate::tab_context::TabContext;

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_data::<Tab, EmptyState>();
    widget_context.add_widget_data::<TabContextProvider, EmptyState>();
    widget_context.add_widget_data::<TabButton, EmptyState>();
    widget_context.add_widget_system(
        Tab::default().get_name(),
        widget_update_with_context::<Tab, EmptyState, TabContext>,
        tab_render,
    );
    widget_context.add_widget_system(
        TabContextProvider::default().get_name(),
        widget_update_with_context::<TabContextProvider, EmptyState, TabContext>,
        tab_context_render,
    );
    widget_context.add_widget_system(
        TabButton::default().get_name(),
        widget_update_with_context::<TabButton, EmptyState, TabContext>,
        tab_button_render,
    );
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
                            height: StyleProp::Value(Units::Pixels(25.0)),
                            width: StyleProp::Value(Units::Stretch(1.0)),
                            ..Default::default()
                        }}
                    >
                        <TabButtonBundle
                            tab_button={TabButton { index: 0, title: "Tab 1".into(), }}
                        />
                        <TabButtonBundle tab_button={TabButton { index: 1, title: "Tab 2".into() }} />
                    </ElementBundle>
                    <ElementBundle
                        styles={KStyle {
                            height: StyleProp::Value(Units::Stretch(1.0)),
                            width: StyleProp::Value(Units::Stretch(1.0)),
                            ..Default::default()
                        }}
                    >
                        <TabBundle tab={Tab { index: 0 }}>
                            <TextWidgetBundle text={TextProps { content: "Tab 1 Content".into(), size: 14.0, line_height: Some(14.0), ..Default::default() }} />
                        </TabBundle>
                        <TabBundle tab={Tab { index: 1 }}>
                            <TextWidgetBundle text={TextProps { content: "Tab 2 Content".into(), size: 14.0, line_height: Some(14.0), ..Default::default() }} />
                        </TabBundle>
                    </ElementBundle>
                </TabContextProviderBundle>
            </WindowBundle>
        </KayakAppBundle>
    }

    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
