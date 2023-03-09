use bevy::{app::AppExit, prelude::*};
use kayak_ui::prelude::{widgets::*, *};

#[derive(Default, Clone, PartialEq, Component)]
pub struct MenuButton {
    text: String,
}

impl Widget for MenuButton {}

#[derive(Bundle)]
pub struct MenuButtonBundle {
    button: MenuButton,
    styles: KStyle,
    on_event: OnEvent,
    widget_name: WidgetName,
}

impl Default for MenuButtonBundle {
    fn default() -> Self {
        Self {
            button: Default::default(),
            styles: KStyle {
                bottom: Units::Pixels(20.0).into(),
                cursor: KCursorIcon(CursorIcon::Hand).into(),
                ..Default::default()
            },
            on_event: OnEvent::default(),
            widget_name: MenuButton::default().get_name(),
        }
    }
}

fn menu_button_render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    menu_button_query: Query<&MenuButton>,
    state_query: Query<&ButtonState>,
) -> bool {
    let state_entity =
        widget_context.use_state(&mut commands, entity, ButtonState { hovering: false });

    let button_text = menu_button_query.get(entity).unwrap().text.clone();
    let button_image = asset_server.load("main_menu/button.png");
    let button_image_hover = asset_server.load("main_menu/button-hover.png");

    let on_event = OnEvent::new(
        move |In((event_dispatcher_context, _, mut event, _entity)): In<(
            EventDispatcherContext,
            WidgetState,
            KEvent,
            Entity,
        )>,
              mut query: Query<&mut ButtonState>| {
            if let Ok(mut button) = query.get_mut(state_entity) {
                match event.event_type {
                    EventType::MouseIn(..) => {
                        event.stop_propagation();
                        button.hovering = true;
                    }
                    EventType::MouseOut(..) => {
                        button.hovering = false;
                    }
                    _ => {}
                }
            }
            (event_dispatcher_context, event)
        },
    );

    if let Ok(button_state) = state_query.get(state_entity) {
        let button_image_handle = if button_state.hovering {
            button_image_hover
        } else {
            button_image
        };

        let parent_id = Some(entity);
        rsx! {
            <NinePatchBundle
                nine_patch={NinePatch {
                    handle: button_image_handle,
                    border: Edge::all(10.0),
                }}
                styles={KStyle {
                    width: Units::Stretch(1.0).into(),
                    height: Units::Pixels(40.0).into(),
                    bottom: Units::Pixels(30.0).into(),
                    left: Units::Pixels(50.0).into(),
                    right: Units::Pixels(50.0).into(),
                    ..KStyle::default()
                }}
                on_event={on_event}
            >
                <TextWidgetBundle
                    styles={KStyle {
                        top: Units::Stretch(1.0).into(),
                        bottom: Units::Stretch(1.0).into(),
                        ..Default::default()
                    }}
                    text={TextProps {
                        alignment: Alignment::Middle,
                        content: button_text,
                        size: 28.0,
                        ..Default::default()
                    }}
                />
            </NinePatchBundle>
        };
    }
    true
}

#[derive(Default, Resource)]
pub struct PreloadResource {
    images: Vec<Handle<Image>>,
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
    mut preload_resource: ResMut<PreloadResource>,
) {
    font_mapping.set_default(asset_server.load("lato-light.kttf"));

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    widget_context.add_widget_data::<MenuButton, ButtonState>();
    widget_context.add_widget_system(
        MenuButton::default().get_name(),
        widget_update::<MenuButton, ButtonState>,
        menu_button_render,
    );

    let panel1_image = asset_server.load("main_menu/panel1.png");
    let logo_image = asset_server.load("main_menu/logo.png");
    let kayak_image = asset_server.load("main_menu/kayak.png");
    let button_image = asset_server.load("main_menu/button.png");
    let button_image_hover = asset_server.load("main_menu/button-hover.png");

    preload_resource.images.extend(vec![
        panel1_image.clone(),
        logo_image.clone(),
        button_image.clone(),
        button_image_hover.clone(),
    ]);

    let handle_click_close = OnEvent::new(
        move |In((event_dispatcher_context, _, event, _entity)): In<(
            EventDispatcherContext,
            WidgetState,
            KEvent,
            Entity,
        )>,
              mut exit: EventWriter<AppExit>| {
            match event.event_type {
                EventType::Click(..) => {
                    exit.send(AppExit);
                }
                _ => {}
            }
            (event_dispatcher_context, event)
        },
    );

    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <NinePatchBundle
                nine_patch={NinePatch {
                    handle: panel1_image,
                    border: Edge::all(25.0),
                }}
                styles={KStyle {
                    width: Units::Pixels(350.0).into(),
                    height: Units::Pixels(512.0).into(),
                    left: Units::Stretch(1.0).into(),
                    right: Units::Stretch(1.0).into(),
                    top: Units::Stretch(1.0).into(),
                    bottom: Units::Stretch(1.0).into(),
                    padding: Edge::new(
                        Units::Pixels(20.0),
                        Units::Pixels(20.0),
                        Units::Pixels(50.0),
                        Units::Pixels(20.0),
                    ).into(),
                    ..KStyle::default()
                }}
            >
                <KImageBundle
                    image={KImage(kayak_image)}
                    styles={KStyle {
                        width: Units::Pixels(310.0).into(),
                        height: Units::Pixels(104.0).into(),
                        top: Units::Pixels(25.0).into(),
                        bottom: Units::Pixels(25.0).into(),
                        ..KStyle::default()
                    }}
                />
                <KImageBundle
                    image={KImage(logo_image)}
                    styles={KStyle {
                        width: Units::Pixels(310.0).into(),
                        height: Units::Pixels(78.0).into(),
                        bottom: Units::Stretch(1.0).into(),
                        ..KStyle::default()
                    }}
                />
                <MenuButtonBundle button={MenuButton { text: "Play".into() }} />
                <MenuButtonBundle button={MenuButton { text: "Options".into() }} />
                <MenuButtonBundle
                    button={MenuButton { text: "Quit".into() }}
                    on_event={handle_click_close}
                />
            </NinePatchBundle>
        </KayakAppBundle>
    };

    commands.spawn(UICameraBundle::new(widget_context));
}

fn main() {
    App::new()
        .init_resource::<PreloadResource>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
