use bevy::prelude::*;
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

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <ElementBundle
                styles={KStyle {
                    layout_type: LayoutType::Row.into(),
                    width: Units::Stretch(1.0).into(),
                    ..Default::default()
                }}
            >
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: vec![
                            BoxShadow {
                                color: Color::rgb(38.0 / 255.0, 57.0 / 255.0, 77.0 / 255.0),
                                radius: 30.0,
                                offset: Vec2::new(0.0, 20.0),
                                spread: Vec2::new(0.0, -10.0),
                            }
                        ].into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: vec![
                            BoxShadow {
                                color: Color::rgba(0.0, 0.0, 0.0, 0.1),
                                radius: 12.0,
                                offset: Vec2::new(0.0, 4.0),
                                spread: Vec2::new(0.0, 0.0),
                            }
                        ].into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(136, 165, 191, 0.48) 6px 2px 16px 0px, rgba(255, 255, 255, 0.8) -6px -2px 16px 0px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(0.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgb(85, 91, 255) 0px 0px 0px 3px, rgb(31, 193, 27) 0px 0px 0px 6px, rgb(255, 217, 19) 0px 0px 0px 9px, rgb(255, 156, 85) 0px 0px 0px 12px, rgb(255, 85, 85) 0px 0px 0px 15px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(50, 50, 93, 0.25) 0px 50px 100px -20px, rgba(0, 0, 0, 0.3) 0px 30px 60px -30px;").into(),
                        ..Default::default()
                    }}
                />
            </ElementBundle>
            <ElementBundle
                styles={KStyle {
                    layout_type: LayoutType::Row.into(),
                    width: Units::Stretch(1.0).into(),
                    ..Default::default()
                }}
            >
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(0, 0, 0, 0.2) 0px 60px 40px -7px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(0, 0, 0, 0.16) 0px 3px 6px, rgba(0, 0, 0, 0.23) 0px 3px 6px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(240, 46, 170, 0.4) 5px 5px, rgba(240, 46, 170, 0.3) 10px 10px, rgba(240, 46, 170, 0.2) 15px 15px, rgba(240, 46, 170, 0.1) 20px 20px, rgba(240, 46, 170, 0.05) 25px 25px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(0, 0, 0, 0.15) 1.95px 1.95px 2.6px;").into(),
                        ..Default::default()
                    }}
                />
                <BackgroundBundle
                    styles={KStyle {
                        background_color: Color::WHITE.into(),
                        left: Units::Pixels(50.0).into(),
                        top: Units::Pixels(50.0).into(),
                        border_radius: Corner::all(6.0).into(),
                        width: Units::Pixels(182.0).into(),
                        height: Units::Pixels(182.0).into(),
                        box_shadow: BoxShadow::from_string("box-shadow: rgba(0, 0, 0, 0.09) 0px 2px 1px, rgba(0, 0, 0, 0.09) 0px 4px 2px, rgba(0, 0, 0, 0.09) 0px 8px 4px, rgba(0, 0, 0, 0.09) 0px 16px 8px, rgba(0, 0, 0, 0.09) 0px 32px 16px;").into(),
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
        .insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
