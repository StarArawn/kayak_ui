use bevy::{
    prelude::{App as BevyApp, AssetServer, Commands, Handle, Res, ResMut},
    window::WindowDescriptor,
    DefaultPlugins,
};
use kayak_ui::bevy::{BevyContext, BevyKayakUIPlugin, FontMapping, ImageManager, UICameraBundle};
use kayak_ui::core::{
    render,
    styles::{Edge, Style, StyleProp, Units},
    Index,
};
use kayak_ui::widgets::{App, NinePatch, ScrollBox, Text};

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut image_manager: ResMut<ImageManager>,
    mut font_mapping: ResMut<FontMapping>,
) {
    commands.spawn_bundle(UICameraBundle::new());

    font_mapping.add("Roboto", asset_server.load("roboto.kayak_font"));

    let handle: Handle<bevy::render::texture::Image> = asset_server.load("kenny/panel_brown.png");
    let panel_brown_handle = image_manager.get(&handle);

    let context = BevyContext::new(|context| {
        let nine_patch_styles = Style {
            width: StyleProp::Value(Units::Pixels(512.0)),
            height: StyleProp::Value(Units::Pixels(512.0)),
            offset: StyleProp::Value(Edge::all(Units::Stretch(1.0))),
            padding: StyleProp::Value(Edge::all(Units::Pixels(25.0))),
            ..Style::default()
        };

        let lorem_ipsum = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sed tellus neque. Proin tempus ligula a mi molestie aliquam. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam venenatis consequat ultricies. Sed ac orci purus. Nullam velit nisl, dapibus vel mauris id, dignissim elementum sapien. Vestibulum faucibus sapien ut erat bibendum, id lobortis nisi luctus. Mauris feugiat at lectus at pretium. Pellentesque vitae finibus ante. Nulla non ex neque. Cras varius, lorem facilisis consequat blandit, lorem mauris mollis massa, eget consectetur magna sem vel enim. Nam aliquam risus pulvinar, volutpat leo eget, eleifend urna. Suspendisse in magna sed ligula vehicula volutpat non vitae augue. Phasellus aliquam viverra consequat. Nam rhoncus molestie purus, sed laoreet neque imperdiet eget. Sed egestas metus eget sodales congue.

Sed vel ante placerat, posuere lacus sit amet, tempus enim. Cras ullamcorper ex vitae metus consequat, a blandit leo semper. Nunc lacinia porta massa, a tempus leo laoreet nec. Sed vel metus tincidunt, scelerisque ex sit amet, lacinia dui. In sollicitudin pulvinar odio vitae hendrerit. Maecenas mollis tempor egestas. Nulla facilisi. Praesent nisi turpis, accumsan eu lobortis vestibulum, ultrices id nibh. Suspendisse sed dui porta, mollis elit sed, ornare sem. Cras molestie est libero, quis faucibus leo semper at.

Nulla vel nisl rutrum, fringilla elit non, mollis odio. Donec convallis arcu neque, eget venenatis sem mattis nec. Nulla facilisi. Phasellus risus elit, vehicula sit amet risus et, sodales ultrices est. Quisque vulputate felis orci, non tristique leo faucibus in. Duis quis velit urna. Sed rhoncus dolor vel commodo aliquet. In sed tempor quam. Nunc non tempus ipsum. Praesent mi lacus, vehicula eu dolor eu, condimentum venenatis diam. In tristique ligula a ligula dictum, eu dictum lacus consectetur. Proin elementum egestas pharetra. Nunc suscipit dui ac nisl maximus, id congue velit volutpat. Etiam condimentum, mauris ac sodales tristique, est augue accumsan elit, ut luctus est mi ut urna. Mauris commodo, tortor eget gravida lacinia, leo est imperdiet arcu, a ullamcorper dui sapien eget erat.

Vivamus pulvinar dui et elit volutpat hendrerit. Praesent luctus dolor ut rutrum finibus. Fusce ut odio ultrices, laoreet est at, condimentum turpis. Morbi at ultricies nibh. Mauris tempus imperdiet porta. Proin sit amet tincidunt eros. Quisque rutrum lacus ac est vehicula dictum. Pellentesque nec augue mi.

Vestibulum rutrum imperdiet nisl, et consequat massa porttitor vel. Ut velit justo, vehicula a nulla eu, auctor eleifend metus. Ut egestas malesuada metus, sit amet pretium nunc commodo ac. Pellentesque gravida, nisl in faucibus volutpat, libero turpis mattis orci, vitae tincidunt ligula ligula ut tortor. Maecenas vehicula lobortis odio in molestie. Curabitur dictum elit sed arcu dictum, ut semper nunc cursus. Donec semper felis non nisl tincidunt elementum.
        "#.to_string();

        render! {
            <App>
                <NinePatch
                    styles={Some(nine_patch_styles)}
                    border={Edge::all(30.0)}
                    handle={panel_brown_handle}
                >
                    <ScrollBox>
                        <Text content={lorem_ipsum} size={14.0} />
                    </ScrollBox>
                </NinePatch>
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
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(BevyKayakUIPlugin)
        .add_startup_system(startup)
        .run();
}
