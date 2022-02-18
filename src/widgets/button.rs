use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    widget, Children, Color, Fragment, OnEvent, WidgetProps,
};
use kayak_core::CursorIcon;

/// Props used by the [`Button`] widget
#[derive(Default, Debug, PartialEq, Clone)]
pub struct ButtonProps {
    /// If true, disables this widget not allowing it to be focusable
    ///
    // TODO: Update this documentation when the disabled issue is fixed
    /// Currently, this does not actually disable the button from being clicked.
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
/// A widget that is styled like a button
///
/// # Props
///
/// __Type:__ [`ButtonProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `focusable` | ✅        |
///
pub fn Button(props: ButtonProps) {
    // TODO: This should probably do more than just provide basic styling.
    //       Ideally, we could add a `Handler` prop for `on_click` and other common cursor
    //       events. Giving it the additional purpose of being a compact way to define a button.
    //       This also allows us to make `disable` trule disable the button.
    //       Also, styles need to reflect disabled status.
    props.styles = Some(
        Style::default()
            .with_style(Style {
                render_command: StyleProp::Value(RenderCommand::Quad),
                ..Default::default()
            })
            .with_style(&props.styles)
            .with_style(Style {
                background_color: StyleProp::Value(Color::new(0.0781, 0.0898, 0.101, 1.0)),
                border_radius: StyleProp::Value((5.0, 5.0, 5.0, 5.0)),
                height: StyleProp::Value(Units::Pixels(45.0)),
                padding_left: StyleProp::Value(Units::Stretch(1.0)),
                padding_right: StyleProp::Value(Units::Stretch(1.0)),
                cursor: CursorIcon::Hand.into(),
                ..Default::default()
            }),
    );

    rsx! {
        <Fragment>
            {children}
        </Fragment>
    }
}
