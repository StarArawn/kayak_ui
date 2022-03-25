use kayak_core::OnLayout;

use crate::core::{
    render_command::RenderCommand,
    rsx,
    styles::{Style, StyleProp, Units},
    use_state, widget, Children, EventType, Handler, OnEvent, WidgetProps,
};

use crate::widgets::{Background, Clip, If, Text};

// TODO: Add `disabled` prop

/// Props used by the [`Fold`] widget
#[derive(WidgetProps, Default, Debug, PartialEq, Clone)]
pub struct FoldProps {
    /// The initial open state of the fold
    pub default_open: bool,
    /// The string displayed as the label of this fold element
    pub label: String,
    /// A callback for when the user presses the fold's label
    ///
    /// The handler is given the boolean value of the desired open state. For example,
    /// if the fold is closed and the user presses on the label, this callback will be
    /// fired with the boolean value `true`.
    pub on_change: Option<Handler<bool>>,
    /// Sets the controlled open state of the fold
    ///
    /// If `None`, the open state will be automatically handled internally.
    /// This is useful for if you don't need or care to manage the toggling yourself.
    pub open: Option<bool>,
    #[prop_field(Styles)]
    pub styles: Option<Style>,
    #[prop_field(Children)]
    pub children: Option<Children>,
    #[prop_field(OnEvent)]
    pub on_event: Option<OnEvent>,
    #[prop_field(OnLayout)]
    pub on_layout: Option<OnLayout>,
    #[prop_field(Focusable)]
    pub focusable: Option<bool>,
}

#[widget]
/// A widget container that toggles its content between visible and hidden when clicked
///
/// # Props
///
/// __Type:__ [`FoldProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
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
pub fn Fold(props: FoldProps) {
    let FoldProps {
        default_open,
        label,
        on_change,
        open,
        ..
    } = props.clone();

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
