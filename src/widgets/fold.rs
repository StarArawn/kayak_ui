use crate::core::{
    render_command::RenderCommand,
    derivative::Derivative,
    OnEvent, rsx, WidgetProps,
    styles::{Style, StyleProp, Units},
    use_state, widget, Children, EventType, Handler,
};

use crate::widgets::{Background, Clip, If, Text};

#[derive(WidgetProps, Derivative)]
#[derivative(Default, Debug, PartialEq, Clone)]
pub struct FoldProps {
    pub label: String,
    pub open: Option<bool>,
    pub on_change: Option<Handler<bool>>,
    pub default_open: bool,
    #[props(Styles)]
    pub styles: Option<Style>,
    #[props(Children)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub children: Children,
    #[props(OnEvent)]
    #[derivative(Default(value = "None"), Debug = "ignore", PartialEq = "ignore")]
    pub on_event: Option<OnEvent>,
    #[props(Focusable)]
    #[derivative(Default(value = "Some(true)"), PartialEq = "ignore")]
    pub focusable: Option<bool>,
}

/// A widget container that toggles its content between visible and hidden when clicked
///
/// If `open` is set to `None`, then the toggle state will be automatically handled by
/// the widget. This is useful for if you don't need or care to manage the toggling yourself.
///
/// # Arguments
///
/// * `label`: The Fold's label
/// * `children`: The Fold's content
/// * `open`: If true, renders the content. If `None`, the widget will manage its own open/close state.
/// * `on_change`: Called when the user clicks on the Fold's label. Contains the next desired toggle state.
/// * `default_open`: Set the initial open state of this widget
///
/// # Examples
///
/// ```
/// # use kayak_ui::core::{Handler, rsx, use_state};
/// # use kayak_ui::widgets::{Text};
///
/// let (open, set_open) = use_state!(false);
/// let on_change = Handler::new(move |value| {
///     set_open(value);
/// });
///
/// rsx! {
///     <Fold label={"Toggle Open/Close".to_string()} open={open} on_change={Some(on_change)}>
///         <Text content={"Fold Content".to_string()} size={16.0} />
///     </Fold>
/// }
/// ```
#[widget]
pub fn Fold(props: FoldProps) {
    let FoldProps {default_open, label, on_change, open, ..} = props.clone();

    // === State === //
    let initial = default_open || open.unwrap_or_default();
    let (is_open, set_is_open, ..) = use_state!(initial);
    if let Some(open) = open {
        // This is a controlled state
        set_is_open(open);
    }

    let handler = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click(..) => {
            if open.is_none() {
                // This is an internally-managed state
                set_is_open(!is_open);
            }
            if let Some(ref callback) = on_change {
                callback.call(!is_open);
            }
        }
        _ => {}
    });

    // === Styles === //
    props.styles = Some(Style {
        height: StyleProp::Value(Units::Auto),
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..props.styles.clone().unwrap_or_default()
    });

    let background_styles = Style {
        background_color: StyleProp::Inherit,
        border_radius: StyleProp::Inherit,
        height: StyleProp::Value(Units::Auto),
        ..Default::default()
    };

    let container_style = Style {
        width: StyleProp::Value(Units::Stretch(1.0)),
        height: StyleProp::Value(Units::Auto),
        ..Default::default()
    };

    let inner_container_styles = container_style.clone();

    // === Render === //
    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip styles={Some(container_style)}>
                <Text content={label} on_event={Some(handler)} size={14.0} />
                <If condition={is_open}>
                    <Clip styles={Some(inner_container_styles)}>
                        {children}
                    </Clip>
                </If>
            </Clip>
        </Background>
    }
}
