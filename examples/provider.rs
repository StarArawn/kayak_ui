//! This example demonstrates how to use the provider/consumer pattern for passing props down
//! to multiple descendants.
//!
//! The problem we'll be solving here is adding support for theming. Theming can generally be
//! added by using something like [set_global_state](kayak_core::KayakContext::set_global_state)
//! or [query_world](kayak_core::KayakContext::query_world). However, this example will demonstrate
//! an implementation using providers and consumers.
//!
//! One reason the provider/consumer pattern might be favored over a global state is that it allows
//! for better specificity and makes local contexts much easier to manage. In the case of theming,
//! this allows us to have multiple active themes, even if they are nested within each other!

use bevy::prelude::{
    App as BevyApp, AssetServer, Commands, DefaultPlugins, Res, ResMut, WindowDescriptor,
};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        render, rsx,
        styles::{LayoutType, Style, StyleProp, Units},
        widget, Bound, Color, EventType, Index, MutableBound, OnEvent,
    },
    widgets::{App, Background, Element, If, Text, TooltipConsumer, TooltipProvider, Window},
};
use std::sync::Arc;

/// The color theme struct we will be using across our demo widgets
#[derive(Debug, Default, Clone, PartialEq)]
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
            primary: Color::new(1.0, 0.475, 0.776, 1.0),
            secondary: Color::new(0.641, 0.476, 0.876, 1.0),
            background: Color::new(0.157, 0.165, 0.212, 1.0),
        }
    }
    fn solar() -> Self {
        Self {
            name: "Solar".to_string(),
            primary: Color::new(0.514, 0.580, 0.588, 1.0),
            secondary: Color::new(0.149, 0.545, 0.824, 1.0),
            background: Color::new(0.026, 0.212, 0.259, 1.0),
        }
    }
    fn vector() -> Self {
        Self {
            name: "Vector".to_string(),
            primary: Color::new(0.533, 1.0, 0.533, 1.0),
            secondary: Color::new(0.098, 0.451, 0.098, 1.0),
            background: Color::new(0.004, 0.059, 0.004, 1.0),
        }
    }
}

/// This widget provides a theme to its children
///
/// Any descendant of this provider can access its theme by calling [create_consumer](kayak_core::KayakContext::create_consumer).
/// It can also be nested within itself, allowing for differing provider values.
#[widget]
fn ThemeProvider(context: &mut KayakContext, initial_theme: Theme) {
    // Create the provider
    context.create_provider(initial_theme);
    rsx! { <>{children}</> }
}

/// A widget that shows a colored box, representing the theme
///
/// This widget acts as one of our consumers of the [ThemeProvider]. It then uses the theme data to
/// display its content and also updates the shared state when clicked.
#[widget]
fn ThemeButton(context: &mut KayakContext, theme: Theme) {
    // Create a consumer
    // This grabs the current theme from the nearest ThemeProvider up the widget tree
    let consumer = context
        .create_consumer::<Theme>()
        .expect("Requires ThemeProvider as an ancestor");

    let theme_name = theme.name.clone();
    let consumer_theme_name = consumer.get().name.clone();
    let theme_primary = theme.primary.clone();

    let theme_clone = Arc::new(theme);
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => {
            // Update the shared state
            // This will cause the ThemeProvider to re-render along with all of the other consumers
            consumer.set((*theme_clone).clone());
        }
        _ => {}
    });

    let mut box_style = Style {
        width: StyleProp::Value(Units::Pixels(30.0)),
        height: StyleProp::Value(Units::Pixels(30.0)),
        background_color: StyleProp::Value(theme_primary),
        ..Default::default()
    };

    if theme_name == consumer_theme_name {
        box_style.top = StyleProp::Value(Units::Pixels(3.0));
        box_style.left = StyleProp::Value(Units::Pixels(3.0));
        box_style.bottom = StyleProp::Value(Units::Pixels(3.0));
        box_style.right = StyleProp::Value(Units::Pixels(3.0));
        box_style.width = StyleProp::Value(Units::Pixels(24.0));
        box_style.height = StyleProp::Value(Units::Pixels(24.0));
    }

    rsx! {
        <TooltipConsumer text={theme_name}>
            <Background styles={Some(box_style)} on_event={Some(on_event)} />
        </TooltipConsumer>
    }
}

