use bevy::{
    prelude::{Bundle, Color, Commands, Component, Entity, In, Query},
    window::CursorIcon,
};
use kayak_ui_macros::rsx;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{
        ComputedStyles, Corner, Edge, KCursorIcon, KPositionType, KStyle, RenderCommand, StyleProp,
        Units,
    },
    widget::Widget,
    widgets::{create_transition, Transition, TransitionEasing, TransitionProps},
};

use super::{
    background::BackgroundBundle,
    clip::ClipBundle,
    text::{TextProps, TextWidgetBundle},
    ElementBundle, TransitionState,
};

#[derive(Component, PartialEq, Clone, Debug)]
pub struct Modal {
    /// The text to display in the modal's title bar
    pub title: String,
    /// A set of styles to apply to the children element wrapper.
    pub children_styles: KStyle,
    /// Is the modal open?
    pub visible: bool,
    /// Animation timeout in milliseconds.
    pub timeout: f32,
    /// The overlay background alpha value
    pub overlay_alpha: f32,
}

impl Default for Modal {
    fn default() -> Self {
        Self {
            title: Default::default(),
            children_styles: Default::default(),
            visible: Default::default(),
            timeout: 250.0,
            overlay_alpha: 0.95,
        }
    }
}

impl Widget for Modal {}

/// Default modal widget
/// A simple widget that renders a modal.
#[derive(Bundle)]
pub struct ModalBundle {
    pub modal: Modal,
    pub styles: KStyle,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for ModalBundle {
    fn default() -> Self {
        Self {
            modal: Default::default(),
            styles: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            widget_name: Modal::default().get_name(),
        }
    }
}

pub fn render(
    In((widget_context, modal_entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&KStyle, &KChildren, &Modal, &mut ComputedStyles)>,
    mut transition_state_query: Query<&mut TransitionState>,
) -> bool {
    if let Ok((modal_styles, modal_children, modal, mut computed_styles)) =
        query.get_mut(modal_entity)
    {
        let styles = KStyle {
            position_type: KPositionType::SelfDirected.into(),
            width: Units::Stretch(1.0).into(),
            height: Units::Stretch(1.0).into(),
            ..Default::default()
        };

        let transition = TransitionProps {
            easing: TransitionEasing::Linear,
            reversing: !modal.visible,
            timeout: modal.timeout,
            looping: false,
            style_a: KStyle {
                opacity: 0.0.into(),
                ..styles.clone()
            },
            style_b: KStyle {
                opacity: 1.0.into(),
                ..styles.clone()
            },
            autoplay: false,
        };
        let transition_entity = create_transition(
            &widget_context,
            &mut commands,
            modal_entity,
            &Transition::new(&transition),
        );

        if let Ok(mut transition_state) = transition_state_query.get_mut(transition_entity) {
            if transition_state.transition.reversing != transition.reversing {
                if transition.reversing {
                    transition_state.transition.start_reverse()
                } else {
                    transition_state.transition.start();
                }

                // Do one update of styles to make sure we start off with the correct styling.
                let new_styles = transition_state.transition.update();
                *computed_styles = ComputedStyles(new_styles);
            }

            // Don't render if nothing is visible.
            if !transition_state.transition.is_playing() && !modal.visible {
                return true;
            }

            let title = modal.title.clone();
            let parent_id = Some(modal_entity);
            rsx! {
                <ElementBundle>
                    <BackgroundBundle
                        styles={KStyle {
                            background_color: Color::rgba(0.0, 0.0, 0.0, modal.overlay_alpha).into(),
                            ..Default::default()
                        }}
                    />
                    <ElementBundle
                        styles={KStyle {
                            background_color: Color::rgba(0.188, 0.203, 0.274, 1.0).into(),
                            border_color: Color::rgba(0.933, 0.745, 0.745, 1.0).into(),
                            border: Edge::all(2.0).into(),
                            border_radius: Corner::all(10.0).into(),
                            render_command: RenderCommand::Quad.into(),
                            position_type: KPositionType::SelfDirected.into(),
                            ..Default::default()
                        }.with_style(modal_styles).into()}
                    >
                        <BackgroundBundle
                            styles={KStyle {
                                cursor: KCursorIcon(CursorIcon::Hand).into(),
                                render_command: RenderCommand::Quad.into(),
                                background_color: Color::rgba(0.188, 0.203, 0.274, 1.0).into(),
                                border_radius:  Corner::all(10.0).into(),
                                height: Units::Pixels(24.0).into(),
                                width: Units::Stretch(1.0).into(),
                                left: Units::Pixels(0.0).into(),
                                right: Units::Pixels(0.0).into(),
                                top: Units::Pixels(0.0).into(),
                                bottom: Units::Pixels(0.0).into(),
                                padding_left: Units::Pixels(5.0).into(),
                                padding_top: Units::Stretch(1.0).into(),
                                padding_bottom: Units::Stretch(1.0).into(),
                                ..KStyle::default()
                            }}
                        >
                            <TextWidgetBundle
                                styles={KStyle {
                                    top: Units::Stretch(1.0).into(),
                                    bottom: Units::Stretch(1.0).into(),
                                    ..Default::default()
                                }}
                                text={TextProps {
                                    content: title,
                                    size: 14.0,
                                    ..Default::default()
                                }}
                            />
                        </BackgroundBundle>
                        <BackgroundBundle
                            styles={KStyle {
                                background_color: StyleProp::Value(Color::rgba(0.239, 0.258, 0.337, 1.0)),
                                width: Units::Stretch(1.0).into(),
                                height: Units::Pixels(2.0).into(),
                                ..Default::default()
                            }}
                        />
                        <ClipBundle
                            styles={modal.children_styles.clone().with_style(KStyle {
                                top: Units::Pixels(10.0).into(),
                                left: Units::Pixels(10.0).into(),
                                right: Units::Pixels(10.0).into(),
                                bottom: Units::Pixels(10.0).into(),
                                ..Default::default()
                            })}
                            children={modal_children.clone()}
                        />
                    </ElementBundle>
                </ElementBundle>
            };
        }
    }

    true
}
