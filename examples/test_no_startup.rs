use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Default, Clone, Copy, PartialEq, Hash, Eq, Debug)]
pub enum GameState {
    #[default]
    First,
    Second,
}

fn first_sys(mut state: ResMut<State<GameState>>) {
    state.overwrite_replace(GameState::Second).unwrap();
}

fn second_sys(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut font_mapping: ResMut<FontMapping>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <TextWidgetBundle
                text={TextProps {
                    content: "Hello World".into(),
                    size: 20.0,
                    ..Default::default()
                }}
            />
        </KayakAppBundle>
    };

    commands.spawn(UICameraBundle::new(widget_context));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_state(GameState::First)
        .add_system_set(SystemSet::on_enter(GameState::First).with_system(first_sys))
        .add_system_set(SystemSet::on_enter(GameState::Second).with_system(second_sys))
        .run()
}
