use bevy::{
    prelude::{
        App as BevyApp, AssetServer, Bundle, Commands, Component, Entity, In, Query, Res, ResMut,
        Vec2,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default, PartialEq, Clone)]
struct CurrentCount;

impl Widget for CurrentCount {}

#[derive(Component, Default, PartialEq, Clone)]
struct CurrentCountState {
    foo: u32,
}

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

fn current_count_render(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&CurrentCountState>,
) -> bool {
    let state_entity =
        widget_context.use_state(&mut commands, entity, CurrentCountState::default());
    if let Ok(current_count) = query.get(state_entity) {
        let parent_id = Some(entity);
        rsx! {
            <TextWidgetBundle
                text={
                    TextProps {
                        content: format!("Current Count: {}", current_count.foo).into(),
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
    widget_context.add_widget_data::<CurrentCount, CurrentCountState>();
    widget_context.add_widget_system(
        CurrentCount::default().get_name(),
        widget_update::<CurrentCount, CurrentCountState>,
        current_count_render,
    );
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "State Example Window".into(),
                    draggable: true,
                    initial_position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(300.0, 250.0),
                    ..KWindow::default()
                }}
            >
                <CurrentCountBundle id={"current_count_entity"} />
                <KButtonBundle
                    on_event={OnEvent::new(
                        move |In((event_dispatcher_context, widget_state, event, _entity)): In<(EventDispatcherContext, WidgetState, Event, Entity)>,
                            mut query: Query<&mut CurrentCountState>| {
                            match event.event_type {
                                EventType::Click(..) => {
                                    if let Some(state_entity) = widget_state.get(current_count_entity) {
                                        if let Ok(mut current_count) = query.get_mut(state_entity) {
                                            current_count.foo += 1;
                                        }
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
