//! This example demonstrates how to use the provider/consumer pattern for passing props down
//! to multiple descendants.
//!
//! The problem we'll be solving here is adding support for theming.
//!
//! One reason the provider/consumer pattern might be favored over a global state is that it allows
//! for better specificity and makes local contexts much easier to manage. In the case of theming,
//! this allows us to have multiple active themes, even if they are nested within each other!

use bevy::{
    prelude::{
        App as BevyApp, AssetServer, Bundle, Color, Commands, Component, Entity, ImageSettings, In,
        Query, Res, ResMut, Vec2,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, KStyle, *};

/// The color theme struct we will be using across our demo widgets
#[derive(Component, Debug, Default, Clone, PartialEq)]
struct Theme {
    name: String,
    primary: Color,
    secondary: Color,
    background: Color,
}

impl Theme {
    fn vampire() -> Self {
        Self {
            name: "Vampire".to_string(),
            primary: Color::rgba(1.0, 0.475, 0.776, 1.0),
            secondary: Color::rgba(0.641, 0.476, 0.876, 1.0),
            background: Color::rgba(0.157, 0.165, 0.212, 1.0),
        }
    }
    fn solar() -> Self {
        Self {
            name: "Solar".to_string(),
            primary: Color::rgba(0.514, 0.580, 0.588, 1.0),
            secondary: Color::rgba(0.149, 0.545, 0.824, 1.0),
            background: Color::rgba(0.026, 0.212, 0.259, 1.0),
        }
    }
    fn vector() -> Self {
        Self {
            name: "Vector".to_string(),
            primary: Color::rgba(0.533, 1.0, 0.533, 1.0),
            secondary: Color::rgba(0.098, 0.451, 0.098, 1.0),
            background: Color::rgba(0.004, 0.059, 0.004, 1.0),
        }
    }
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ThemeButton {
    pub theme: Theme,
}
impl Widget for ThemeButton {}
impl WidgetProps for ThemeButton {}

#[derive(Bundle)]
pub struct ThemeButtonBundle {
    theme_button: ThemeButton,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for ThemeButtonBundle {
    fn default() -> Self {
        Self {
            theme_button: Default::default(),
            styles: KStyle::default(),
            widget_name: ThemeButton::default().get_name(),
        }
    }
}

fn update_theme_button(
    In((widget_context, theme_button_entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&ThemeButton>,
    mut context_query: Query<&mut Theme>,
) -> bool {
    if let Ok(theme_button) = query.get(theme_button_entity) {
        if let Some(theme_context_entity) =
            widget_context.get_context_entity::<Theme>(theme_button_entity)
        {
            if let Ok(theme) = context_query.get_mut(theme_context_entity) {
                let mut box_style = KStyle {
                    width: StyleProp::Value(Units::Pixels(30.0)),
                    height: StyleProp::Value(Units::Pixels(30.0)),
                    background_color: StyleProp::Value(theme_button.theme.primary),
                    ..Default::default()
                };

                if theme_button.theme.name == theme.name {
                    box_style.top = StyleProp::Value(Units::Pixels(3.0));
                    box_style.left = StyleProp::Value(Units::Pixels(3.0));
                    box_style.bottom = StyleProp::Value(Units::Pixels(3.0));
                    box_style.right = StyleProp::Value(Units::Pixels(3.0));
                    box_style.width = StyleProp::Value(Units::Pixels(24.0));
                    box_style.height = StyleProp::Value(Units::Pixels(24.0));
                }

                let parent_id = Some(theme_button_entity);
                rsx! {
                    <BackgroundBundle
                        styles={box_style}
                        on_event={OnEvent::new(
                            move |In((event_dispatcher_context, _, event, _entity)): In<(
                                EventDispatcherContext,
                                WidgetState,
                                Event,
                                Entity,
                            )>,
                            query: Query<&ThemeButton>,
                            mut context_query: Query<&mut Theme>,
                            | {
                                match event.event_type {
                                    EventType::Click(..) => {
                                        if let Ok(button) = query.get(theme_button_entity) {
                                            if let Ok(mut context_theme) = context_query.get_mut(theme_context_entity) {
                                                *context_theme = button.theme.clone();
                                            }
                                        }
                                    },
                                    _ => {}
                                }
                                (event_dispatcher_context, event)
                            },
                        )}
                    />
                }
            }
        }
    }

    true
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
struct ThemeSelector;
impl Widget for ThemeSelector {}
impl WidgetProps for ThemeSelector {}

#[derive(Bundle)]
pub struct ThemeSelectorBundle {
    theme_selector: ThemeSelector,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for ThemeSelectorBundle {
    fn default() -> Self {
        Self {
            theme_selector: Default::default(),
            styles: KStyle {
                height: StyleProp::Value(Units::Auto),
                padding_bottom: Units::Pixels(40.0).into(),
                ..Default::default()
            },
            widget_name: ThemeSelector::default().get_name(),
        }
    }
}

fn update_theme_selector(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    query: Query<&ThemeSelector>,
) -> bool {
    if let Ok(_) = query.get(entity) {
        let button_container_style = KStyle {
            layout_type: StyleProp::Value(LayoutType::Row),
            width: StyleProp::Value(Units::Stretch(1.0)),
            height: StyleProp::Value(Units::Auto),
            top: StyleProp::Value(Units::Pixels(5.0)),
            ..Default::default()
        };

        let vampire_theme = Theme::vampire();
        let solar_theme = Theme::solar();
        let vector_theme = Theme::vector();

        let parent_id = Some(entity);
        rsx! {
            <ElementBundle styles={button_container_style}>
                <ThemeButtonBundle theme_button={ThemeButton { theme: vampire_theme }} />
                <ThemeButtonBundle theme_button={ThemeButton { theme: solar_theme }} />
                <ThemeButtonBundle theme_button={ThemeButton { theme: vector_theme }} />
            </ElementBundle>
        }
    }

    true
}

#[derive(Component, Debug, Default, Clone, PartialEq)]
pub struct ThemeDemo {
    is_root: bool,
    context_entity: Option<Entity>,
}
impl Widget for ThemeDemo {}
impl WidgetProps for ThemeDemo {}

#[derive(Bundle)]
pub struct ThemeDemoBundle {
    theme_demo: ThemeDemo,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for ThemeDemoBundle {
    fn default() -> Self {
        Self {
            theme_demo: Default::default(),
            styles: KStyle::default(),
            widget_name: ThemeDemo::default().get_name(),
        }
    }
}

fn update_theme_demo(
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    mut query_set: Query<&mut ThemeDemo>,
    theme_context: Query<&Theme>,
) -> bool {
    if let Ok(mut theme_demo) = query_set.get_mut(entity) {
        if let Some(theme_context_entity) = widget_context.get_context_entity::<Theme>(entity) {
            if let Ok(theme) = theme_context.get(theme_context_entity) {
                let select_lbl = if theme_demo.is_root {
                    format!("Select Theme (Current: {})", theme.name)
                } else {
                    format!("Select A Different Theme (Current: {})", theme.name)
                };

                if theme_demo.is_root {
                    if theme_demo.context_entity.is_none() {
                        let theme_entity = commands.spawn(Theme::vector()).id();
                        theme_demo.context_entity = Some(theme_entity);
                    }
                }

                let context_entity = if let Some(entity) = theme_demo.context_entity {
                    entity
                } else {
                    Entity::from_raw(1000000)
                };
                let text_styles = KStyle {
                    color: StyleProp::Value(theme.primary),
                    height: StyleProp::Value(Units::Pixels(28.0)),
                    ..Default::default()
                };
                let btn_style = KStyle {
                    background_color: StyleProp::Value(theme.secondary),
                    width: StyleProp::Value(Units::Stretch(0.75)),
                    height: StyleProp::Value(Units::Pixels(32.0)),
                    top: StyleProp::Value(Units::Pixels(5.0)),
                    left: StyleProp::Value(Units::Stretch(1.0)),
                    right: StyleProp::Value(Units::Stretch(1.0)),
                    ..Default::default()
                };

                let parent_id = Some(entity);
                let mut children = kayak_ui::prelude::KChildren::new();
                rsx! {
                    <>
                        <TextWidgetBundle
                            text={TextProps {
                                content: select_lbl,
                                size: 14.0,
                                line_height: Some(28.0),
                                ..Default::default()
                            }}
                            styles={KStyle {
                                height: StyleProp::Value(Units::Pixels(28.0)),
                                ..Default::default()
                            }}
                        />
                        <ThemeSelectorBundle />
                        <BackgroundBundle
                            styles={KStyle {
                                background_color: StyleProp::Value(theme.background),
                                top: StyleProp::Value(Units::Pixels(15.0)),
                                width: StyleProp::Value(Units::Stretch(1.0)),
                                height: StyleProp::Value(Units::Stretch(1.0)),
                                ..Default::default()
                            }}
                        >
                            <TextWidgetBundle
                                text={TextProps {
                                    content: "Lorem ipsum dolor...".into(),
                                    size: 12.0,
                                    ..Default::default()
                                }}
                                styles={text_styles.clone()}
                            />
                            <KButtonBundle
                                styles={btn_style.clone()}
                            >
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "BUTTON".into(),
                                        size: 14.0,
                                        ..Default::default()
                                    }}
                                />
                            </KButtonBundle>
                            {
                                if theme_demo.is_root {
                                    widget_context.set_context_entity::<Theme>(
                                        parent_id,
                                        context_entity,
                                    );
                                    constructor! {
                                        <ElementBundle
                                            styles={KStyle {
                                                top: StyleProp::Value(Units::Pixels(10.0)),
                                                left: StyleProp::Value(Units::Pixels(10.0)),
                                                bottom: StyleProp::Value(Units::Pixels(10.0)),
                                                right: StyleProp::Value(Units::Pixels(10.0)),
                                                ..Default::default()
                                            }}
                                        >
                                            <ThemeDemoBundle />
                                        </ElementBundle>
                                    }
                                }
                            }
                        </BackgroundBundle>
                    </>
                }

                children.process(&widget_context, parent_id);
            }
        }
    }

    true
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_data::<ThemeDemo, EmptyState>();
    widget_context.add_widget_data::<ThemeButton, EmptyState>();
    widget_context.add_widget_data::<ThemeSelector, EmptyState>();
    widget_context.add_widget_system(
        ThemeDemo::default().get_name(),
        widget_update_with_context::<ThemeDemo, EmptyState, Theme>,
        update_theme_demo,
    );
    widget_context.add_widget_system(
        ThemeButton::default().get_name(),
        widget_update_with_context::<ThemeButton, EmptyState, Theme>,
        update_theme_button,
    );
    widget_context.add_widget_system(
        ThemeSelector::default().get_name(),
        widget_update_with_context::<ThemeSelector, EmptyState, Theme>,
        update_theme_selector,
    );
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            {
                let theme_entity = commands.spawn(Theme::vampire()).id();
                widget_context.set_context_entity::<Theme>(parent_id, theme_entity);
            }
            <WindowBundle
                window={KWindow {
                    title: "Context Example".into(),
                    draggable: true,
                    initial_position: Vec2::ZERO,
                    size: Vec2::new(350.0, 400.0),
                    ..Default::default()
                }}
            >
                <ThemeDemoBundle
                    theme_demo={ThemeDemo {
                        is_root: true,
                        context_entity: None,
                    }}
                />
            </WindowBundle>
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    BevyApp::new()
        .insert_resource(ImageSettings::default_nearest())
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
