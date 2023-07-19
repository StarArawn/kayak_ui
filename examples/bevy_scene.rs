use bevy::{
    math::{Vec3Swizzles, Vec4Swizzles},
    prelude::*,
    window::PrimaryWindow,
};
use kayak_ui::prelude::{widgets::*, *};

const TILE_SIZE: Vec2 = Vec2::from_array([50.0, 50.0]);
const COLORS: &[Color] = &[Color::TEAL, Color::MAROON, Color::INDIGO];

// ! === Unnecessary Details Below === ! //
// Below this point are mainly implementation details. The main purpose of this example is to show how to know
// when to allow or disallow world interaction through `BevyContext` (see the `set_active_tile_target` function)

/// A resource used to control the color of the tiles
#[derive(Resource)]
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

/// This is the system that sets the active tile's target position
///
/// To prevent the tile from being moved to a position under our UI, we can use the `BevyContext` resource
/// to filter out clicks that occur over the UI
fn set_active_tile_target(
    mut tile: Query<&mut ActiveTile>,
    cursor: Res<Input<MouseButton>>,
    event_context: Query<&EventDispatcher, With<GameUI>>,
    camera_transform: Query<&GlobalTransform, With<WorldCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if !cursor.just_pressed(MouseButton::Left) {
        // Only run this system when the mouse button is clicked
        return;
    }

    if event_context.single().contains_cursor() {
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

    let world_pos = cursor_to_world(window.single(), camera_transform.single());
    let tile_pos = world_to_tile(world_pos);
    let mut tile = tile.single_mut();
    tile.target = tile_pos;
}

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
    event_context: Query<&EventDispatcher, With<GameUI>>,
    mut tile: Query<&mut Transform, With<GhostTile>>,
    mut cursor_moved: EventReader<CursorMoved>,
    camera_transform: Query<&GlobalTransform, With<WorldCamera>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    for _ in cursor_moved.iter() {
        if !event_context.single().contains_cursor() {
            let world_pos = cursor_to_world(window.single(), camera_transform.single());
            let tile_pos = world_to_tile(world_pos);
            let mut ghost = tile.single_mut();
            ghost.translation.x = tile_pos.x;
            ghost.translation.y = tile_pos.y;
        }
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
        .spawn(SpriteBundle {
            sprite: Sprite {
                color: COLORS[active_color.index],
                custom_size: Some(TILE_SIZE),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(ActiveTile::default());
    commands
        .spawn(SpriteBundle {
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
fn cursor_to_world(window: &Window, camera_transform: &GlobalTransform) -> Vec2 {
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
fn ghost_color(color: Color) -> Color {
    let mut c = color;
    c.set_a(0.35);
    c
}

#[derive(Component)]
pub struct GameUI;

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    // The UI Camera and the world camera are the same.
    // CameraUIKayak is used to tell kayak which camera should render UI.
    let camera_entity = commands
        .spawn((Camera2dBundle::default(), CameraUIKayak, WorldCamera))
        .id();

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let mut widget_context = KayakRootContext::new(camera_entity);
    widget_context.add_plugin(KayakWidgetsContextPlugin);

    let handle_change_color = OnEvent::new(
        move |In(_entity): In<Entity>,
              event: Res<KEvent>,
              mut active_color: ResMut<ActiveColor>| {
            if let EventType::LeftClick(..) = event.event_type {
                active_color.index = (active_color.index + 1) % COLORS.len();
            }
        },
    );

    let text_styles = KStyle {
        left: StyleProp::Value(Units::Stretch(1.0)),
        right: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };
    let button_styles = KStyle {
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

    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    draggable: true,
                    initial_position: Vec2::new(50.0, 50.0),
                    size: Vec2::new(300.0, 200.0),
                    title: "Square Mover: The Game".to_string(),
                    ..Default::default()
                }}
            >
                <TextWidgetBundle
                    styles={text_styles.clone()}
                    text={TextProps {
                        size: 13.0,
                        content: "You can check if the cursor is over the UI or on a focusable widget using the BevyContext resource.".to_string(),
                        ..Default::default()
                    }}
                />
                <KButtonBundle
                    button={KButton {
                        text: "Change Tile Color".into(),
                    }}
                    on_event={handle_change_color}
                    styles={button_styles}
                />
                <TextWidgetBundle
                    styles={KStyle {
                        top: Units::Pixels(10.0).into(),
                        ..text_styles
                    }}
                    text={TextProps {
                        size: 11.0,
                        content: "Go ahead and click the button! The tile won't move.".to_string(),
                        ..Default::default()
                    }}
                />
            </WindowBundle>
        </KayakAppBundle>
    };

    commands.spawn((widget_context, EventDispatcher::default(), GameUI));
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(ActiveColor { index: 0 })
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .add_startup_system(world_setup)
        .add_system(move_ghost_tile)
        .add_system(set_active_tile_target)
        .add_system(move_active_tile)
        .add_system(on_color_change)
        .run()
}
