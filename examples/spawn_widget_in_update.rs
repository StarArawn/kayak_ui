use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

struct EventSpawnTextWidget;

#[derive(Resource)]
struct ParentWidget(Option<Entity>);

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
    mut parent_widget: ResMut<ParentWidget>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let handle_spawn_text_widget = OnEvent::new(
        move |In((event_dispatcher_context, _, event, _entity)): In<(
            EventDispatcherContext,
            WidgetState,
            Event,
            Entity,
        )>,
              mut ev_spawn_text_widget: EventWriter<EventSpawnTextWidget>| {
            if let EventType::MouseDown(_) = event.event_type {
                ev_spawn_text_widget.send(EventSpawnTextWidget);
            }
            (event_dispatcher_context, event)
        },
    );

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <KButtonBundle
                button={KButton {
                    text: "Click me to spawn a new TextWidgetBundle".to_owned(),
                    ..Default::default()
                }}
                on_event={handle_spawn_text_widget}
            />
            <ElementBundle
                id={"parent_widget_entity"}
            >
            {parent_widget.0 = Some(parent_widget_entity);}
            </ElementBundle>
        </KayakAppBundle>
    }

    commands.spawn(UICameraBundle::new(widget_context));
}

fn handle_events(
    parent_widget: Res<ParentWidget>,
    mut commands: Commands,
    mut q_context: Query<&mut KayakRootContext>,
    mut ev_spawn_widget: EventReader<EventSpawnTextWidget>,
) {
    for _ in ev_spawn_widget.iter() {
        let mut widget_context = q_context.single_mut();

        let text_entity = widget_context.spawn_widget_update(
            &mut commands,
            parent_widget.0,
            TextWidgetBundle {
                text: TextProps {
                    content: "Hello from update!".to_owned(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        info!("Spawned {}", text_entity.index());
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .insert_resource(ParentWidget(None))
        .add_event::<EventSpawnTextWidget>()
        .add_startup_system(startup)
        .add_system(handle_events)
        .run()
}
