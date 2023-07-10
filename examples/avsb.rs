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
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
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
                    }}
                    on_event={OnEvent::new(
                        move |In(_entity): In<Entity>,
                        mut event: ResMut<KEvent>,
                            mut query: Query<&mut CurrentCountState>| {
                            if let EventType::Click(..) = event.event_type {
                                event.prevent_default();
                                event.stop_propagation();
                                if let Ok(mut current_count) = query.get_mut(state_entity) {
                                    current_count.foo += 1;
                                }
                            }
                        },
                    )}
                />
            </ElementBundle>
        };
    }

    true
}

/// A test for switching from component A to component B.
/// Q: Why does adding a key cause the state of the widget to "reset"?
/// A: When you use a unique identifier the entity id will be unique to the identifier.
///    if you do not use a key the widget is considered the "same" as before with updated props.
#[derive(Component, Default, Clone, PartialEq)]
pub struct AvsB {}

impl Widget for AvsB {}

#[derive(Component, Default, Clone, PartialEq)]
pub struct AvsBState {
    is_a: bool,
}

#[derive(Bundle)]
pub struct AvsBBundle {
    pub avsb: AvsB,
    pub widget_name: WidgetName,
}

impl Default for AvsBBundle {
    fn default() -> Self {
        Self {
            avsb: Default::default(),
            widget_name: AvsB::default().get_name(),
        }
    }
}

fn render(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    query: Query<&AvsBState>,
) -> bool {
    let state_entity = widget_context.use_state(&mut commands, entity, AvsBState::default());
    if let Ok(state) = query.get(state_entity) {
        let parent_id = Some(entity);

        rsx! {
            <ElementBundle>
                <WindowContextProviderBundle>
                    {
                        if state.is_a {
                            constructor! {
                                <WindowBundle
                                    window={KWindow {
                                        title: "Window A".into(),
                                        draggable: true,
                                        initial_position: Vec2::new(500.0, 10.0),
                                        size: Vec2::new(300.0, 250.0),
                                        ..KWindow::default()
                                    }}
                                >
                                    <CurrentCountBundle
                                        // Comment me out to see what happens when unique keys are not added!
                                        key={"current-count-0"}
                                    />
                                </WindowBundle>
                            }
                        } else {
                            constructor! {
                                <WindowBundle
                                    window={KWindow {
                                        title: "Window B".into(),
                                        draggable: true,
                                        initial_position: Vec2::new(500.0, 10.0),
                                        size: Vec2::new(300.0, 250.0),
                                        ..KWindow::default()
                                    }}
                                >
                                    <CurrentCountBundle
                                        // Comment me out to see what happens when unique keys are not added!
                                        key={"current-count-1"}
                                    />
                                </WindowBundle>
                            }
                        }
                    }
                </WindowContextProviderBundle>
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

    font_mapping.set_default(asset_server.load("fonts/roboto.kttf"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);

    widget_context.add_widget_data::<AvsB, AvsBState>();
    widget_context.add_widget_system(
        AvsB::default().get_name(),
        widget_update::<AvsB, AvsBState>,
        render,
    );

    widget_context.add_widget_data::<CurrentCount, CurrentCountState>();
    widget_context.add_widget_system(
        CurrentCount::default().get_name(),
        widget_update::<CurrentCount, CurrentCountState>,
        current_count_render,
    );

    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <AvsBBundle />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn swap(input: Res<Input<KeyCode>>, mut query: Query<&mut AvsBState, Without<PreviousWidget>>) {
    if input.just_pressed(KeyCode::Space) {
        for mut avsb in query.iter_mut() {
            avsb.is_a = !avsb.is_a;
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            // bevy_inspector_egui::quick::WorldInspectorPlugin::new(),
            KayakContextPlugin,
            KayakWidgets,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, swap)
        .run()
}
