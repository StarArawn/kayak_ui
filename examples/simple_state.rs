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
                    styles={KStyle {
                        font_size: (48.).into(),
                        height: Units::Pixels(64.).into(),
                        ..default()
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
