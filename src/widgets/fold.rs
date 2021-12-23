use crate::core::{render_command::RenderCommand, styles::{Style, StyleProp, Units}, Children, EventType, rsx, widget, use_state, OnEvent, Bound, MutableBound, OnChange, ChangeEvent};

use crate::widgets::{Background, Clip, If, Text};

/// A widget container that toggles its content between visible and hidden when clicked
///
/// If `on_change` is set to `None`, then the toggle state will be automatically handled by
/// the widget. This is useful for if you don't need or care to manage the toggling yourself.
///
/// # Arguments
///
/// * `label`: The fold's label
/// * `children`: The fold's content
/// * `open`: If true, renders the children
/// * `on_change`: Called when the user clicks on the Fold's label. Contains the next desired
///                toggle state of the Fold (which may be used to change the `open` prop)
///
/// # Examples
///
/// ```
/// # use crate::{rsx, use_state};
///
/// let (open, set_open) = use_state!(false);
/// let on_change = OnChange::new(move |event| {
///     set_open(event.value);
/// });
///
/// rsx! {
///     <Fold label={"Toggle Open/Close".to_string()} open={open} on_change={Some(on_change)}>
///         <Text content={"Fold Content".to_string()} size={16.0} />
///     </Fold>
/// }
/// ```
#[widget]
pub fn Fold(label: String, children: Children, open: bool, on_change: Option<OnChange<bool>>) {

    // === State === //
    let open_clone = open.clone();
    let (is_open, handler) = if let Some(ref on_change) = on_change {
        // This is a controlled state
        let on_change_clone = on_change.clone();
        let handler = OnEvent::new(move |_, event| match event.event_type {
            EventType::Click => {
                on_change_clone.send(ChangeEvent {
                    value: !open_clone
                });
            }
            _ => {}
        });
        (open.clone(), handler)
    } else {
        // This is an internally-managed state
        let (is_open, set_is_open) = use_state!(open_clone);
        let handler = OnEvent::new(move |_, event| match event.event_type {
            EventType::Click => {
                set_is_open(!is_open);
            }
            _ => {}
        });
        (is_open, handler)
    };

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
        height: StyleProp::Value(Units::Auto),
        ..Default::default()
    };

    let text_styles = Style {
        height: StyleProp::Value(Units::Pixels(26.0)),
        ..Default::default()
    };

    let content_styles = Style {
        height: StyleProp::Value(Units::Stretch(1.0)),
        ..Default::default()
    };

    // === Render === //
    let label = label.clone();
    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip styles={Some(container_style)}>
                <Text content={label} on_event={Some(handler)} size={14.0} styles={Some(text_styles)} />
                <If condition={is_open}>
                    <Clip styles={Some(content_styles)}>
                        {children}
                    </Clip>
                </If>
            </Clip>
        </Background>
    }
}
