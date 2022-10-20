use bevy::{
    math::Vec2,
    prelude::{
        App as BevyApp, AssetServer, Camera2dBundle, Commands, Component, Handle, Input, KeyCode,
        Query, Res, ResMut, Sprite, SpriteBundle, Transform, With, Without,
    },
    render::color::Color,
    window::WindowDescriptor,
    DefaultPlugins,
};

use kayak_font::{bevy::KayakFontPlugin, Alignment, KayakFont};
use renderer::{FontRenderPlugin, Text};

mod renderer;

const FONT_SIZE: f32 = 24.0;
const INITIAL_SIZE: Vec2 = Vec2::from_array([400.0, 300.0]);
const INITIAL_POS: Vec2 = Vec2::from_array([-200.0, 0.0]);
const INSTRUCTIONS: &str =
    "Press 'A' and 'D' to shrink and grow the text box.\nPress 'Space' to cycle text alignment.";

#[derive(Component)]
struct Instructions;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let font_handle: Handle<KayakFont> = asset_server.load("roboto.kayak_font");

    commands
        .spawn(Text {
            horz_alignment: Alignment::Start,
            color: Color::WHITE,
            content: "Hello World! This text should wrap because it's kinda-super-long. How cool is that?!\nHere's a new line.\n\tHere's a tab.".into(),
            font_size: FONT_SIZE,
            line_height: FONT_SIZE * 1.2, // Firefox method of calculating default line heights see: https://developer.mozilla.org/en-US/docs/Web/CSS/line-height
            position: INITIAL_POS,
            size: INITIAL_SIZE,
        })
        .insert(font_handle.clone());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::DARK_GRAY,
            custom_size: Some(INITIAL_SIZE),
            ..Default::default()
        },
        transform: Transform::from_xyz(
            (INITIAL_SIZE.x / 2.0) + INITIAL_POS.x,
            (-INITIAL_SIZE.y / 4.0) - 20.0,
            -0.05,
        ),
        ..Default::default()
    });

    commands.spawn((
        Text {
            horz_alignment: Alignment::Middle,
            color: Color::WHITE,
            content: INSTRUCTIONS.into(),
            font_size: 32.0,
            line_height: 32.0 * 1.2, // Firefox method of calculating default line heights see: https://developer.mozilla.org/en-US/docs/Web/CSS/line-height
            position: Vec2::new(-360.0, 250.0),
            size: Vec2::new(720.0, 200.0),
        },
        Instructions,
        font_handle.clone(),
    ));
}

fn control_text(
    keyboard_input: ResMut<Input<KeyCode>>,
    mut text_box: Query<&mut Text, Without<Instructions>>,
    mut instructions: Query<&mut Text, With<Instructions>>,
    mut bg: Query<&mut Sprite>,
) {
    let speed =
        if keyboard_input.pressed(KeyCode::LShift) || keyboard_input.pressed(KeyCode::RShift) {
            2.5
        } else {
            1.0
        };

    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut text in text_box.iter_mut() {
            let next = match text.horz_alignment {
                Alignment::Start => Alignment::Middle,
                Alignment::Middle => Alignment::End,
                Alignment::End => Alignment::Start,
            };
            text.horz_alignment = next;
        }
    }

    let speed = if keyboard_input.pressed(KeyCode::D) {
        speed
    } else if keyboard_input.pressed(KeyCode::A) {
        -speed
    } else {
        return;
    };

    for mut text in text_box.iter_mut() {
        text.size.x += speed;
        text.position.x -= speed / 2.0;

        let mut instructions = instructions.single_mut();
        instructions.content = String::from(INSTRUCTIONS);
        instructions
            .content
            .push_str(&format!("\nSize: {}", text.size.x));
    }

    for mut sprite in bg.iter_mut() {
        sprite.custom_size.as_mut().unwrap().x += speed;
    }
}

fn main() {
    BevyApp::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakFontPlugin)
        .add_plugin(FontRenderPlugin)
        .add_startup_system(startup)
        .add_system(control_text)
        .run();
}
