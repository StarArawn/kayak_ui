use bevy::{
    prelude::App as BevyApp,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{bind, render, rsx, widget, Binding, Bound, MutableBound};
use kayak_ui::widgets::{App, Text, Window};

fn main() {
    BevyApp::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(setup)
        .add_system(cube_rotator_system)
        .add_system(count_up)
        .run();
}

#[derive(Clone, PartialEq)]
struct GlobalCount(pub u32);

#[widget]
fn Counter() {
    let global_count = context
        .query_world::<Res<Binding<GlobalCount>>, _, _>(move |global_count| global_count.clone());

    context.bind(&global_count);

    let global_count = global_count.get().0;

    rsx! {
        <>
            <Window position={(50.0, 50.0)} size={(300.0, 300.0)} title={"Counter Example".to_string()}>
                <Text size={32.0} content={format!("Current Count: {}", global_count).to_string()}>{}</Text>
            </Window>
        </>
    }
}

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_mapping: ResMut<FontMapping>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1280,
        height: 720,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
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

    commands.spawn_bundle(UICameraBundle {
        camera: Camera {
            priority: -1,
            target: RenderTarget::Image(image_handle.clone()),
            ..Camera::default()
        },
        ..UICameraBundle::new()
    });

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.insert_resource(bind(GlobalCount(0)));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <Counter />
            </App>
        }
    });
    commands.insert_resource(context);

    // Light
    // NOTE: Currently lights are shared between passes - see https://github.com/bevyengine/bevy/issues/3462
    commands.spawn_bundle(PointLightBundle {
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
        .spawn_bundle(PbrBundle {
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
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0))
            .looking_at(Vec3::default(), Vec3::Y),
        ..default()
    });
}

/// Rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<MainPassCube>>) {
    for mut transform in &mut query {
        transform.rotate_x(1.0 * time.delta_seconds());
        transform.rotate_y(0.7 * time.delta_seconds());
    }
}

fn count_up(global_count: Res<Binding<GlobalCount>>) {
    global_count.set(GlobalCount(global_count.get().0 + 1));
}
