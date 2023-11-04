use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use kayak_ui::{
    prelude::{widgets::*, *},
    CameraUIKayak,
};

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

#[derive(Component)]
struct MainUI;

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let size = Extent3d {
        width: 1024,
        height: 1024,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            view_formats: &[TextureFormat::Bgra8UnormSrgb],
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    let camera_entity = commands
        .spawn(Camera2dBundle {
            camera: Camera {
                order: -1,
                target: RenderTarget::Image(image_handle.clone()),
                ..Camera::default()
            },
            camera_2d: Camera2d {
                clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::Default,
            },
            ..Default::default()
        })
        .insert(CameraUIKayak)
        .id();

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle
            styles={KStyle {
                padding: Edge::new(
                    Units::Stretch(1.0),
                    Units::Stretch(0.0),
                    Units::Stretch(1.0),
                    Units::Stretch(0.0),
                ).into(),
                ..Default::default()
            }}
        >
            <TextWidgetBundle
                text={TextProps {
                    size: 150.0,
                    content: "Hello Cube!".into(),
                    alignment: Alignment::Middle,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };
    commands.spawn((widget_context, EventDispatcher::default()));

    // Setup 3D scene
    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
        ..default()
    });

    let cube_size = 4.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    // This material has the texture that has been rendered.
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        reflectance: 0.02,
        unlit: false,
        ..default()
    });

    // Main pass cube, with material containing the rendered first pass texture.
    commands
        .spawn(PbrBundle {
            mesh: cube_handle,
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
                ..default()
            },
            ..default()
        })
        .insert(MainPassCube);

    // The main pass camera.
    let camera_entity = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
                .looking_at(Vec3::default(), Vec3::Y),
            ..default()
        })
        .insert(CameraUIKayak)
        .id();

    // Spawn another UI in 2D space!
    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                text={TextProps {
                    size: 100.0,
                    content: "Hello World!".into(),
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };
    commands.spawn((widget_context, EventDispatcher::default(), MainUI));
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.0 * time.delta_seconds());
        transform.rotate_y(0.7 * time.delta_seconds());
    }
}

fn depsawn_ui(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    ui_query: Query<(Entity, &KayakRootContext), With<MainUI>>,
) {
    if keyboard_input.pressed(KeyCode::Escape) {
        if let Ok((entity, _)) = ui_query.get_single() {
            commands.entity(entity).despawn_descendants();
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((KayakContextPlugin, KayakWidgets))
        .add_systems(Startup, startup)
        .add_systems(Update, (cube_rotator_system, depsawn_ui))
        .run()
}
