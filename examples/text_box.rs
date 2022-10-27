use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default, Clone, PartialEq)]
struct TextBoxExample;

#[derive(Component, Default, Clone, PartialEq)]
struct TextBoxExampleState {
    pub value1: String,
    pub value2: String,
}

impl Widget for TextBoxExample {}

#[derive(Bundle)]
struct TextBoxExampleBundle {
    text_box_example: TextBoxExample,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for TextBoxExampleBundle {
    fn default() -> Self {
        Self {
            text_box_example: Default::default(),
            styles: Default::default(),
            widget_name: TextBoxExample::default().get_name(),
        }
    }
}

fn update_text_box_example(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    state_query: Query<&TextBoxExampleState>,
) -> bool {
    let state_entity = widget_context.use_state::<TextBoxExampleState>(
        &mut commands,
        entity,
        TextBoxExampleState {
            value1: "Hello World".into(),
            value2: "Hello World2".into(),
        },
    );

    if let Ok(textbox_state) = state_query.get(state_entity) {
        let on_change = OnChange::new(
            move |In((_widget_context, _, value)): In<(KayakWidgetContext, Entity, String)>,
                  mut state_query: Query<&mut TextBoxExampleState>| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    state.value1 = value;
                }
            },
        );

        let on_change2 = OnChange::new(
            move |In((_widget_context, _, value)): In<(KayakWidgetContext, Entity, String)>,
                  mut state_query: Query<&mut TextBoxExampleState>| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    state.value2 = value;
                }
            },
        );

        let parent_id = Some(entity);
        rsx! {
            <ElementBundle>
                <TextBoxBundle
                    styles={KStyle {
                        bottom: StyleProp::Value(Units::Pixels(10.0)),
                        ..Default::default()
                    }}
                    text_box={TextBoxProps { value: textbox_state.value1.clone(), ..Default::default()}}
                    on_change={on_change}
                />
                <TextBoxBundle
                    text_box={TextBoxProps { value: textbox_state.value2.clone(), ..Default::default()}}
                    on_change={on_change2}
                />
            </ElementBundle>
        }
    }
    true
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    // Camera 2D forces a clear pass in bevy.
    // We do this because our scene is not rendering anything else.
    commands.spawn(Camera2dBundle::default());
    commands.spawn(UICameraBundle::new());

    let mut widget_context = KayakRootContext::new();

    widget_context.add_widget_data::<TextBoxExample, TextBoxExampleState>();
    widget_context.add_widget_system(
        TextBoxExample::default().get_name(),
        widget_update::<TextBoxExample, TextBoxExampleState>,
        update_text_box_example,
    );
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "Hello text box".into(),
                    draggable: true,
                    initial_position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(300.0, 250.0),
                    ..KWindow::default()
                }}
            >
                <TextBoxExampleBundle />
            </WindowBundle>
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
