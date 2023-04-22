use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use kayak_ui::prelude::{widgets::*, KStyle, *};

#[derive(Component, Default, Clone, PartialEq)]
pub struct MyQuad {
    pos: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub z_index: i32,
}

fn my_quad_update(
    In(entity): In<Entity>,
    mut query: Query<(&MyQuad, &KStyle, &mut ComputedStyles, &mut OnEvent)>,
) -> bool {
    if let Ok((quad, style, mut computed_styles, mut on_event)) = query.get_mut(entity) {
        *computed_styles = KStyle::default()
            .with_style(KStyle {
                render_command: StyleProp::Value(RenderCommand::Quad),
                position_type: StyleProp::Value(KPositionType::SelfDirected),
                left: StyleProp::Value(Units::Pixels(quad.pos.x)),
                top: StyleProp::Value(Units::Pixels(quad.pos.y)),
                width: StyleProp::Value(Units::Pixels(quad.size.x)),
                height: StyleProp::Value(Units::Pixels(quad.size.y)),
                z_index: StyleProp::Value(quad.z_index),
                ..Default::default()
            })
            .with_style(style)
            .with_style(KStyle {
                background_color: StyleProp::Value(quad.color),
                ..Default::default()
            })
            .into();

        *on_event = OnEvent::new(
            move |In(_entity): In<Entity>,
                  mut event: ResMut<KEvent>,
                  mut query: Query<(&mut KStyle, &MyQuad)>| {
                event.prevent_default();
                event.stop_propagation();
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
    computed_styles: ComputedStyles,
    on_event: OnEvent,
    widget_name: WidgetName,
}

impl Default for MyQuadBundle {
    fn default() -> Self {
        Self {
            my_quad: Default::default(),
            styles: KStyle::default(),
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

    rsx! {
        <KayakAppBundle>
            {
                (0..1000i32).for_each(|i| {
                    let pos = Vec2::new(fastrand::i32(0..1280) as f32, fastrand::i32(0..720) as f32);
                    let size = Vec2::new(
                        fastrand::i32(32..64) as f32,
                        fastrand::i32(32..64) as f32,
                    );
                    let color = Color::rgba(
                        fastrand::f32(),
                        fastrand::f32(),
                        fastrand::f32(),
                        fastrand::f32(),
                    );
                    constructor! {
                        <MyQuadBundle
                            my_quad={MyQuad { pos, size, color, z_index: i }}
                        />
                    }
                });
            }
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
