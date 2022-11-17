use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default, PartialEq, Clone)]
struct MyWidget;

impl Widget for MyWidget {}

#[derive(Component, Default, PartialEq, Clone)]
struct MyWidgetState {
    pub show_window: bool,
}

#[derive(Bundle)]
struct MyWidgetBundle {
    count: MyWidget,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for MyWidgetBundle {
    fn default() -> Self {
        Self {
            count: MyWidget::default(),
            styles: KStyle::default(),
            widget_name: MyWidget::default().get_name(),
        }
    }
}

fn my_widget_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&MyWidgetState>,
) -> bool {
    let state_entity = widget_context.use_state(&mut commands, entity, MyWidgetState::default());
    if let Ok(state) = query.get(state_entity) {
        let parent_id = Some(entity);
        rsx! {
            <ElementBundle>
                <KButtonBundle
                    button={KButton {
                        text: "Show Window".into(),
                        user_styles: KStyle {
                            left: Units::Stretch(1.0).into(),
                            right: Units::Stretch(1.0).into(),
                            ..Default::default()
                        }
                    }}
                    on_event={OnEvent::new(
                        move |In((event_dispatcher_context, _, mut event, _entity)): In<(EventDispatcherContext, WidgetState, Event, Entity)>,
                            mut query: Query<&mut MyWidgetState>| {
                            event.prevent_default();
                            event.stop_propagation();
                            match event.event_type {
                                EventType::Click(..) => {
                                    if let Ok(mut state) = query.get_mut(state_entity) {
                                        state.show_window = true;
                                    }
                                }
                                _ => {}
                            }
                            (event_dispatcher_context, event)
                        },
                    )}
                />
                {if state.show_window {
                    constructor! {
                        <WindowBundle
                            window={KWindow {
                                title: "Conditional widget rendering!".into(),
                                draggable: true,
                                initial_position: Vec2::new(10.0, 10.0),
                                size: Vec2::new(300.0, 250.0),
                                ..KWindow::default()
                            }}
                        >
                            <KButtonBundle
                                button={KButton { text: "Hide Window".into(), ..Default::default() }}
                                on_event={OnEvent::new(
                                    move |In((event_dispatcher_context, _, mut event, _entity)): In<(EventDispatcherContext, WidgetState, Event, Entity)>,
                                        mut query: Query<&mut MyWidgetState>| {
                                        match event.event_type {
                                            EventType::Click(..) => {
                                                event.prevent_default();
                                                event.stop_propagation();
                                                if let Ok(mut state) = query.get_mut(state_entity) {
                                                    state.show_window = false;
                                                }
                                            }
                                            _ => {}
                                        }
                                        (event_dispatcher_context, event)
                                    },
                                )}
                            />
                        </WindowBundle>
                    }
                }}
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
    font_mapping.set_default(asset_server.load("lato-light.kayak_font"));

    // Camera 2D forces a clear pass in bevy.
    // We do this because our scene is not rendering anything else.
    commands.spawn(Camera2dBundle::default());

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    widget_context.add_widget_data::<MyWidget, MyWidgetState>();
    widget_context.add_widget_system(
        MyWidget::default().get_name(),
        widget_update::<MyWidget, MyWidgetState>,
        my_widget_render,
    );
    rsx! {
        <KayakAppBundle>
            <MyWidgetBundle />
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
