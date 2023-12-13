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
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    query: Query<&MyWidgetState>,
) -> bool {
    let state_entity = widget_context.use_state(&mut commands, entity, MyWidgetState::default());
    if let Ok(state) = query.get(state_entity) {
        let parent_id = Some(entity);
        rsx! {
            <ElementBundle>
                <KButtonBundle
                    styles={KStyle {
                        left: Units::Stretch(1.0).into(),
                        right: Units::Stretch(1.0).into(),
                        ..Default::default()
                    }}
                    button={KButton {
                        text: "Show Window".into(),
                    }}
                    on_event={OnEvent::new(
                        move |In(_entity): In<Entity>,
                        mut event: ResMut<KEvent>,
                            mut query: Query<&mut MyWidgetState>| {
                            event.prevent_default();
                            event.stop_propagation();
                            if let EventType::Click(..) = event.event_type {
                                if let Ok(mut state) = query.get_mut(state_entity) {
                                    state.show_window = true;
                                }
                            }
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
                                button={KButton { text: "Hide Window".into() }}
                                on_event={OnEvent::new(
                                    move |In(_entity): In<Entity>,
                                    mut event: ResMut<KEvent>,
                                        mut query: Query<&mut MyWidgetState>| {
                                        if let EventType::Click(..) = event.event_type {
                                            event.prevent_default();
                                            event.stop_propagation();
                                            if let Ok(mut state) = query.get_mut(state_entity) {
                                                state.show_window = false;
                                            }
                                        }
                                    },
                                )}
                            />
                        </WindowBundle>
                    }
                }}
            </ElementBundle>
        };
    }

    true
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    font_mapping.set_default(asset_server.load("lato-light.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
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
