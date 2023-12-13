use bevy::prelude::*;
use bevy_svg::prelude::Svg;
use kayak_ui::prelude::{widgets::*, KStyle, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak))
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let svg: Handle<Svg> = asset_server.load("kayak.svg");

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <ElementBundle
                styles={KStyle {
                    position_type: StyleProp::Value(KPositionType::SelfDirected),
                    left: StyleProp::Value(Units::Pixels(10.0)),
                    top: StyleProp::Value(Units::Pixels(10.0)),
                    ..Default::default()
                }}
            >
                <KSvgBundle
                    svg={KSvg(svg)}
                    styles={KStyle {
                        width: StyleProp::Value(Units::Pixels(800.0)),
                        height: StyleProp::Value(Units::Pixels(800.0)),
                        ..Default::default()
                    }}
                />
            </ElementBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .insert_resource(Msaa::Sample8)
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            KayakContextPlugin,
            KayakWidgets,
            bevy_svg::prelude::SvgPlugin,
        ))
        .add_systems(Startup, startup)
        .run()
}