/// A widget displaying a set of [ThemeButton] widgets
///
/// This is just an abstracted container. Not much to see here...
#[widget]
fn ThemeSelector() {
    let vampire_theme = Theme::vampire();
    let solar_theme = Theme::solar();
    let vector_theme = Theme::vector();

    let button_container_style = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Auto),
        top: StyleProp::Value(Units::Pixels(5.0)),
        ..Default::default()
    };

    rsx! {
        <Element styles={Some(button_container_style)}>
            <ThemeButton theme={vampire_theme} />
            <ThemeButton theme={solar_theme} />
            <ThemeButton theme={vector_theme} />
        </Element>
    }
}

/// A widget that demonstrates the theming in action
///
/// The `is_root` prop just ensures we don't recursively render this widget to infinity
#[widget]
fn ThemeDemo(context: &mut KayakContext, is_root: bool) {
    // Create a consumer
    // This grabs the current theme from the nearest ThemeProvider up the widget tree
    let consumer = context
        .create_consumer::<Theme>()
        .expect("Requires ThemeProvider as an ancestor");
    let theme = consumer.get();

    let select_lbl = if is_root {
        format!("Select Theme (Current: {})", theme.name)
    } else {
        format!("Select A Different Theme (Current: {})", theme.name)
    };

    let select_lbl_style = Style {
        height: StyleProp::Value(Units::Pixels(28.0)),
        ..Default::default()
    };

    let bg_style = Style {
        background_color: StyleProp::Value(theme.background),
        top: StyleProp::Value(Units::Pixels(15.0)),
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let text = "Lorem ipsum dolor...".to_string();
    let text_style = Style {
        color: StyleProp::Value(theme.primary),
        height: StyleProp::Value(Units::Pixels(28.0)),
        ..Default::default()
    };

    let btn_text = "BUTTON".to_string();
    let btn_text_style = Style {
        top: StyleProp::Value(Units::Pixels(4.0)),
        ..Default::default()
    };
    let btn_style = Style {
        background_color: StyleProp::Value(theme.secondary),
        width: StyleProp::Value(Units::Stretch(0.75)),
        height: StyleProp::Value(Units::Pixels(32.0)),
        top: StyleProp::Value(Units::Pixels(5.0)),
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
        padding_left: StyleProp::Value(Units::Stretch(1.0)),
        padding_right: StyleProp::Value(Units::Stretch(1.0)),
        padding_top: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    let nested_style = Style {
        top: StyleProp::Value(Units::Pixels(10.0)),
        left: StyleProp::Value(Units::Pixels(10.0)),
        bottom: StyleProp::Value(Units::Pixels(10.0)),
        right: StyleProp::Value(Units::Pixels(10.0)),
        ..Default::default()
    };

    rsx! {
        <>
            <Text content={select_lbl} size={14.0} styles={Some(select_lbl_style)} />
            <ThemeSelector />

            <Background styles={Some(bg_style)}>

                <Text content={text} size={12.0} styles={Some(text_style)} />
                <Background styles={Some(btn_style)}>
                    <Text content={btn_text} line_height={Some(20.0)} size={14.0} styles={Some(btn_text_style)} />
                </Background>

                <If condition={is_root}>
                    <Element styles={Some(nested_style)}>

                        // This is one of the benefits of the provider/consumer pattern:
                        // We can nest a provider within the context of another provider.
                        // Doing this here allows us to apply alternate theming to the
                        // nested section without having to find it manually within a
                        // global state or resource.
                        <ThemeProvider initial_theme={Theme::vampire()}>
                            <ThemeDemo is_root={false} />
                        </ThemeProvider>

                    </Element>
                </If>

            </Background>
        </>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <TooltipProvider size={(350.0, 350.0)} position={(0.0, 0.0)}>
                    <Window size={(350.0, 350.0)} position={(0.0, 0.0)} title={"Provider Example".to_string()}>
                        <ThemeProvider initial_theme={Theme::vampire()}>
                            <ThemeDemo is_root={true} />
                        </ThemeProvider>
                    </Window>
                </TooltipProvider>
            </App>
        }
    });

    commands.insert_resource(context);
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
