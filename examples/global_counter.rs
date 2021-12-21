use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Res, ResMut, World},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, UICameraBundle};
use kayak_ui::core::{bind, render, rsx, widget, Binding, Bound, Index, MutableBound};
use kayak_ui::widgets::{App, Text, Window};

#[derive(Clone, PartialEq)]
struct GlobalCount(pub u32);

#[widget]
fn Counter(context: &mut KayakContext) {
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

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add(asset_server.load("roboto.kayak_font"));

    commands.insert_resource(bind(GlobalCount(0)));

    let context = BevyContext::new(|context| {
        render! {
            <App>
                <Counter />
            </App>
        }
    });
    commands.insert_resource(context);
}

fn count_up(global_count: Res<Binding<GlobalCount>>) {
    global_count.set(GlobalCount(global_count.get().0 + 1));
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
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .add_system(count_up)
        .run();
}
