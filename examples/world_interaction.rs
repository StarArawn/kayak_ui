//! This example showcases how to handle world interactions in a way that considers Kayak.
//!
//! Specifically, it demonstrates how to determine if a click should affect the world or if
//! it should be left to be handled by the UI. This concept is very important when it comes
//! to designing input handling, as an incorrect implementation could lead to unexpected
//! behavior.

use bevy::{
    math::{const_vec2, Vec3Swizzles, Vec4Swizzles},
    prelude::{
        App as BevyApp, AssetServer, Color as BevyColor, Commands, Component, CursorMoved,
        EventReader, GlobalTransform, Input, MouseButton, OrthographicCameraBundle, Query, Res,
        ResMut, Sprite, SpriteBundle, Transform, Vec2, Windows, With, Without,
    },
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::{
    bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle},
    core::{
        render, rsx,
        styles::{Edge, Style, StyleProp, Units},
        use_state, widget, EventType, Index, OnEvent,
    },
    widgets::{App, Button, Text, Window},
};

const TILE_SIZE: Vec2 = const_vec2!([50.0, 50.0]);
const COLORS: &[BevyColor] = &[BevyColor::TEAL, BevyColor::MAROON, BevyColor::INDIGO];

/// This is the system that sets the active tile's target position
///
/// To prevent the tile from being moved to a position under our UI, we can use the `BevyContext` resource
/// to filter out clicks that occur over the UI
fn set_active_tile_target(
    mut tile: Query<&mut ActiveTile>,
    cursor: Res<Input<MouseButton>>,
    context: Res<BevyContext>,
    camera_transform: Query<&GlobalTransform, With<WorldCamera>>,
    windows: Res<Windows>,
) {
    if !cursor.just_pressed(MouseButton::Left) {
        // Only run this system when the mouse button is clicked
        return;
    }

    if context.contains_cursor() {
        // This is the important bit:
        // If the cursor is over a part of the UI, then we should not allow clicks to pass through to the world
        return;
    }

    // If you wanted to allow clicks through the UI as long as the cursor is not on a focusable widget (such as Buttons),
    // you could use `context.wants_cursor()` instead:
    //
    // ```
    // if context.wants_cursor() {
    //     return;
    // }
    // ```

    let world_pos = cursor_to_world(&windows, &camera_transform.single());
    let tile_pos = world_to_tile(world_pos);
    let mut tile = tile.single_mut();
    tile.target = tile_pos;
}

#[widget]
fn ControlPanel() {
    let text_styles = Style {
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };
    let button_styles = Style {
        min_width: StyleProp::Value(Units::Pixels(150.0)),
        width: StyleProp::Value(Units::Auto),
        height: StyleProp::Value(Units::Auto),
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        top: StyleProp::Value(Units::Pixels(16.0)),
        bottom: StyleProp::Value(Units::Pixels(8.0)),
        padding: StyleProp::Value(Edge::axis(Units::Pixels(8.0), Units::Pixels(48.0))),
        ..Default::default()
    };

    let (color_index, set_color_index, ..) = use_state!(0);
    let current_index =
        context.query_world::<Res<ActiveColor>, _, usize>(|active_color| active_color.index);
    if color_index != current_index {
        context.query_world::<ResMut<ActiveColor>, _, ()>(|mut active_color| {
            active_color.index = color_index
        });
    }

    let on_change_color = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => {
            // Cycle the color
            set_color_index((color_index + 1) % COLORS.len());
        }
        _ => {}
    });

    rsx! {
        <>
            <Window draggable={true} position={(50.0, 50.0)} size={(300.0, 200.0)} title={"Square Mover: The Game".to_string()}>
                <Text size={13.0} content={"You can check if the cursor is over the UI or on a focusable widget using the BevyContext resource.".to_string()} styles={Some(text_styles)} />
                <Button on_event={Some(on_change_color)} styles={Some(button_styles)}>
                    <Text size={16.0} content={"Change Tile Color".to_string()} />
                </Button>
                <Text size={11.0} content={"Go ahead and click the button! The tile won't move.".to_string()} styles={Some(text_styles)} />
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
                <ControlPanel />
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
            resizable: false,
            ..Default::default()
        })
        .insert_resource(ActiveColor { index: 0 })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .add_startup_system(world_setup)
        .add_system(move_ghost_tile)
        .add_system(set_active_tile_target)
        .add_system(move_active_tile)
        .add_system(on_color_change)
        .run();
}

