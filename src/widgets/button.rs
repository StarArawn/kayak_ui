use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Children, Color, Fragment, OnEvent, WidgetProps,
};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct ButtonProps {
    pub disabled: bool,
    pub styles: Option<Style>,
    pub children: Option<Children>,
    pub on_event: Option<OnEvent>,
    pub focusable: Option<bool>,
}

impl WidgetProps for ButtonProps {
    fn get_children(&self) -> Option<Children> {
        self.children.clone()
    }

    fn set_children(&mut self, children: Option<Children>) {
        self.children = children;
    }

    fn get_styles(&self) -> Option<Style> {
        self.styles.clone()
    }

    fn get_on_event(&self) -> Option<OnEvent> {
        self.on_event.clone()
    }

    fn get_focusable(&self) -> Option<bool> {
        Some(!self.disabled)
    }
}

#[widget]
pub fn Button(props: ButtonProps) {
    let base_styles = props.styles.clone().unwrap_or_default();
    props.styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Quad),
        border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
        height: if base_styles.height == StyleProp::Default {
            StyleProp::Value(Units::Pixels(45.0))
        } else {
            base_styles.height
        },
        background_color: if matches!(base_styles.background_color, StyleProp::Default) {
            StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0))
        } else {
            base_styles.background_color
        },
        padding_left: StyleProp::Value(Units::Stretch(1.0)),
        padding_right: StyleProp::Value(Units::Stretch(1.0)),
        ..base_styles
    });
    rsx! {
        <Fragment>
            {children}
        </Fragment>
    }
}
