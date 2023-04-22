use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default, Clone, PartialEq)]
pub struct MyQuad {
    transition: TransitionProps,
}

fn my_quad_update(
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<&MyQuad>,
) -> bool {
    if let Ok(quad) = query.get_mut(entity) {
        create_transition(
            &widget_context,
            &mut commands,
            entity,
            &Transition::new(&quad.transition),
        );
    }

    true
}

impl Widget for MyQuad {}

#[derive(Bundle)]
pub struct MyQuadBundle {
    my_quad: MyQuad,
    computed_styles: ComputedStyles,
    on_event: OnEvent,
    widget_name: WidgetName,
}

impl Default for MyQuadBundle {
    fn default() -> Self {
        Self {
            my_quad: Default::default(),
            on_event: OnEvent::default(),
            computed_styles: ComputedStyles::default(),
            widget_name: MyQuad::default().get_name(),
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
    widget_context.add_widget_system(
        MyQuad::default().get_name(),
        widget_update::<MyQuad, EmptyState>,
        my_quad_update,
    );
    let parent_id = None;

    // Some default styles for our transition examples
    let quad_styles = KStyle {
        left: Units::Pixels(50.0).into(),
        top: Units::Pixels(50.0).into(),
        width: Units::Pixels(100.0).into(),
        height: Units::Pixels(100.0).into(),
        ..Default::default()
    };

    rsx! {
        <KayakAppBundle>
            // Move a quad back and forth.
            <TransitionBundle
                transition={TransitionProps {
                    easing: TransitionEasing::QuadraticInOut,
                    timeout: 500.0,
                    looping: true,
                    style_a: KStyle {
                        ..quad_styles.clone()
                    },
                    style_b: KStyle {
                        left: Units::Pixels(500.0).into(),
                        ..quad_styles.clone()
                    },
                    ..Default::default()
                }}
            >
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::rgba(1.0, 0.0, 0.0, 1.0).into(),
                        ..Default::default()
                    }}
                />
            </TransitionBundle>
            // Example of two transitions. One element that moves, and the other that changes the color of the quad!
            <TransitionBundle
                transition={TransitionProps {
                    easing: TransitionEasing::CubicInOut,
                    timeout: 5000.0,
                    looping: true,
                    style_a: KStyle {
                        left: Units::Pixels(50.0).into(),
                        ..quad_styles.clone()
                    },
                    style_b: KStyle {
                        left: Units::Pixels(500.0).into(),
                        ..quad_styles.clone()
                    },
                    ..Default::default()
                }}
            >
                // We can have a nested transition!
                <MyQuadBundle
                    my_quad={MyQuad {
                        transition: TransitionProps {
                            easing: TransitionEasing::CircularInOut,
                            timeout: 5000.0,
                            looping: true,
                            style_a: KStyle {
                                render_command: RenderCommand::Quad.into(),
                                background_color: Color::rgba(1.0, 0.0, 0.0, 1.0).into(),
                                ..Default::default()
                            },
                            style_b: KStyle {
                                render_command: RenderCommand::Quad.into(),
                                background_color: Color::rgba(0.0, 0.0, 1.0, 1.0).into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }}
                    }
                />
            </TransitionBundle>
            // Transitions work with clipping as well!
            <TransitionBundle
                transition={TransitionProps {
                    easing: TransitionEasing::CubicInOut,
                    timeout: 1000.0,
                    looping: true,
                    style_a: KStyle {
                        width: Units::Pixels(0.0).into(),
                        ..quad_styles.clone()
                    },
                    style_b: KStyle {
                        width: Units::Pixels(100.0).into(),
                        ..quad_styles.clone()
                    },
                    ..Default::default()
                }}
            >
                <ClipBundle>
                    <BackgroundBundle
                        styles={KStyle {
                            background_color: Color::rgba(1.0, 0.0, 0.0, 1.0).into(),
                            ..Default::default()
                        }}
                    />
                </ClipBundle>
            </TransitionBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
