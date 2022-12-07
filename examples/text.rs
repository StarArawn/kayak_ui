use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, KStyle, *};

#[derive(Component, Default, Clone, PartialEq, Eq)]
pub struct MyWidgetProps {
    pub foo: u32,
}

fn my_widget_1_render(
    In((_widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    my_resource: Res<MyResource>,
    mut query: Query<(&mut MyWidgetProps, &KStyle, &mut ComputedStyles)>,
) -> bool {
    if let Ok((mut my_widget, style, mut computed_styles)) = query.get_mut(entity) {
        my_widget.foo = my_resource.0;
        dbg!(my_widget.foo);
        // Note: We will see two updates because of the mutable change to styles.
        // Which means when foo changes MyWidget will render twice!
        *computed_styles = KStyle {
            render_command: StyleProp::Value(RenderCommand::Text {
                content: format!("My number is: {}", my_widget.foo),
                alignment: Alignment::Start,
                word_wrap: false,
                subpixel: false,
            }),
            ..Default::default()
        }
        .with_style(style)
        .into();
    }

    true
}

// Our own version of widget_update that handles resource change events.
pub fn widget_update_with_resource<
    Props: PartialEq + Component + Clone,
    State: PartialEq + Component + Clone,
>(
    In((widget_context, entity, previous_entity)): In<(KayakWidgetContext, Entity, Entity)>,
    my_resource: Res<MyResource>,
    widget_param: WidgetParam<Props, State>,
) -> bool {
    widget_param.has_changed(&widget_context, entity, previous_entity) || my_resource.is_changed()
}

impl Widget for MyWidgetProps {}

#[derive(Bundle)]
pub struct MyWidgetBundle {
    props: MyWidgetProps,
    styles: KStyle,
    computed_styles: ComputedStyles,
    widget_name: WidgetName,
}

impl Default for MyWidgetBundle {
    fn default() -> Self {
        Self {
            props: Default::default(),
            styles: Default::default(),
            computed_styles: Default::default(),
            widget_name: MyWidgetProps::default().get_name(),
        }
    }
}

#[derive(Resource)]
pub struct MyResource(pub u32);

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    // Camera 2D forces a clear pass in bevy.
    // We do this because our scene is not rendering anything else.
    commands.spawn(Camera2dBundle::default());

    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    let mut widget_context = KayakRootContext::new();
    widget_context.add_plugin(KayakWidgetsContextPlugin);
    let parent_id = None;
    widget_context.add_widget_data::<MyWidgetProps, EmptyState>();
    widget_context.add_widget_system(
        MyWidgetProps::default().get_name(),
        widget_update_with_resource::<MyWidgetProps, EmptyState>,
        my_widget_1_render,
    );
    rsx! {
        <KayakAppBundle><MyWidgetBundle props={MyWidgetProps { foo: 0 }} /></KayakAppBundle>
    };

    commands.spawn(UICameraBundle::new(widget_context));
}

fn update_resource(keyboard_input: Res<Input<KeyCode>>, mut my_resource: ResMut<MyResource>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        my_resource.0 += 1;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .insert_resource(MyResource(1))
        .add_startup_system(startup)
        .add_system(update_resource)
        .run()
}
