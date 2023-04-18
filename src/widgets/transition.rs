use bevy::prelude::*;
use instant::Instant;
use interpolation::Ease;

pub use interpolation::EaseFunction;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle},
    widget::Widget,
};

#[derive(Component, Clone, PartialEq)]
pub struct Transition {
    /// The easing function that dictates the interpolation factor.
    easing: EaseFunction,
    /// Indicates the direction of the animation
    reversing: bool,
    /// The start time of the animation.
    start: Instant,
    /// The time in milliseconds until the animation completed.
    timeout: f32,
    /// Does the animation loop?
    ///
    /// TODO: Change from boolean to enum that allows disabling the reversing animation.
    looping: bool,
    /// The starting styles of the widget.
    style_a: KStyle,
    /// The ending styles of the widget.
    style_b: KStyle,
}

impl Transition {
    /// Creates a new transition with the given values. Style's that can't be interpolated default to A.
    ///
    /// - transition: The transition props used to create the transition.
    ///
    pub fn new(transition: &TransitionProps) -> Transition {
        Self {
            easing: transition.easing,
            start: Instant::now(),
            reversing: transition.reversing,
            timeout: transition.timeout,
            looping: transition.looping,
            style_a: transition.style_a.clone(),
            style_b: transition.style_b.clone(),
        }
    }

    pub(crate) fn update(&mut self) -> KStyle {
        let elapsed_time = self.start.elapsed().as_secs_f32() * 1000.0; // as Milliseconds
        if elapsed_time < self.timeout {
            let mut x = Ease::calc(elapsed_time / self.timeout, self.easing);
            if self.reversing {
                x = 1.0 - x;
            }
            self.style_a.lerp(&self.style_b, x)
        } else if self.looping {
            // Restart animation
            self.start = Instant::now();
            self.reversing = !self.reversing;
            if self.reversing {
                self.style_b.clone()
            } else {
                self.style_a.clone()
            }
        } else {
            // End of animation just return B.
            self.style_b.clone()
        }
    }

    /// Is the animation currently playing?
    pub fn is_running(&self) -> bool {
        let elapsed_time = self.start.elapsed().as_secs_f32() * 1000.0; // as Milliseconds
        elapsed_time < self.timeout
    }

    /// Starts the animation.
    pub fn start(&mut self) {
        self.reversing = false;
        self.start = Instant::now();
    }

    /// Starts the animation in reverse.
    pub fn start_reverse(&mut self) {
        self.reversing = true;
        self.start = Instant::now();
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            easing: EaseFunction::CubicIn,
            start: Instant::now(),
            reversing: false,
            timeout: 0.0,
            looping: Default::default(),
            style_a: KStyle::default(),
            style_b: KStyle::default(),
        }
    }
}

#[derive(Component, Clone, PartialEq)]
pub struct TransitionState {
    pub transition: Transition,
    pub widget_entity: Entity,
}

impl Default for TransitionState {
    fn default() -> Self {
        Self {
            transition: Default::default(),
            widget_entity: Entity::from_raw(0),
        }
    }
}

pub fn create_transition(
    widget_context: &KayakWidgetContext,
    commands: &mut Commands,
    widget_entity: Entity,
    transition: &Transition,
) -> Entity {
    widget_context.use_state(
        commands,
        widget_entity,
        TransitionState {
            transition: transition.clone(),
            widget_entity,
        },
    )
}

pub fn update_transitions(
    mut query: Query<&mut TransitionState>,
    mut computed_styles_query: Query<&mut ComputedStyles>,
) {
    for mut transition in query.iter_mut() {
        let new_styles = transition.transition.update();
        if let Ok(mut computed_styles) = computed_styles_query.get_mut(transition.widget_entity) {
            *computed_styles = ComputedStyles(new_styles);
        }
    }
}

/// The transition props that represent an animation.
///
/// Note: Styles that can't be interpolated will default to `style_a`.
#[derive(Component, Clone, PartialEq)]
pub struct TransitionProps {
    /// The easing function that dictates the interpolation factor.
    pub easing: EaseFunction,
    /// Indicates the direction of the animation
    pub reversing: bool,
    /// The time in milliseconds until the animation completed.
    pub timeout: f32,
    /// Does the animation loop?
    ///
    /// TODO: Change from boolean to enum that allows disabling the reversing animation.
    pub looping: bool,
    /// The starting styles of the widget.
    pub style_a: KStyle,
    /// The ending styles of the widget.
    pub style_b: KStyle,
}

impl Default for TransitionProps {
    fn default() -> Self {
        Self {
            easing: EaseFunction::CubicInOut,
            reversing: false,
            timeout: 300.0,
            looping: true,
            style_a: Default::default(),
            style_b: Default::default(),
        }
    }
}

impl Widget for TransitionProps {}

/// A generic widget
/// You can consider this to kind behave like a div in html
/// Accepts: KStyle, OnEvent, and KChildren.
#[derive(Bundle)]
pub struct TransitionBundle {
    pub transition: TransitionProps,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for TransitionBundle {
    fn default() -> Self {
        Self {
            transition: Default::default(),
            computed_styles: ComputedStyles::default(),
            children: Default::default(),
            widget_name: TransitionProps::default().get_name(),
        }
    }
}

pub fn render(
    In((widget_context, entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    mut query: Query<(&TransitionProps, &KChildren)>,
    mut transition_state_query: Query<&mut TransitionState>,
) -> bool {
    if let Ok((transition, children)) = query.get_mut(entity) {
        let transition_entity = create_transition(
            &widget_context,
            &mut commands,
            entity,
            &Transition::new(transition),
        );
        if let Ok(mut transition_state) = transition_state_query.get_mut(transition_entity) {
            transition_state.transition.looping = transition.looping;
            transition_state.transition.easing = transition.easing;
            transition_state.transition.style_a = transition.style_a.clone();
            transition_state.transition.style_b = transition.style_b.clone();
            transition_state.transition.timeout = transition.timeout;
        }
        children.process(&widget_context, Some(entity));
    }
    true
}
