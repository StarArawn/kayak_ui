use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::AsBindGroup};
use kayak_ui::prelude::{widgets::*, *};

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "94c4e6f9-6f10-422c-85ec-6d582d471afc"]
pub struct MyUIMaterial {}
impl MaterialUI for MyUIMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "rainbow_shader.wgsl".into()
    }
}

#[derive(Component, Default, PartialEq, Clone)]
struct MyWidget;

impl Widget for MyWidget {}

#[derive(Component, Default, PartialEq, Clone)]
struct MyWidgetState {
    pub show_modal: bool,
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
    mut materials: ResMut<Assets<MyUIMaterial>>,
) -> bool {
    let state_entity = widget_context.use_state(&mut commands, entity, MyWidgetState::default());
    if let Ok(state) = query.get(state_entity) {
        let my_material = MyUIMaterial {};
        let my_material_handle = materials.add(my_material);
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
                        text: "Show Modal".into(),
                    }}
                    on_event={OnEvent::new(
                        move |In(_entity): In<Entity>,
                        mut event: ResMut<KEvent>,
                            mut query: Query<&mut MyWidgetState>| {
                            event.prevent_default();
                            event.stop_propagation();
                            if let EventType::Click(..) = event.event_type {
                                if let Ok(mut state) = query.get_mut(state_entity) {
                                    state.show_modal = true;
                                }
                            }
                        },
                    )}
                />
                <ModalBundle
                    modal={Modal {
                        title: "Modal".into(),
                        visible: state.show_modal,
                        ..Modal::default()
                    }}
                    styles={KStyle {
                        left: Units::Stretch(0.75).into(),
                        right: Units::Stretch(0.75).into(),
                        top: Units::Stretch(0.5).into(),
                        bottom: Units::Stretch(0.5).into(),
                        ..Default::default()
                    }}
                >
                    <TextWidgetBundle
                        styles={KStyle {
                            material: MaterialHandle::new(move |commands, entity| {
                                commands.entity(entity).insert(my_material_handle.clone_weak());
                            }).into(),
                            ..Default::default()
                        }}
                        text={TextProps {
                            content: "Hello Modal!".into(),
                            size: 20.0,
                            ..Default::default()
                        }}
                    />
                    <KButtonBundle
                        button={KButton { text: "Hide Modal".into() }}
                        on_event={OnEvent::new(
                            move |In(_entity): In<Entity>,
                            mut event: ResMut<KEvent>,
                                mut query: Query<&mut MyWidgetState>| {
                                if let EventType::Click(..) = event.event_type {
                                    event.prevent_default();
                                    event.stop_propagation();
                                    if let Ok(mut state) = query.get_mut(state_entity) {
                                        state.show_modal = false;
                                    }
                                }
                            },
                        )}
                    />
                </ModalBundle>
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
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_plugin(MaterialUIPlugin::<MyUIMaterial>::default())
        .add_startup_system(startup)
        .run()
}
