use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};

use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        render, rsx,
        styles::{Style, StyleProp, Units},
        use_state, widget, Color, EventType, Handler, Index, OnEvent,
    },
    widgets::{App, Background, Button, Fold, If, Inspector, Text, Window},
};

#[widget]
fn FolderTree(context: &mut KayakContext) {
    let text_styles = Style {
        // width: StyleProp::Value(Units::Stretch(1.0)),
        // height: StyleProp::Value(Units::Pixels(18.0)),
        ..Default::default()
    };

    let button_text_styles = Style {
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(22.0)),
        ..Default::default()
    };

    let button_styles = Style {
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Pixels(24.0)),
        background_color: StyleProp::Value(Color::new(0.33, 0.33, 0.33, 1.0)),
        ..Default::default()
    };

    let fold_child_base_styles = Style {
        left: StyleProp::Value(Units::Pixels(5.0)),
        // Children need to be sized
        height: StyleProp::Value(Units::Auto),
        ..Default::default()
    };

    // === Folder A === //
    let fold_a_styles = Some(Style {
        background_color: StyleProp::Value(Color::new(0.25882, 0.24314, 0.19608, 1.0)),
        ..Default::default()
    });
    let fold_a_child_styles = Style {
        background_color: StyleProp::Value(Color::new(0.16863, 0.16863, 0.12549, 1.0)),
        ..fold_child_base_styles.clone()
    };
    let fold_a_child_child_styles = Style {
        background_color: StyleProp::Value(Color::new(0.12941, 0.12941, 0.09412, 1.0)),
        ..fold_a_child_styles.clone()
    };

    // === Folder B === //
    let fold_b_styles = Style {
        background_color: StyleProp::Value(Color::new(0.19608, 0.25882, 0.21569, 1.0)),
        ..Default::default()
    };
    let fold_b_child_styles = Style {
        background_color: StyleProp::Value(Color::new(0.11765, 0.16078, 0.12941, 1.0)),
        ..fold_child_base_styles.clone()
    };

    let (is_b_open, set_b_open, ..) = use_state!(false);
    let set_close_b = set_b_open.clone();
    let close_b = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => set_close_b(false),
        _ => {}
    }));
    let set_open_b = set_b_open.clone();
    let open_b = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => set_open_b(true),
        _ => {}
    }));

    // === Folder C === //
    let fold_c_styles = Some(Style {
        background_color: StyleProp::Value(Color::new(0.25882, 0.19608, 0.23529, 1.0)),
        ..Default::default()
    });
    let fold_c_child_styles = Style {
        background_color: StyleProp::Value(Color::new(0.16863, 0.12549, 0.15294, 1.0)),
        ..fold_child_base_styles.clone()
    };
    let try_style = Style {
        color: StyleProp::Value(Color::new(1.0, 0.5, 0.5, 1.0)),
        ..text_styles.clone()
    };

    let (tried, set_tried, ..) = use_state!(false);
    let on_toggle_c = Some(Handler::new(move |_| {
        set_tried(true);
    }));

    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Fold Example".to_string()}>
                // === Folder A === //
                <Fold label={"Folder A".to_string()} styles={fold_a_styles}>
                    <Fold label={"Folder A1".to_string()} default_open={true} styles={Some(fold_a_child_styles)}>
                        <Background styles={Some(fold_a_child_child_styles)}>
                            <Text styles={Some(text_styles)} size={12.0} content={"I default open".to_string()}>{}</Text>
                        </Background>
                    </Fold>
                    <Fold label={"Folder A2".to_string()} styles={Some(fold_a_child_styles)}>
                        <Background styles={Some(fold_a_child_child_styles)}>
                            <Text styles={Some(text_styles)} size={12.0} content={"I default closed".to_string()}>{}</Text>
                        </Background>
                    </Fold>
                </Fold>
                // === Folder B === //
                <Fold label={"Folder B".to_string()} open={Some(is_b_open)} styles={Some(fold_b_styles)}>
                    <Background styles={Some(fold_b_child_styles)}>
                        <Text styles={Some(text_styles)} size={12.0} content={"The open/close state is manually controlled.".to_string()}>{}</Text>
                        <Text styles={Some(text_styles)} size={12.0} content={"Click the button to close:".to_string()}>{}</Text>
                        <Button on_event={close_b} styles={Some(button_styles)}>
                            <Text styles={Some(button_text_styles)} size={14.0} content={"Close B".to_string()}>{}</Text>
                        </Button>
                    </Background>
                </Fold>
                // === Folder C === //
                <Fold label={"Folder C".to_string()} open={Some(true)} on_change={on_toggle_c} styles={fold_c_styles}>
                    <Background styles={Some(fold_c_child_styles)}>
                        <Text styles={Some(text_styles)} size={12.0} content={"Can't close me!".to_string()}>{}</Text>
                        <If condition={tried}>
                             <Text styles={Some(try_style)} size={12.0} content={"Nice try".to_string()}>{}</Text>
                        </If>
                    </Background>
                </Fold>

                <Button on_event={open_b} styles={Some(button_styles)}>
                    <Text styles={Some(button_text_styles)} size={14.0} content={"Open B".to_string()}>{}</Text>
                </Button>
            </Window>
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
                <FolderTree />
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
