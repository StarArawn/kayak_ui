use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::AsBindGroup,
};
use kayak_ui::{
    prelude::{widgets::*, *},
    CameraUIKayak,
};

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "94c4e6f9-6f10-422c-85ec-6d582d471afc"]
pub struct MyUIMaterial {}
impl MaterialUI for MyUIMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "rainbow_shader.wgsl".into()
    }
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyUIMaterial>>,
) {
    let camera_entity = commands
        .spawn(Camera2dBundle::default())
        .insert(CameraUIKayak)
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let my_material = MyUIMaterial {};
    let my_material_handle = materials.add(my_material);

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                styles={KStyle {
                    position_type: KPositionType::SelfDirected.into(),
                    left: Units::Pixels(20.0).into(),
                    top: Units::Pixels(5.0).into(),
                    material: MaterialHandle::new(move |commands, entity| {
                        commands.entity(entity).insert(my_material_handle.clone_weak());
                    }).into(),
                    ..Default::default()
                }}
                text={TextProps {
                    content: "Hello Shader!".into(),
                    size: 20.0,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default()));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            KayakContextPlugin,
            KayakWidgets,
            MaterialUIPlugin::<MyUIMaterial>::default(),
        ))
        .add_systems(Startup, startup)
        .run()
}
