use bevy::prelude::*;
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
    computed_styles: ComputedStyles,
    widget_name: WidgetName,
}

impl Default for CurrentCountBundle {
    fn default() -> Self {
        Self {
            count: CurrentCount::default(),
            styles: KStyle::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: CurrentCount::default().get_name(),
        }
    }
}

fn current_count_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&CurrentCountState>,
) -> bool {
    let state_entity =
        widget_context.use_state(&mut commands, entity, CurrentCountState::default());
    if let Ok(current_count) = query.get(state_entity) {
        let parent_id = Some(entity);
        rsx! {
            <ElementBundle>
                <TextWidgetBundle
                    text={
                        TextProps {
                            content: format!("Current Count: {}", current_count.foo),
                            size: 16.0,
                            line_height: Some(40.0),
                            ..Default::default()
                        }
                    }
                />
                <KButtonBundle
                    button={KButton {
                        text: "Click me!".into(),
                        ..Default::default()
                    }}
                    on_event={OnEvent::new(
                        move |In((event_dispatcher_context, _, mut event, _entity)): In<(EventDispatcherContext, WidgetState, Event, Entity)>,
                            mut query: Query<&mut CurrentCountState>| {
                            match event.event_type {
                                EventType::Click(..) => {
                                    event.prevent_default();
                                    event.stop_propagation();
                                    if let Ok(mut current_count) = query.get_mut(state_entity) {
                                        current_count.foo += 1;
                                    }
                                }
                                _ => {}
                            }
                            (event_dispatcher_context, event)
                        },
                    )}
                />
            </ElementBundle>
        }
    }

    true
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("lato-light.kttf"));

    // Camera 2D forces a clear pass in bevy.
    // We do this because our scene is not rendering anything else.
    commands.spawn(Camera2dBundle::default());

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    widget_context.add_widget_data::<CurrentCount, CurrentCountState>();
    widget_context.add_widget_system(
        CurrentCount::default().get_name(),
        widget_update::<CurrentCount, CurrentCountState>,
        current_count_render,
    );
    rsx! {
        <KayakAppBundle>
            <WindowContextProviderBundle>
                <WindowBundle
                    window={KWindow {
                        title: "State Example Window".into(),
                        draggable: true,
                        initial_position: Vec2::new(10.0, 10.0),
                        size: Vec2::new(300.0, 250.0),
                        ..KWindow::default()
                    }}
                >
                    <CurrentCountBundle />
                </WindowBundle>
                <WindowBundle
                    window={KWindow {
                        title: "State Example Window".into(),
                        draggable: true,
                        initial_position: Vec2::new(500.0, 10.0),
                        size: Vec2::new(300.0, 250.0),
                        ..KWindow::default()
                    }}
                >
                    <CurrentCountBundle />
                </WindowBundle>
            </WindowContextProviderBundle>
        </KayakAppBundle>
    }

    commands.spawn(UICameraBundle::new(widget_context));
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
