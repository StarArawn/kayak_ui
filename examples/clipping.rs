use bevy::prelude::*;
use kayak_ui::prelude::{widgets::*, KStyle, *};

fn startup(
    mut commands: Commands,
    mut font_mapping: ResMut<FontMapping>,
    asset_server: Res<AssetServer>,
) {
    font_mapping.set_default(asset_server.load("lato-light.kayak_font"));

    commands.spawn(UICameraBundle::new());

    let image = asset_server.load("panel.png");

    let mut widget_context = KayakRootContext::new();
    let parent_id = None;

    let nine_patch_styles = KStyle {
        width: StyleProp::Value(Units::Pixels(512.0)),
        height: StyleProp::Value(Units::Pixels(512.0)),
        offset: StyleProp::Value(Edge::all(Units::Stretch(1.0))),
        padding: StyleProp::Value(Edge::all(Units::Pixels(25.0))),
        ..KStyle::default()
    };

    let lorem_ipsum = r#"
Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras sed tellus neque. Proin tempus ligula a mi molestie aliquam. Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus. Nullam venenatis consequat ultricies. Sed ac orci purus. Nullam velit nisl, dapibus vel mauris id, dignissim elementum sapien. Vestibulum faucibus sapien ut erat bibendum, id lobortis nisi luctus. Mauris feugiat at lectus at pretium. Pellentesque vitae finibus ante. Nulla non ex neque. Cras varius, lorem facilisis consequat blandit, lorem mauris mollis massa, eget consectetur magna sem vel enim. Nam aliquam risus pulvinar, volutpat leo eget, eleifend urna. Suspendisse in magna sed ligula vehicula volutpat non vitae augue. Phasellus aliquam viverra consequat. Nam rhoncus molestie purus, sed laoreet neque imperdiet eget. Sed egestas metus eget sodales congue.
                                    
 Sed vel ante placerat, posuere lacus sit amet, tempus enim. Cras ullamcorper ex vitae metus consequat, a blandit leo semper. Nunc lacinia porta massa, a tempus leo laoreet nec. Sed vel metus tincidunt, scelerisque ex sit amet, lacinia dui. In sollicitudin pulvinar odio vitae hendrerit. Maecenas mollis tempor egestas. Nulla facilisi. Praesent nisi turpis, accumsan eu lobortis vestibulum, ultrices id nibh. Suspendisse sed dui porta, mollis elit sed, ornare sem. Cras molestie est libero, quis faucibus leo semper at.
                                    
 Nulla vel nisl rutrum, fringilla elit non, mollis odio. Donec convallis arcu neque, eget venenatis sem mattis nec. Nulla facilisi. Phasellus risus elit, vehicula sit amet risus et, sodales ultrices est. Quisque vulputate felis orci, non tristique leo faucibus in. Duis quis velit urna. Sed rhoncus dolor vel commodo aliquet. In sed tempor quam. Nunc non tempus ipsum. Praesent mi lacus, vehicula eu dolor eu, condimentum venenatis diam. In tristique ligula a ligula dictum, eu dictum lacus consectetur. Proin elementum egestas pharetra. Nunc suscipit dui ac nisl maximus, id congue velit volutpat. Etiam condimentum, mauris ac sodales tristique, est augue accumsan elit, ut luctus est mi ut urna. Mauris commodo, tortor eget gravida lacinia, leo est imperdiet arcu, a ullamcorper dui sapien eget erat.
                                
 Vivamus pulvinar dui et elit volutpat hendrerit. Praesent luctus dolor ut rutrum finibus. Fusce ut odio ultrices, laoreet est at, condimentum turpis. Morbi at ultricies nibh. Mauris tempus imperdiet porta. Proin sit amet tincidunt eros. Quisque rutrum lacus ac est vehicula dictum. Pellentesque nec augue mi.
                                
 Vestibulum rutrum imperdiet nisl, et consequat massa porttitor vel. Ut velit justo, vehicula a nulla eu, auctor eleifend metus. Ut egestas malesuada metus, sit amet pretium nunc commodo ac. Pellentesque gravida, nisl in faucibus volutpat, libero turpis mattis orci, vitae tincidunt ligula ligula ut tortor. Maecenas vehicula lobortis odio in molestie. Curabitur dictum elit sed arcu dictum, ut semper nunc cursus. Donec semper felis non nisl tincidunt elementum.
    "#.to_string();

    rsx! {
        <KayakAppBundle>
            <NinePatchBundle
                nine_patch={NinePatch {
                    handle: image,
                    border: Edge::all(30.0),
                }}
                styles={nine_patch_styles}
            >
                <ClipBundle>
                    <TextWidgetBundle
                        text={TextProps {
                            content: lorem_ipsum,
                            size: 14.0,
                            ..Default::default()
                        }}
                    />
                </ClipBundle>
            </NinePatchBundle>
        </KayakAppBundle>
    }
    commands.insert_resource(widget_context);
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugin(KayakContextPlugin)
        .add_plugin(KayakWidgets)
        .add_startup_system(startup)
        .run()
}
