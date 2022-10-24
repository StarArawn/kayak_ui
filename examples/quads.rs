use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::{
        App as BevyApp, AssetServer, Bundle, Color, Commands, Component, Entity, In, Query, Res,
        ResMut, Vec2,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, KStyle, *};

#[derive(Component, Default, Clone, PartialEq)]
pub struct MyQuad {
    pos: Vec2,
    pub size: Vec2,
    pub color: Color,
}

fn my_quad_update(
    In((_widget_context, entity)): In<(WidgetContext, Entity)>,
    mut query: Query<(&MyQuad, &mut KStyle, &mut OnEvent)>,
) -> bool {
    if let Ok((quad, mut style, mut on_event)) = query.get_mut(entity) {
        if style.render_command.resolve() != RenderCommand::Quad {
            style.render_command = StyleProp::Value(RenderCommand::Quad);
            style.position_type = StyleProp::Value(PositionType::SelfDirected);
            style.left = StyleProp::Value(Units::Pixels(quad.pos.x));
            style.top = StyleProp::Value(Units::Pixels(quad.pos.y));
            style.width = StyleProp::Value(Units::Pixels(quad.size.x));
            style.height = StyleProp::Value(Units::Pixels(quad.size.y));
            style.background_color = StyleProp::Value(quad.color);
        }

        *on_event = OnEvent::new(
            move |In((event_dispatcher_context, _, event, entity)): In<(
                EventDispatcherContext,
                WidgetState,
                Event,
                Entity,
            )>,
                  mut query: Query<(&mut KStyle, &MyQuad)>| {
                match event.event_type {
                    EventType::MouseIn(..) => {
                        if let Ok((mut styles, _)) = query.get_mut(entity) {
                            styles.background_color = StyleProp::Value(Color::WHITE);
                        }
                    }
                    EventType::MouseOut(..) => {
                        if let Ok((mut styles, my_quad)) = query.get_mut(entity) {
                            styles.background_color = StyleProp::Value(my_quad.color);
                        }
                    }
                    _ => {}
                }
                (event_dispatcher_context, event)
            },
        );
    }

    true
}

impl Widget for MyQuad {}

#[derive(Bundle)]
pub struct MyQuadBundle {
    my_quad: MyQuad,
    styles: KStyle,
    on_event: OnEvent,
    widget_name: WidgetName,
}

impl Default for MyQuadBundle {
    fn default() -> Self {
        Self {
            my_quad: Default::default(),
            styles: KStyle::default(),
            on_event: OnEvent::default(),
            widget_name: MyQuad::default().get_name(),
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
    widget_context.add_widget_system(
        MyQuad::default().get_name(),
        widget_update::<MyQuad, EmptyState>,
        my_quad_update,
    );
    let parent_id = None;

    rsx! {
        <KayakAppBundle>
            {
                (0..1000).for_each(|_| {
                    let pos = Vec2::new(fastrand::i32(0..1280) as f32, fastrand::i32(0..720) as f32);
                    let size = Vec2::new(
                        fastrand::i32(32..64) as f32,
                        fastrand::i32(32..64) as f32,
                    );
                    let color = Color::rgba(
                        fastrand::f32(),
                        fastrand::f32(),
                        fastrand::f32(),
                        1.0,
                    );
                    constructor! {
                        <MyQuadBundle
                            my_quad={MyQuad { pos, size, color }}
                        />
                    }
                });
            }
        </KayakAppBundle>
    }

    commands.insert_resource(widget_context);
}

fn main() {
    BevyApp::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
