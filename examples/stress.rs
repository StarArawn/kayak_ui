use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::Index;
use kayak_ui::core::{
    constructor, render,
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp},
    use_state, widget, Bound, EventType, MutableBound, OnEvent, VecTracker,
};
use kayak_ui::widgets::{App, Window};

mod helper_widgets;
use helper_widgets::FpsWidget;

#[derive(Clone, Default, Debug, PartialEq)]
struct WindowData {
    pub name: String,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[widget]
fn CustomWidget() {
    let (window_data, set_window_data) = use_state!(Vec::<WindowData>::new());

    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..styles.clone().unwrap_or_default()
    });

    let mut window_data_cloned = window_data.clone();
    self.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => {
            for i in 0..100 {
                window_data_cloned.push(WindowData {
                    name: format!("Window {}", window_data_cloned.len() + i),
                    position: (fastrand::i32(0..1270) as f32, fastrand::i32(0..720) as f32),
                    size: (
                        fastrand::i32(10..1270) as f32,
                        fastrand::i32(10..720) as f32,
                    ),
                });
            }
            set_window_data(window_data_cloned.clone());
        }
        _ => {}
    }));

    let tracked_vec = VecTracker::new(window_data.iter().map(|window_data| constructor! {
        <Window position={window_data.position} size={window_data.size} title={window_data.name.clone()} />
    }).collect::<Vec<_>>());
    rsx! {
        <>
            {tracked_vec.clone()}
        </>
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
                <CustomWidget />
                <FpsWidget />
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
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_plugin(helper_widgets::FrameTimePlugin)
        .add_startup_system(startup)
        .run();
}
