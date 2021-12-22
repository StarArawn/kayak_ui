use std::sync::{Arc};
use crate::core::{
    Bound, Children, Color, EventType, MutableBound, OnEvent, rsx, widget,
    render_command::RenderCommand, 
    styles::{PositionType, Style, StyleProp, Units}
};

use crate::widgets::{Background, Clip, Element, If, Text};

#[derive(Clone, PartialEq, Debug, Default)]
pub struct TooltipData {
    /// The anchor coordinates in pixels (x, y)
    ///
    /// If `None`, defaults to cursor position
    pub anchor: Option<(f32, f32)>,
    /// Used to force an update
    ticker: u8,
    /// The size of the tooltip in pixels (width, height)
    pub size: Option<(f32, f32)>,
    /// The text to display
    pub text: String,
    /// Whether the tooltip is visible or not
    pub visible: bool,
}

impl TooltipData {
    /// Force the tooltip to re-render
    pub fn mark_dirty(&mut self) {
        // Hack for forcing the Binding to re-render
        self.ticker = self.ticker.wrapping_add(1);
    }
}


/// A provider for managing a tooltip.
///
/// This widget creates a single tooltip that can be controlled by any descendant [TooltipConsumer].
///
/// # Arguments
///
/// * `position`: The position of the containing rect (used to layout the tooltip).
/// * `size`: The size of the containing rect (used to layout the tooltip).
///
/// # Examples
///
/// ```
/// # use kayak_ui::core::{rsx, widget};
///
/// #[widget]
/// fn MyWidget() {
///   rsx! {
///     <>
///         <TooltipProvider size={Some(1280.0, 720.0)}>
///             // ...
///             <TooltipConsumer text={"Tooltip A".to_string()}>
///                 // ...
///             </TooltipConsumer>
///             <TooltipConsumer text={"Tooltip B".to_string()}>
///                 // ...
///             </TooltipConsumer>
///             // ...
///         </TooltipProvider>
///     </>
///   }
/// }
/// ```
#[widget]
pub fn TooltipProvider(
    children: Children,
    position: (f32, f32),
    size: (f32, f32),
) {
    const WIDTH: f32 = 150.0;
    const HEIGHT: f32 = 18.0;
    const PADDING: (f32, f32) = (10.0, 5.0);

    let tooltip = context.create_provider(TooltipData::default());
    let TooltipData { anchor, size: tooltip_size, text, visible, .. } = tooltip.get();
    let anchor = anchor.unwrap_or(context.last_mouse_position());
    let tooltip_size = tooltip_size.unwrap_or((WIDTH, HEIGHT));


    *styles = Some(Style {
        left: StyleProp::Value(Units::Pixels(position.0)),
        top: StyleProp::Value(Units::Pixels(position.1)),
        width: StyleProp::Value(Units::Pixels(size.0)),
        height: StyleProp::Value(Units::Pixels(size.1)),
        ..styles.clone().unwrap_or_default()
    });

    let mut tooltip_styles = Style {
        position_type: StyleProp::Value(PositionType::SelfDirected),
        background_color: StyleProp::Value(Color::new(0.13, 0.15, 0.17, 0.85)),
        width: StyleProp::Value(Units::Pixels(tooltip_size.0)),
        height: StyleProp::Value(Units::Pixels(tooltip_size.1)),
        ..Style::default()
    };

    if anchor.0 < size.0 / 2.0 {
        tooltip_styles.left = StyleProp::Value(Units::Pixels(anchor.0 + PADDING.0));
    } else {
        // TODO: Replace with `right` (currently not working properly)
        tooltip_styles.left = StyleProp::Value(Units::Pixels(anchor.0 - tooltip_size.0));
    }

    if anchor.1 < size.1 / 2.0 {
        tooltip_styles.top = StyleProp::Value(Units::Pixels(anchor.1 + PADDING.1));
    } else {
        // TODO: Replace with `bottom` (currently not working properly)
        tooltip_styles.top = StyleProp::Value(Units::Pixels(anchor.1 - tooltip_size.1));
    }

    let text_styles = Style {
        width: StyleProp::Value(Units::Pixels(tooltip_size.0)),
        height: StyleProp::Value(Units::Pixels(tooltip_size.1)),
        ..Style::default()
    };

    rsx! {
        <>
            <Element>
                {children}
            </Element>
            <If condition={visible}>
                <Background styles={Some(tooltip_styles)}>
                    <Clip>
                        <Text content={text} size={12.0} styles={Some(text_styles)} />
                    </Clip>
                </Background>
            </If>
        </>
    }
}

/// A consumer of [TooltipProvider], displaying a tooltip when its children are hovered.
///
/// # Arguments
///
/// * `text`: The text to display in the tooltip.
/// * `anchor`: The position (in pixels) to which the tooltip will be anchored. If `None`, defaults to the cursor's position.
/// * `size`: The size (in pixels) of the tooltip.
///
/// # Examples
/// ```
/// # use kayak_ui::core::{rsx, widget};
///
/// #[widget]
/// fn MyWidget() {
///   rsx! {
///     <>
///         <TooltipProvider size={Some(1280.0, 720.0)}>
///             // ...
///             <TooltipConsumer text={"Tooltip A".to_string()}>
///                 // ...
///             </TooltipConsumer>
///             <TooltipConsumer text={"Tooltip B".to_string()}>
///                 // ...
///             </TooltipConsumer>
///             // ...
///         </TooltipProvider>
///     </>
///   }
/// }
/// ```
#[widget]
pub fn TooltipConsumer(
    children: Children,
    text: String,
    anchor: Option<(f32, f32)>,
    size: Option<(f32, f32)>,
) {
    *styles = Some(Style {
        render_command: StyleProp::Value(RenderCommand::Clip),
        width: StyleProp::Value(Units::Auto),
        height: StyleProp::Value(Units::Auto),
        ..styles.clone().unwrap_or_default()
    });

    let data = context.create_consumer::<TooltipData>().expect("TooltipConsumer requires TooltipProvider as an ancestor");

    let text = Arc::new(text);
    let anchor = anchor.unwrap_or(context.last_mouse_position());

    self.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::MouseIn => {
            let mut state = data.get();
            state.visible = true;
            state.text = (*text).clone();
            state.size = size;
            data.set(state);
        }
        EventType::Hover => {
            let mut state = data.get();
            state.mark_dirty();
            state.anchor = Some(anchor);
            data.set(state);
        }
        EventType::MouseOut => {
            let mut state = data.get();
            // Set hidden only if the tooltip's text matches this consumer's
            // Otherwise, it likely got picked up by another widget and should be kept visible
            state.visible = false || state.text != *text;
            data.set(state);
        }
        _ => {}
    }));

    rsx! {
        <>
            {children}
        </>
    }
}