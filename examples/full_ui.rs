use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res, ResMut, World},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, ImageManager, UICameraBundle};
use kayak_ui::core::{
    layout_cache::Space,
    render, rsx,
    styles::{LayoutType, Style, StyleProp, Units},
    widget, Bound, Children, EventType, Index, MutableBound, OnEvent,
};
use kayak_ui::widgets::{App, NinePatch, Text};

#[widget]
fn BlueButton(context: KayakContext, children: Children, styles: Option<Style>) {
    let (blue_button_handle, blue_button_hover_handle) = {
        let world = context.get_global_state::<World>();
        if world.is_err() {
            return;
        }

        let mut world = world.unwrap();

        let (handle1, handle2) = {
            let asset_server = world.get_resource::<AssetServer>().unwrap();
            let handle1: Handle<bevy::render::texture::Image> =
                asset_server.load("../assets/kenny/buttonSquare_blue_pressed.png");
            let handle2: Handle<bevy::render::texture::Image> =
                asset_server.load("../assets/kenny/buttonSquare_blue.png");

            (handle1, handle2)
        };

        let mut image_manager = world.get_resource_mut::<ImageManager>().unwrap();
        let blue_button_handle = image_manager.get(&handle1);
        let blue_button_hover_handle = image_manager.get(&handle2);

        (blue_button_handle, blue_button_hover_handle)
    };

    let current_button_handle = context.create_state::<u16>(blue_button_handle).unwrap();

    let button_styles = Style {
        width: StyleProp::Value(Units::Pixels(200.0)),
        height: StyleProp::Value(Units::Pixels(50.0)),
        padding_left: StyleProp::Value(Units::Stretch(1.0)),
        padding_right: StyleProp::Value(Units::Stretch(1.0)),
        padding_top: StyleProp::Value(Units::Stretch(1.0)),
        padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
        ..styles.clone().unwrap_or_default()
    };

    let cloned_current_button_handle = current_button_handle.clone();
    let on_event = OnEvent::new(move |_, event| match event.event_type {
        EventType::MouseIn => {
            cloned_current_button_handle.set(blue_button_hover_handle);
        }
        EventType::MouseOut => {
            cloned_current_button_handle.set(blue_button_handle);
        }
        _ => (),
    });

    rsx! {
        <NinePatch
            border={Space {
                left: 10.0,
                right: 10.0,
                top: 10.0,
                bottom: 10.0,
            }}
            handle={current_button_handle.get()}
            styles={Some(button_styles)}
            on_event={Some(on_event)}
        >
            {children}
        </NinePatch>
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_manager: ResMut<ImageManager>,
    mut font_mapping: ResMut<FontMapping>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));
    let main_font = asset_server.load("antiquity.kayak_font");
    font_mapping.add("Antiquity", main_font.clone());

    let handle: Handle<bevy::render::texture::Image> = asset_server.load("kenny/panel_brown.png");
    let panel_brown_handle = image_manager.get(&handle);

    let context = BevyContext::new(|context| {
        let nine_patch_styles = Style {
            layout_type: StyleProp::Value(LayoutType::Column),
            width: StyleProp::Value(Units::Pixels(512.0)),
            height: StyleProp::Value(Units::Pixels(512.0)),
            left: StyleProp::Value(Units::Stretch(1.0)),
            right: StyleProp::Value(Units::Stretch(1.0)),
            top: StyleProp::Value(Units::Stretch(1.0)),
            bottom: StyleProp::Value(Units::Stretch(1.0)),
            padding_left: StyleProp::Value(Units::Stretch(1.0)),
            padding_right: StyleProp::Value(Units::Stretch(1.0)),
            padding_top: StyleProp::Value(Units::Stretch(1.0)),
            padding_bottom: StyleProp::Value(Units::Stretch(1.0)),
            ..Style::default()
        };

        let header_styles = Style {
            // width: StyleProp::Value(Units::Pixels(408.0)),
            // height: StyleProp::Value(Units::Pixels(42.0)),
            bottom: StyleProp::Value(Units::Stretch(1.0)),
            ..Style::default()
        };

        let text_styles = Style {
            // width: StyleProp::Value(Units::Pixels(56.0)),
            // height: StyleProp::Value(Units::Pixels(24.0)),
            ..Style::default()
        };

        let options_button_text_styles = Style {
            // width: StyleProp::Value(Units::Pixels(94.0)),
            // height: StyleProp::Value(Units::Pixels(24.0)),
            ..Style::default()
        };

        let options_button_styles = Style {
            top: StyleProp::Value(Units::Pixels(15.0)),
            ..Style::default()
        };

        let main_font_id = font_mapping.get(&main_font);

        render! {
            <App>
                <NinePatch
                    styles={Some(nine_patch_styles)}
                    border={Space {
                        left: 30.0,
                        right: 30.0,
                        top: 30.0,
                        bottom: 30.0,
                    }}
                    handle={panel_brown_handle}
                >
                    <Text
                        styles={Some(header_styles)}
                        size={35.0}
                        content={"Name My Game Plz".to_string()}
                        font={main_font_id}
                    />
                    <BlueButton>
                        <Text styles={Some(text_styles)} size={20.0} content={"Play".to_string()} font={main_font_id} />
                    </BlueButton>
                    <BlueButton styles={Some(options_button_styles)}>
                        <Text styles={Some(options_button_text_styles)} size={20.0} content={"Options".to_string()} font={main_font_id} />
                    </BlueButton>
                    <BlueButton styles={Some(options_button_styles)}>
                        <Text styles={Some(text_styles)} size={20.0} content={"Quit".to_string()} font={main_font_id} />
                    </BlueButton>
                </NinePatch>
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
