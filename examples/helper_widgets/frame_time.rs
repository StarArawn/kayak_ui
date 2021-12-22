use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{Commands, Plugin, Res, World},
};
use kayak_ui::core::{
    bind, rsx,
    styles::{PositionType, Style, StyleProp, Units},
    widget, Binding, Bound, Children, MutableBound,
};
use kayak_ui::widgets::Text;

#[widget]
pub fn FpsWidget(context: KayakContext, children: Children, styles: Option<Style>) {
    let (fps, frame_time) = {
        let world = context.get_global_state::<World>();
        if world.is_err() {
            return;
        }
        let world = world.unwrap();

        (
            world.get_resource::<Binding<Fps>>().unwrap().clone(),
            world.get_resource::<Binding<FrameTime>>().unwrap().clone(),
        )
    };

    context.bind(&fps);
    context.bind(&frame_time);

    let text_styles = Style {
        position_type: StyleProp::Value(PositionType::SelfDirected),
        top: StyleProp::Value(Units::Pixels(0.0)),
        left: StyleProp::Value(Units::Pixels(0.0)),
        ..styles.clone().unwrap_or_default()
    };

    let frame_time_text_styles = Style {
        top: StyleProp::Value(Units::Pixels(16.0)),
        ..text_styles.clone()
    };

    let fps_text = format!("FPS: {}", fps.get().0.round());
    let frame_time_text = format!(
        "FrameTime: {}",
        (frame_time.get().0 * 10000.0).round() / 10000.0
    );
    rsx! {
        <>
            <Text styles={Some(text_styles)} size={16.0} content={fps_text} />
            <Text styles={Some(frame_time_text_styles)} size={16.0} content={frame_time_text} />
        </>
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fps(f32);
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameTime(f32);

pub fn startup(mut commands: Commands) {
    commands.insert_resource(bind(Fps(0.0)));
    commands.insert_resource(bind(FrameTime(0.0)));
}

pub fn update(
    diagnostics: Res<Diagnostics>,
    fps: Res<Binding<Fps>>,
    frame_time: Res<Binding<FrameTime>>,
) {
    let fps_diagnostics = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).unwrap();
    let frame_time_diagnostics = diagnostics
        .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .unwrap();

    fps.set(Fps(fps_diagnostics.average().unwrap_or(0.0) as f32));
    frame_time.set(FrameTime(
        frame_time_diagnostics.average().unwrap_or(0.0) as f32
    ));
}

pub struct FrameTimePlugin;

impl Plugin for FrameTimePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(startup).add_system(update);
    }
}
