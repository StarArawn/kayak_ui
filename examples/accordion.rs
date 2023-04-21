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

    font_mapping.set_default(asset_server.load("fonts/roboto.kttf"));

    let image = asset_server.load("generic-rpg-vendor.png");

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "Accordion Example Window".into(),
                    draggable: true,
                    initial_position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(450.0, 550.0),
                    ..KWindow::default()
                }}
            >
                <ScrollContextProviderBundle>
                    <ScrollBoxBundle>
                        <AccordionContextBundle accordion={AccordionContextProvider { allow_only_one: true, ..Default::default() }}>
                            <AccordionSummaryBundle>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Accordion 1".into(),
                                        size: 18.0,
                                        ..Default::default()
                                    }}
                                />
                            </AccordionSummaryBundle>
                            <AccordionDetailsBundle>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit leo lobortis eget.".into(),
                                        size: 14.0,
                                        ..Default::default()
                                    }}
                                />
                            </AccordionDetailsBundle>
                            <AccordionSummaryBundle accordion={AccordionSummary { index: 1 }}>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Accordion 2".into(),
                                        size: 18.0,
                                        ..Default::default()
                                    }}
                                />
                            </AccordionSummaryBundle>
                            <AccordionDetailsBundle accordion={AccordionDetails { index: 1 }}>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit leo lobortis eget.".into(),
                                        size: 14.0,
                                        ..Default::default()
                                    }}
                                />
                                <KImageBundle
                                    image={KImage(image.clone())}
                                    styles={KStyle {
                                        top: Units::Pixels(10.0).into(),
                                        border_radius: Corner::all(500.0).into(),
                                        width: Units::Pixels(200.0).into(),
                                        height: Units::Pixels(182.0).into(),
                                        ..Default::default()
                                    }}
                                />
                                <KImageBundle
                                    image={KImage(image)}
                                    styles={KStyle {
                                        top: Units::Pixels(10.0).into(),
                                        border_radius: Corner::all(500.0).into(),
                                        width: Units::Pixels(200.0).into(),
                                        height: Units::Pixels(182.0).into(),
                                        ..Default::default()
                                    }}
                                />
                            </AccordionDetailsBundle>
                            <AccordionSummaryBundle accordion={AccordionSummary { index: 2 }}>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Accordion 3".into(),
                                        size: 18.0,
                                        ..Default::default()
                                    }}
                                />
                            </AccordionSummaryBundle>
                            <AccordionDetailsBundle accordion={AccordionDetails { index: 2 }}>
                                <TextWidgetBundle
                                    text={TextProps {
                                        content: "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit leo lobortis eget. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit leo lobortis eget. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Suspendisse malesuada lacus ex, sit amet blandit leo lobortis eget.".into(),
                                        size: 14.0,
                                        ..Default::default()
                                    }}
                                />
                            </AccordionDetailsBundle>
                        </AccordionContextBundle>
                    </ScrollBoxBundle>
                </ScrollContextProviderBundle>
            </WindowBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        // .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_startup_system(startup)
        .run()
}
