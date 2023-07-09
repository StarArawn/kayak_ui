use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default, PartialEq, Eq, Clone)]
pub struct MyWidgetProps {}

fn my_widget_1_update(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    query: Query<Entity, Or<(With<Mounted>, Changed<MyWidgetProps>)>>,
) -> bool {
    if query.get(entity).is_ok() {
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
                                content: (*text).into(),
                                ..Default::default()
                            }}
                        />
                    }
                })}
            </ElementBundle>
        };
        return true;
    }

    false
}

impl Widget for MyWidgetProps {}

#[derive(Bundle)]
pub struct MyWidgetBundle {
    props: MyWidgetProps,
    widget_name: WidgetName,
}

impl Default for MyWidgetBundle {
    fn default() -> Self {
        Self {
            props: Default::default(),
            widget_name: MyWidgetProps::default().get_name(),
        }
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    widget_context.add_widget_data::<MyWidgetProps, EmptyState>();
    widget_context.add_widget_system(
        MyWidgetProps::default().get_name(),
        widget_update::<MyWidgetProps, EmptyState>,
        my_widget_1_update,
    );
    rsx! {
        <KayakAppBundle><MyWidgetBundle /></KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
