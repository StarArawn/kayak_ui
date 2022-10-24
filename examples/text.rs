use bevy::{
    prelude::{
        App as BevyApp, AssetServer, Bundle, Commands, Component, Entity, In, Input, KeyCode,
        Query, Res, ResMut, Resource,
    },
    DefaultPlugins,
};
use kayak_ui::prelude::{widgets::*, KStyle, *};

#[derive(Component, Default, Clone, PartialEq)]
pub struct MyWidgetProps {
    pub foo: u32,
}

fn my_widget_1_update(
    In((_widget_context, entity)): In<(WidgetContext, Entity)>,
    my_resource: Res<MyResource>,
    mut query: Query<(&mut MyWidgetProps, &mut KStyle)>,
) -> bool {
    if my_resource.is_changed() || my_resource.is_added() {
        if let Ok((mut my_widget, mut style)) = query.get_mut(entity) {
            my_widget.foo = my_resource.0;
            dbg!(my_widget.foo);
            style.render_command = StyleProp::Value(RenderCommand::Text {
                content: format!("My number is: {}", my_widget.foo).to_string(),
                alignment: Alignment::Start,
            });
            return true;
        }
    }

    false
}

impl Widget for MyWidgetProps {}

#[derive(Bundle)]
pub struct MyWidgetBundle {
    props: MyWidgetProps,
    styles: KStyle,
    widget_name: WidgetName,
}

impl Default for MyWidgetBundle {
    fn default() -> Self {
        Self {
            props: Default::default(),
            styles: Default::default(),
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
    font_mapping.set_default(asset_server.load("roboto.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let mut widget_context = Context::new();
    let parent_id = None;
    widget_context.add_widget_data::<MyWidgetProps, EmptyState>();
    widget_context.add_widget_system(
        MyWidgetProps::default().get_name(),
        widget_update::<MyWidgetProps, EmptyState>,
        my_widget_1_update,
    );
    rsx! {
        <KayakAppBundle><MyWidgetBundle props={MyWidgetProps { foo: 0 }} /></KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn update_resource(keyboard_input: Res<Input<KeyCode>>, mut my_resource: ResMut<MyResource>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        my_resource.0 += 1;
    }
}

fn main() {
    BevyApp::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ContextPlugin)
        .add_plugin(KayakWidgets)
        .insert_resource(MyResource(1))
        .add_startup_system(startup)
        .add_system(update_resource)
        .run()
}