// ! === Unnecessary Details Below === ! //
// Below this point are mainly implementation details. The main purpose of this example is to show how to know
// when to allow or disallow world interaction through `BevyContext` (see the `set_active_tile_target` function)

/// A resource used to control the color of the tiles
struct ActiveColor {
    index: usize,
}

/// A component used to control the "Active Tile" that moves to the clicked positions
#[derive(Default, Component)]
struct ActiveTile {
    target: Vec2,
}

/// A component used to control the "Ghost Tile" that follows the user's cursor
#[derive(Component)]
struct GhostTile;

/// A component used to mark the "world camera" (differentiating it from other cameras possibly in the scene)
#[derive(Component)]
struct WorldCamera;

/// A system that moves the active tile to its target position
fn move_active_tile(mut tile: Query<(&mut Transform, &ActiveTile)>) {
    let (mut transform, tile) = tile.single_mut();
    let curr_pos = transform.translation.xy();
    let next_pos = curr_pos.lerp(tile.target, 0.1);
    transform.translation.x = next_pos.x;
    transform.translation.y = next_pos.y;
}

/// A system that moves the ghost tile to the cursor's position
fn move_ghost_tile(
    mut tile: Query<&mut Transform, With<GhostTile>>,
    mut cursor_moved: EventReader<CursorMoved>,
    camera_transform: Query<&GlobalTransform, With<WorldCamera>>,
    windows: Res<Windows>,
) {
    for _ in cursor_moved.iter() {
        let world_pos = cursor_to_world(&windows, &camera_transform.single());
        let tile_pos = world_to_tile(world_pos);
        let mut ghost = tile.single_mut();
        ghost.translation.x = tile_pos.x;
        ghost.translation.y = tile_pos.y;
    }
}

/// A system that updates the tiles' color
fn on_color_change(
    mut active_tile: Query<&mut Sprite, (With<ActiveTile>, Without<GhostTile>)>,
    mut ghost_tile: Query<&mut Sprite, (With<GhostTile>, Without<ActiveTile>)>,
    active_color: Res<ActiveColor>,
) {
    if !active_color.is_changed() {
        return;
    }

    let mut active_tile = active_tile.single_mut();
    active_tile.color = COLORS[active_color.index];

    let mut ghost_tile = ghost_tile.single_mut();
    ghost_tile.color = ghost_color(COLORS[active_color.index]);
}

/// A system that sets up the world
fn world_setup(mut commands: Commands, active_color: Res<ActiveColor>) {
    commands
        .spawn_bundle(OrthographicCameraBundle::new_2d())
        .insert(WorldCamera);
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: COLORS[active_color.index],
                custom_size: Some(TILE_SIZE),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ActiveTile::default());
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: ghost_color(COLORS[active_color.index]),
                custom_size: Some(TILE_SIZE),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(GhostTile);
}

/// Get the world position of the cursor in 2D space
fn cursor_to_world(windows: &Windows, camera_transform: &GlobalTransform) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let size = Vec2::new(window.width(), window.height());

    let mut pos = window.cursor_position().unwrap_or_default();
    pos -= size / 2.0;

    let point = camera_transform.compute_matrix() * pos.extend(0.0).extend(1.0);
    point.xy()
}

/// Converts a world coordinate to a rounded tile coordinate
fn world_to_tile(world_pos: Vec2) -> Vec2 {
    let extents = TILE_SIZE / 2.0;
    let world_pos = world_pos - extents;
    (world_pos / TILE_SIZE).ceil() * TILE_SIZE
}

/// Get the ghost tile color for a given color
fn ghost_color(color: BevyColor) -> BevyColor {
    let mut c = color;
    c.set_a(0.35);
    c
}
