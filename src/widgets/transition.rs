use bevy::prelude::*;
use instant::Instant;
use interpolation::Ease;
use interpolation::EaseFunction;

use crate::{
    children::KChildren,
    context::WidgetName,
    prelude::KayakWidgetContext,
    styles::{ComputedStyles, KStyle},
    widget::Widget,
};

#[derive(Copy, Clone, PartialEq)]
pub enum TransitionEasing {
    Linear,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    QuarticIn,
    QuarticOut,
    QuarticInOut,
    QuinticIn,
    QuinticOut,
    QuinticInOut,
    SineIn,
    SineOut,
    SineInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,
    ElasticIn,
    ElasticOut,
    ElasticInOut,
    BackIn,
    BackOut,
    BackInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl TransitionEasing {
    fn try_into_easing_function(&self) -> Option<EaseFunction> {
        match self {
            TransitionEasing::QuadraticIn => Some(EaseFunction::QuadraticIn),
            TransitionEasing::QuadraticOut => Some(EaseFunction::QuadraticOut),
            TransitionEasing::QuadraticInOut => Some(EaseFunction::QuadraticInOut),
            TransitionEasing::CubicIn => Some(EaseFunction::CubicIn),
            TransitionEasing::CubicOut => Some(EaseFunction::CubicOut),
            TransitionEasing::CubicInOut => Some(EaseFunction::CubicInOut),
            TransitionEasing::QuarticIn => Some(EaseFunction::QuarticIn),
            TransitionEasing::QuarticOut => Some(EaseFunction::QuarticOut),
            TransitionEasing::QuarticInOut => Some(EaseFunction::QuarticInOut),
            TransitionEasing::QuinticIn => Some(EaseFunction::QuinticIn),
            TransitionEasing::QuinticOut => Some(EaseFunction::QuinticOut),
            TransitionEasing::QuinticInOut => Some(EaseFunction::QuinticInOut),
            TransitionEasing::SineIn => Some(EaseFunction::SineIn),
            TransitionEasing::SineOut => Some(EaseFunction::SineOut),
            TransitionEasing::SineInOut => Some(EaseFunction::SineInOut),
            TransitionEasing::CircularIn => Some(EaseFunction::CircularIn),
            TransitionEasing::CircularOut => Some(EaseFunction::CircularOut),
            TransitionEasing::CircularInOut => Some(EaseFunction::CircularInOut),
            TransitionEasing::ExponentialIn => Some(EaseFunction::ExponentialIn),
            TransitionEasing::ExponentialOut => Some(EaseFunction::ExponentialOut),
            TransitionEasing::ExponentialInOut => Some(EaseFunction::ExponentialInOut),
            TransitionEasing::ElasticIn => Some(EaseFunction::ElasticIn),
            TransitionEasing::ElasticOut => Some(EaseFunction::ElasticOut),
            TransitionEasing::ElasticInOut => Some(EaseFunction::ElasticInOut),
            TransitionEasing::BackIn => Some(EaseFunction::BackIn),
            TransitionEasing::BackOut => Some(EaseFunction::BackOut),
            TransitionEasing::BackInOut => Some(EaseFunction::BackInOut),
            TransitionEasing::BounceIn => Some(EaseFunction::BounceIn),
            TransitionEasing::BounceOut => Some(EaseFunction::BounceOut),
            TransitionEasing::BounceInOut => Some(EaseFunction::BounceInOut),
            _ => None,
        }
    }
}

#[derive(Component, Clone, PartialEq)]
pub struct Transition {
    pub playing: bool,
    /// The easing function that dictates the interpolation factor.
    pub easing: TransitionEasing,
    /// Indicates the direction of the animation
    pub reversing: bool,
    /// The start time of the animation.
    start: Instant,
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

impl Transition {
    /// Creates a new transition with the given values. Style's that can't be interpolated default to A.
    ///
    /// - transition: The transition props used to create the transition.
    ///
    pub fn new(transition: &TransitionProps) -> Transition {
        Self {
            playing: transition.autoplay,
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
                                                                        // dbg!(elapsed_time, self.timeout, self.reversing, self.playing);
        if (elapsed_time < self.timeout) && self.playing {
            let mut x = if let Some(easing) = self.easing.try_into_easing_function() {
                Ease::calc((elapsed_time / self.timeout).clamp(0.0, 1.0), easing)
            } else {
                (elapsed_time / self.timeout).clamp(0.0, 1.0)
            };
            if self.reversing {
                x = 1.0 - x;
            }
            self.style_a.lerp(&self.style_b, x)
        } else if self.looping && self.playing {
            // Restart animation
            self.start = Instant::now();
            self.reversing = !self.reversing;
            if self.reversing {
                self.style_b.clone()
            } else {
                self.style_a.clone()
            }
        } else {
            // End of animation
            self.playing = false;
            if self.reversing {
                self.style_a.clone()
            } else {
                self.style_b.clone()
            }
        }
    }

    /// Is the animation currently playing?
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Starts the animation.
    pub fn start(&mut self) {
        self.reversing = false;
        self.start = Instant::now();
        self.playing = true;
    }

    /// Starts the animation in reverse.
    pub fn start_reverse(&mut self) {
        self.reversing = true;
        self.start = Instant::now();
        self.playing = true;
    }
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            playing: false,
            easing: TransitionEasing::Linear,
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
    pub easing: TransitionEasing,
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
    // Should the animation start playing instantly.
    pub autoplay: bool,
}

impl Default for TransitionProps {
    fn default() -> Self {
        Self {
            easing: TransitionEasing::Linear,
            reversing: false,
            timeout: 300.0,
            looping: true,
            style_a: Default::default(),
            style_b: Default::default(),
            autoplay: true,
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
    In(entity): In<Entity>,
    widget_context: Res<KayakWidgetContext>,
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
        children.process(&widget_context, &mut commands, Some(entity));
    }
    true
}
