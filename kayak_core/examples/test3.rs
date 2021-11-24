use kayak_core::color::Color;
use kayak_core::context::KayakContext;
use kayak_core::render_command::RenderCommand;
use kayak_core::styles::{Style, StyleProp};
use kayak_core::{rsx, widget, Children, Index};
use morphorm::{PositionType, Units};

#[widget]
fn MyWidget(context: &mut KayakContext, children: Children) {
    let number = *context.create_state::<u32>(0).unwrap();
    let my_styles = Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        width: StyleProp::Value(Units::Pixels(300.0)),
        height: StyleProp::Value(Units::Pixels(300.0)),
        background_color: StyleProp::Value(Color::BLACK),
        ..Style::default()
    };
    rsx! {
        <MyWidget2 styles={Some(my_styles)} test={number}>
            {children}
        </MyWidget2>
    }
}

#[widget]
fn MyWidget2(test: u32, children: Children) {
    dbg!(test);
    rsx! {
        <>
            {children}
        </>
    }
}

fn main() {
    let mut context = KayakContext::new();

    let my_widget = MyWidget {
        id: Index::default(),
        children: None,
        styles: Some(Style {
            position_type: StyleProp::Value(PositionType::SelfDirected),
            width: StyleProp::Value(Units::Pixels(1280.0)),
            height: StyleProp::Value(Units::Pixels(720.0)),
            ..Style::default()
        }),
    };

    let (_, widget_id) = context.widget_manager.create_widget(0, my_widget, None);

    let mut my_widget = context.widget_manager.take(widget_id);
    my_widget.render(&mut context);
    context.set_current_id(widget_id);
    context.set_state::<u32>(1);
    my_widget.render(&mut context);
    context.widget_manager.repossess(my_widget);

    context.widget_manager.render();

    context.widget_manager.calculate_layout();

    dbg!(context.widget_manager.build_render_primitives());
}
