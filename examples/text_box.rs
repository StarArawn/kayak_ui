use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::Color;
use kayak_render_macros::use_state;
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{Style, StyleProp, Units},
    widget, Index,
};
use kayak_ui::widgets::{App, Inspector, OnChange, SpinBox, TextBox, Window};

#[widget]
fn TextBoxExample() {
    let (value, set_value, _) = use_state!("I started with a value!".to_string());
    let (empty_value, set_empty_value, _) = use_state!("".to_string());
    let (red_value, set_red_value, _) = use_state!("This text is red".to_string());
    let (spin_value, set_spin_value, _) = use_state!(3.0f32);

    let input_styles = Style {
        top: StyleProp::Value(Units::Pixels(10.0)),
        ..Default::default()
    };

    let red_text_styles = Style {
        color: StyleProp::Value(Color::new(1., 0., 0., 1.)),
        ..input_styles.clone()
    };

    let on_change = OnChange::new(move |event| {
        set_value(event.value);
    });

    let on_change_empty = OnChange::new(move |event| {
        set_empty_value(event.value);
    });

    let on_change_red = OnChange::new(move |event| {
        set_red_value(event.value);
    });

    let on_change_spin = OnChange::new(move |event| {
        set_spin_value(32.0);
    });

    rsx! {
        <Window position={(50.0, 50.0)} size={(500.0, 300.0)} title={"TextBox Example".to_string()}>
            <TextBox styles={Some(input_styles)} value={value} on_change={Some(on_change)} />
            <TextBox
                styles={Some(input_styles)}
                value={empty_value}
                on_change={Some(on_change_empty)}
                placeholder={Some("This is a placeholder".to_string())}
            />
            <TextBox styles={Some(red_text_styles)} value={red_value} on_change={Some(on_change_red)} />
            <SpinBox<f32> styles={Some(input_styles)} value={spin_value} on_change={Some(on_change_spin)} />
        </Window>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <TextBoxExample />
                <Inspector />
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
