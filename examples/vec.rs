use bevy::{
    prelude::{
        App as BevyApp, AssetServer, Bundle, Changed, Commands, Component, Entity, In, Or, Query,
        Res, ResMut, With,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, KStyle, *};

#[derive(Component, Default)]
pub struct MyWidgetProps {}

fn my_widget_1_update(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<Entity, Or<(With<Mounted>, Changed<MyWidgetProps>)>>,
) -> bool {
    if let Ok(_) = query.get(entity) {
        let parent_id = Some(entity);
        let data = vec![
            "Text 1", "Text 2", "Text 3", "Text 4", "Text 5", "Text 6", "Text 7", "Text 8",
            "Text 9", "Text 10",
        ];
        rsx! {
            <ElementBundle>
                {data.iter().for_each(|text| {
                    constructor! {
                        <TextWidgetBundle
                            text={TextProps {
                                content: text.clone().into(),
                                ..Default::default()
                            }}
                        />
                    }
                })}
            </ElementBundle>
        }
        return true;
    }

    false
}

impl Widget for MyWidgetProps {}

#[derive(Bundle)]
pub struct MyWidgetBundle {
    props: MyWidgetProps,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for MyWidgetBundle {
    fn default() -> Self {
        Self {
            props: Default::default(),
            styles: Default::default(),
            widget_name: MyWidgetProps::default().get_name(),
        }
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    let parent_id = None;
    widget_context.add_widget_system(MyWidgetProps::default().get_name(), my_widget_1_update);
    rsx! {
        <KayakAppBundle><MyWidgetBundle /></KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    BevyApp::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
