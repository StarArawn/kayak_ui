use bevy::{
    prelude::{
        Added, App as BevyApp, AssetServer, Bundle, Changed, Commands, Component, Entity, In, Or,
        ParamSet, Query, Res, ResMut, Vec2, With,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, *};

#[derive(Component, Default)]
struct TextBoxExample;

#[derive(Component, Default)]
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
    In((widget_context, entity)): In<(WidgetContext, Entity)>,
    mut commands: Commands,
    props_query: Query<
        &TextBoxExample,
        Or<(Changed<TextBoxExample>, Changed<KStyle>, With<Mounted>)>,
    >,
    mut state_query: ParamSet<(
        Query<Entity, Or<(Added<TextBoxExampleState>, Changed<TextBoxExampleState>)>>,
        Query<&TextBoxExampleState>,
    )>,
) -> bool {
    if !props_query.is_empty() || !state_query.p0().is_empty() {
        let state_entity = widget_context.get_context_entity::<TextBoxExampleState>(entity);
        if state_entity.is_none() {
            let state_entity = commands
                .spawn(TextBoxExampleState {
                    value1: "Hello World".into(),
                    value2: "Hello World2".into(),
                })
                .id();
            widget_context.set_context_entity::<TextBoxExampleState>(Some(entity), state_entity);
            return false;
        }
        let state_entity = state_entity.unwrap();

        let p1 = state_query.p1();
        let textbox_state = p1.get(state_entity).unwrap();

        let on_change = OnChange::new(
            move |In((_widget_context, _, value)): In<(WidgetContext, Entity, String)>,
                  mut state_query: Query<&mut TextBoxExampleState>| {
                if let Ok(mut state) = state_query.get_mut(state_entity) {
                    state.value1 = value;
                }
            },
        );

        let on_change2 = OnChange::new(
            move |In((_widget_context, _, value)): In<(WidgetContext, Entity, String)>,
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

        return true;
    }

    false
}

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
<<<<<<< HEAD
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <TextBoxExample />
            </App>
        }
    });

    commands.insert_resource(context);
=======
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    widget_context.add_widget_system(
        TextBoxExample::default().get_name(),
        update_text_box_example,
    );
    let parent_id = None;
    rsx! {
        <KayakAppBundle>
            <WindowBundle
                window={KWindow {
                    title: "Hello text box".into(),
                    draggable: true,
                    position: Vec2::new(10.0, 10.0),
                    size: Vec2::new(300.0, 250.0),
                    ..KWindow::default()
                }}
            >
                <TextBoxExampleBundle />
            </WindowBundle>
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
>>>>>>> exp/main
}

fn main() {
    BevyApp::new()
<<<<<<< HEAD
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("UI Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
=======
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
>>>>>>> exp/main
}
