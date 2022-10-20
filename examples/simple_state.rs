use bevy::{
    prelude::{
        App as BevyApp, AssetServer, Bundle, Changed, Commands, Component, Entity, In, Query, Res,
        ResMut, Vec2,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default)]
struct CurrentCount(pub u32);

impl Widget for CurrentCount {}

#[derive(Bundle)]
struct CurrentCountBundle {
    count: CurrentCount,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for CurrentCountBundle {
    fn default() -> Self {
        Self {
            count: CurrentCount::default(),
            styles: KStyle::default(),
            widget_name: CurrentCount::default().get_name(),
        }
    }
}

fn current_count_update(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&CurrentCount, Changed<CurrentCount>>,
) -> bool {
    if let Ok(current_count) = query.get(entity) {
        let parent_id = Some(entity);
        rsx! {
            <TextWidgetBundle
                text={
                    TextProps {
                        content: format!("Current Count: {}", current_count.0).into(),
                        size: 16.0,
                        line_height: Some(40.0),
                        ..Default::default()
                    }
                }
            />
        }

        return true;
    }

    false
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
    widget_context.add_widget_system(CurrentCount::default().get_name(), current_count_update);
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "State Example Window".into(),
                    draggable: true,
                    position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(300.0, 250.0),
                    ..KWindow::default()
                }}
            >
                <CurrentCountBundle id={"current_count_entity"} />
                <KButtonBundle
                    on_event={OnEvent::new(
                        move |In((event_dispatcher_context, event, _entity)): In<(EventDispatcherContext, Event, Entity)>,
                            mut query: Query<&mut CurrentCount>| {
                            match event.event_type {
                                EventType::Click(..) => {
                                    if let Ok(mut current_count) =
                                        query.get_mut(current_count_entity)
                                    {
                                        current_count.0 += 1;
                                    }
                                }
                                _ => {}
                            }
                            (event_dispatcher_context, event)
                        },
                    )}
                >
                    <TextWidgetBundle
                        text={TextProps {
                            content: "Click me!".into(),
                            size: 16.0,
                            alignment: Alignment::Start,
                            ..Default::default()
                        }}
                    />
                </KButtonBundle>
            </WindowBundle>
        </KayakAppBundle>
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
