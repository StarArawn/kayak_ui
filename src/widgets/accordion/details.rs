use bevy::prelude::*;
use kayak_ui_macros::rsx;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle, LayoutType, Units},
    widget::Widget,
    widgets::{
        create_transition, BackgroundBundle, ClipBundle, Transition, TransitionEasing,
        TransitionProps, TransitionState,
    },
};

use super::AccordionContext;

#[derive(Component, Default, Debug, Clone, PartialEq)]
pub struct AccordionDetails {
    pub index: usize,
}

impl Widget for AccordionDetails {}

#[derive(Bundle, Debug, Clone, PartialEq)]
pub struct AccordionDetailsBundle {
    pub accordion: AccordionDetails,
    pub children: KChildren,
    pub computed_styles: ComputedStyles,
    pub widget_name: WidgetName,
}

impl Default for AccordionDetailsBundle {
    fn default() -> Self {
        Self {
            accordion: Default::default(),
            children: Default::default(),
            computed_styles: Default::default(),
            widget_name: AccordionDetails::default().get_name(),
        }
    }
}

pub fn render(
    In(accordion_widget): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
    mut commands: Commands,
    mut query: Query<(&AccordionDetails, &KChildren)>,
    context_query: Query<&AccordionContext>,
    mut transition_state_query: Query<&mut TransitionState>,
) -> bool {
    if let Ok((accordion, passed_children)) = query.get_mut(accordion_widget) {
        if let Some(context_entity) =
            widget_context.get_context_entity::<AccordionContext>(accordion_widget)
        {
            if let Ok(context) = context_query.get(context_entity) {
                let mut children_height = 0.0;
                if let Some(parent_layout) = widget_context.get_layout(accordion_widget) {
                    // Start with 20 the size of the margin for the parent.
                    let mut previous_y = parent_layout.posy - 20.0;
                    for child in passed_children.iter() {
                        let layout = widget_context.get_layout(*child).unwrap_or_default();
                        children_height += layout.posy - previous_y;
                        children_height += layout.height;
                        previous_y = layout.posy + layout.height;
                    }
                }

                let transition_props = TransitionProps {
                    easing: TransitionEasing::QuadraticInOut,
                    reversing: !context.is_open(accordion.index),
                    timeout: 500.0,
                    looping: false,
                    style_a: KStyle {
                        height: Units::Pixels(0.0).into(),
                        ..Default::default()
                    },
                    style_b: KStyle {
                        height: Units::Pixels(children_height).into(),
                        ..Default::default()
                    },
                    autoplay: false, // When the animation first initializes we want it to be at the end of the animation.
                };

                let transition_entity: Entity = create_transition(
                    &widget_context,
                    &mut commands,
                    accordion_widget,
                    &Transition::new(&transition_props),
                );

                if let Ok(mut transition_state) = transition_state_query.get_mut(transition_entity)
                {
                    transition_state.transition.style_b.height =
                        Units::Pixels(children_height).into();
                    if transition_state.transition.reversing != transition_props.reversing {
                        transition_state.transition.style_b.height =
                            Units::Pixels(children_height).into();
                        if transition_props.reversing {
                            transition_state.transition.start_reverse()
                        } else {
                            transition_state.transition.start();
                        }
                    }
                }

                let parent_id = Some(accordion_widget);
                rsx! {
                    <BackgroundBundle
                        styles={KStyle {
                            background_color: Color::rgba(0.133, 0.145, 0.2, 1.0).into(),
                            layout_type: LayoutType::Row.into(),
                            height: Units::Stretch(1.0).into(),
                            ..Default::default()
                        }}
                    >
                        <ClipBundle
                            styles={KStyle {
                                top: Units::Pixels(10.0).into(),
                                left: Units::Pixels(10.0).into(),
                                right: Units::Pixels(10.0).into(),
                                bottom: Units::Pixels(10.0).into(),
                                ..Default::default()
                            }}
                            children={passed_children.clone()}
                        />
                    </BackgroundBundle>
                };
            }
        }
    }

    true
}
