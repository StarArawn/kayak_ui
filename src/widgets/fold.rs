use crate::core::{
    render_command::RenderCommand,
    styles::{Style, StyleProp, Units},
    Children, EventType, Handler, rsx, widget, use_state, OnEvent
};

use crate::widgets::{Background, Clip, If, Text};

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
pub fn Fold(label: String, children: Children, open: Option<bool>, on_change: Option<Handler<bool>>, default_open: bool) {

    // === State === //
    let initial = default_open || open.unwrap_or_default();
    let (is_open, set_is_open, ..) = use_state!(initial);
    if let Some(open) = open {
        // This is a controlled state
        set_is_open(open);
    }

    let handler = OnEvent::new(move |_, event| match event.event_type {
        EventType::Click => {
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
    *styles = Some(Style {
        height: StyleProp::Value(Units::Auto),
        render_command: StyleProp::Value(RenderCommand::Layout),
        ..styles.clone().unwrap_or_default()
    });

    let background_styles = Style {
        background_color: StyleProp::Inherit,
        border_radius: StyleProp::Inherit,
        height: StyleProp::Value(Units::Auto),
        ..Default::default()
    };

    let container_style = Style {
        width: StyleProp::Value(Units::Stretch(1.0)),
        // height: StyleProp::Value(Units::Auto),
        height: StyleProp::Value(Units::Pixels(20.0)),
        ..Default::default()
    };

    let text_styles = Style {
        height: StyleProp::Value(Units::Pixels(26.0)),
        ..Default::default()
    };

    // === Render === //
    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip styles={Some(container_style)}>
                <Text content={label} on_event={Some(handler)} size={14.0} styles={Some(text_styles)} />
                <If condition={is_open}>
                    <Clip>
                        {children}
                    </Clip>
                </If>
            </Clip>
        </Background>
    }
}
