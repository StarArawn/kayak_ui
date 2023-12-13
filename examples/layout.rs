use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

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
            <WindowBundle
                window={KWindow {
                    title: "Layout example".into(),
                    draggable: true,
                    initial_position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(512.0, 512.0),
                    ..KWindow::default()
                }}
            >
                <ElementBundle
                    styles={KStyle{
                        layout_type: LayoutType::Grid.into(),
                        grid_rows: vec![Units::Stretch(1.0), Units::Stretch(2.0), Units::Stretch(5.0)].into(),
                        grid_cols: vec![Units::Stretch(1.0), Units::Stretch(1.0)].into(),
                        ..default()
                    }}
                >
                    <BackgroundBundle
                        styles={KStyle{
                            background_color: Color::rgb(0.4, 0.9, 0.4).into(),
                            color: Color::rgb(0.0, 0.0, 0.0).into(),
                            padding: Edge::all(Units::Pixels(5.0)).into(),
                            border_radius: Corner::all(10.0).into(),
                            row_index: 0.into(),
                            col_index: 0.into(),
                            col_span: 2.into(),
                            layout_type: LayoutType::Row.into(),
                            col_between: Units::Pixels(5.0).into(),
                            ..default()
                        }}
                    >
                        <TextWidgetBundle
                            text={TextProps {
                                content: "A".into(),
                                ..default()
                            }}
                        />
                        <TextWidgetBundle
                            text={TextProps {
                                content: "B".into(),
                                ..default()
                            }}
                        />
                        <TextWidgetBundle
                            text={TextProps {
                                content: "C".into(),
                                ..default()
                            }}
                        />
                        <TextWidgetBundle
                            text={TextProps {
                                content: "D".into(),
                                ..default()
                            }}
                        />
                        <TextWidgetBundle
                            text={TextProps {
                                content: "E".into(),
                                ..default()
                            }}
                        />

                    </BackgroundBundle>
                    <TextWidgetBundle
                        text={TextProps {
                            content: "R1 C0".into(),
                            ..default()
                        }}
                        styles={KStyle{
                            row_index: 1.into(),
                            col_index: 0.into(),
                            ..default()
                        }}
                    />
                    <TextWidgetBundle
                        text={TextProps {
                            content: "R1 C1".into(),
                            ..default()
                        }}
                        styles={KStyle{
                            row_index: 1.into(),
                            col_index: 1.into(),
                            ..default()
                        }}
                    />
                    <TextWidgetBundle
                        text={TextProps {
                            content: "R2 C0".into(),
                            ..default()
                        }}
                        styles={KStyle{
                            row_index: 2.into(),
                            col_index: 0.into(),
                            ..default()
                        }}
                    />
                    <TextWidgetBundle
                        text={TextProps {
                            content: "R2 C1".into(),
                            ..default()
                        }}
                        styles={KStyle{
                            row_index: 2.into(),
                            col_index: 1.into(),
                            ..default()
                        }}
                    />
                </ElementBundle>
            </WindowBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .run()
}
