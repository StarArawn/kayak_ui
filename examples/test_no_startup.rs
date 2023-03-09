use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Default, Clone, Copy, PartialEq, Hash, Eq, Debug, States)]
pub enum GameState {
    #[default]
    First,
    Second,
}

fn first_sys(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::Second);
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
        .add_state::<GameState>()
        .add_system(first_sys.in_schedule(OnEnter(GameState::First)))
        .add_system(second_sys.in_schedule(OnEnter(GameState::Second)))
        .run();
}
