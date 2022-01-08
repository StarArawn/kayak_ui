use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_core::Color;
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{
    render, rsx,
    styles::{Style, StyleProp, Units},
    widget, Bound, Index, MutableBound,
};
use kayak_ui::widgets::{App, OnChange, TextBox, Window};

#[widget]
fn TextBoxExample(context: &mut KayakContext) {
    let value = context
        .create_state("I started with a value!".to_string())
        .unwrap();
    let red_value = context
        .create_state("This text is red".to_string())
        .unwrap();
    let empty_value = context.create_state("".to_string()).unwrap();

    let input_styles = Style {
        top: StyleProp::Value(Units::Pixels(10.0)),
        ..Default::default()
    };

    let red_text_styles = Style {
        color: StyleProp::Value(Color::new(1., 0., 0., 1.)),
        ..input_styles.clone()
    };

    let cloned_value = value.clone();
    let on_change = OnChange::new(move |event| {
        cloned_value.set(event.value);
    });

    let cloned_value2 = empty_value.clone();
    let on_change2 = OnChange::new(move |event| {
        cloned_value2.set(event.value);
    });

    let cloned_red_value = red_value.clone();
    let on_change_red = OnChange::new(move |event| {
        cloned_red_value.set(event.value);
    });

    let current_value = value.get();
    let current_value2 = empty_value.get();
    let current_red_value = red_value.get();
    rsx! {
        <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"TextBox Example".to_string()}>
            <TextBox styles={Some(input_styles)} value={current_value} on_change={Some(on_change)} />
            <TextBox
                styles={Some(input_styles)}
                value={current_value2}
                on_change={Some(on_change2)}
                placeholder={Some("This is a placeholder".to_string())}
            />
            <TextBox styles={Some(red_text_styles)} value={current_red_value} on_change={Some(on_change_red)} />
        </Window>
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <TextBoxExample />
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
