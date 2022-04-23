use crate::{
    core::{
        render_command::RenderCommand,
        rsx,
        styles::{Corner, Style, Units},
        widget, Bound, Children, Color, EventType, MutableBound, OnEvent, WidgetProps,
    },
    widgets::{Button, ChangeEvent},
};
use kayak_core::{
    styles::{LayoutType, StyleProp},
    CursorIcon, OnLayout,
};

use crate::widgets::{Background, Clip, OnChange, Text};

#[derive(Debug, PartialEq, Clone)]
pub struct SpinBoxProps {
    /// If true, prevents the widget from being focused (and consequently edited)
    pub disabled: bool,
    /// A callback for when the text value was changed
    pub on_change: Option<OnChange>,
    /// The text to display when the user input is empty
    pub placeholder: Option<String>,
    /// The user input
    ///
    /// This is a controlled state. You _must_ set this to the value to you wish to be displayed.
    /// You can use the [`on_change`] callback to update this prop as the user types.
    pub value: String,
    pub styles: Option<Style>,
    /// Text on increment button defaults to `>`
    pub incr_str: String,
    /// Text on decrement button defaults to `<`
    pub decr_str: String,
    pub children: Option<Children>,
    pub on_event: Option<OnEvent>,
    pub on_layout: Option<OnLayout>,
    pub focusable: Option<bool>,
}


impl Default for SpinBoxProps {
    fn default() -> SpinBoxProps {
        SpinBoxProps { 
            incr_str: ">".into(),
            decr_str: "<".into(),
            disabled: Default::default(),
            on_change:  Default::default(),
            placeholder: Default::default(),
            value: Default::default(),
            styles: Default::default(),
            children: Default::default(),
            on_event: Default::default(),
            on_layout: Default::default(),
            focusable: Default::default(),
        }
    }
}

impl WidgetProps for SpinBoxProps {
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

    fn get_on_layout(&self) -> Option<OnLayout> {
        self.on_layout.clone()
    }

    fn get_focusable(&self) -> Option<bool> {
        Some(!self.disabled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FocusSpinbox(pub bool);

#[widget]
/// A widget that displays a spinnable text field
///
/// # Props
///
/// __Type:__ [`SpinBoxProps`]
///
/// | Common Prop | Accepted |
/// | :---------: | :------: |
/// | `children`  | ✅        |
/// | `styles`    | ✅        |
/// | `on_event`  | ✅        |
/// | `on_layout` | ✅        |
/// | `focusable` | ✅        |
///
pub fn SpinBox(props: SpinBoxProps) {
    let SpinBoxProps {
        on_change,
        placeholder,
        value,
        ..
    } = props.clone();

    props.styles = Some(
        Style::default()
            // Required styles
            .with_style(Style {
                render_command: RenderCommand::Layout.into(),
                ..Default::default()
            })
            // Apply any prop-given styles
            .with_style(&props.styles)
            // If not set by props, apply these styles
            .with_style(Style {
                top: Units::Pixels(0.0).into(),
                bottom: Units::Pixels(0.0).into(),
                height: Units::Pixels(26.0).into(),
                cursor: CursorIcon::Text.into(),
                ..Default::default()
            }),
    );

    let background_styles = Style {
        background_color: Color::new(0.176, 0.196, 0.215, 1.0).into(),
        border_radius: Corner::all(5.0).into(),
        height: Units::Pixels(26.0).into(),
        padding_left: Units::Pixels(5.0).into(),
        padding_right: Units::Pixels(5.0).into(),
        ..Default::default()
    };

    let has_focus = context.create_state(FocusSpinbox(false)).unwrap();

    let mut current_value = value.clone();
    let cloned_on_change = on_change.clone();
    let cloned_has_focus = has_focus.clone();

    props.on_event = Some(OnEvent::new(move |_, event| match event.event_type {
        EventType::CharInput { c } => {
            if !cloned_has_focus.get().0 {
                return;
            }
            if is_backspace(c) {
                if !current_value.is_empty() {
                    current_value.truncate(current_value.len() - 1);
                }
            } else if !c.is_control() {
                current_value.push(c);
            }
            if let Some(on_change) = cloned_on_change.as_ref() {
                if let Ok(mut on_change) = on_change.0.write() {
                    on_change(ChangeEvent {
                        value: current_value.clone(),
                    });
                }
            }
        }
        EventType::Focus => cloned_has_focus.set(FocusSpinbox(true)),
        EventType::Blur => cloned_has_focus.set(FocusSpinbox(false)),
        _ => {}
    }));

    let text_styles = if value.is_empty() || (has_focus.get().0 && value.is_empty()) {
        Style {
            color: Color::new(0.5, 0.5, 0.5, 1.0).into(),
            ..Style::default()
        }
    } else {
        Style {
            width: Units::Stretch(100.0).into(),
            ..Style::default()
        }
    };

    let button_style = Some(Style {
        height: Units::Pixels(24.0).into(),
        width: Units::Pixels(24.0).into(),
        ..Default::default()
    });

    let value = if value.is_empty() {
        placeholder.unwrap_or_else(|| value.clone())
    } else {
        value
    };

    let inline_style = Style {
        layout_type: StyleProp::Value(LayoutType::Row),
        ..Style::default()
    };

    let incr_str = props.clone().incr_str;
    let decr_str = props.clone().decr_str;

    rsx! {
        <Background styles={Some(background_styles)}>
            <Clip styles={Some(inline_style)}>
                <Button styles={button_style}>
                    <Text content={decr_str} />
                </Button>
                <Text
                    content={value}
                    size={14.0}
                    styles={Some(text_styles)}
                />
                <Button styles={button_style}>
                    <Text content={incr_str} />
                </Button>
            </Clip>
        </Background>
    }
}

/// Checks if the given character contains the "Backspace" sequence
///
/// Context: [Wikipedia](https://en.wikipedia.org/wiki/Backspace#Common_use)
fn is_backspace(c: char) -> bool {
    c == '\u{8}' || c == '\u{7f}'
}
