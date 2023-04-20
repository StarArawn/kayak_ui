use bevy::{prelude::*, utils::HashMap};

use crate::{widget::Widget, children::KChildren, prelude::KayakWidgetContext, context::WidgetName, styles::{ComputedStyles, KStyle, RenderCommand, Units}};

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct AccordionContext {
    allow_one: bool,
    accordions: HashMap<usize, bool>,
}

impl AccordionContext {
    pub fn is_open(&self, index: usize) -> bool {
        self.accordions.get(&index).map(|v| *v).unwrap_or(false)
    }

    pub fn toggle_current(&mut self, index: usize) {
        if self.allow_one {
            self.accordions.iter_mut().filter(|(e, _)| **e != index).for_each(|(_, v)| { *v = false; });
        }
        if let Some(open) = self.accordions.get_mut(&index) {
            *open = !*open;
        } else {
            self.accordions.insert(index, true);
        }
    }
}

#[derive(Component, Default, Debug, Clone, PartialEq, Eq)]
pub struct AccordionContextProvider {
    pub allow_only_one: bool,
    pub default_open: Option<usize>, 
}

impl Widget for AccordionContextProvider { }

#[derive(Bundle, Debug, Clone, PartialEq)]
pub struct AccordionContextBundle {
    pub accordion: AccordionContextProvider,
    pub computed_styles: ComputedStyles,
    pub children: KChildren,
    pub widget_name: WidgetName,
}

impl Default for AccordionContextBundle {
    fn default() -> Self {
        Self {
            accordion: Default::default(),
            children: Default::default(),
            computed_styles: ComputedStyles(KStyle {
                render_command: RenderCommand::Layout.into(),
                height: Units::Auto.into(),
                width: Units::Stretch(1.0).into(),
                ..KStyle::default()
            }),
            widget_name: AccordionContextProvider::default().get_name(),
        }
    }
}

pub fn render(
    In((widget_context, widget_entity)): In<(KayakWidgetContext, Entity)>,
    mut commands: Commands,
    children_query: Query<(&AccordionContextProvider, &KChildren)>,
) -> bool {
    if let Ok((accordion, children)) = children_query.get(widget_entity) {
        let context_entity = if let Some(context_entity) =
            widget_context.get_context_entity::<AccordionContext>(widget_entity)
        {
            context_entity
        } else {
            let mut accordion_context = AccordionContext::default();
            accordion_context.allow_one = accordion.allow_only_one;
            if let Some(default_open) = accordion.default_open {
                accordion_context.toggle_current(default_open);
            }
            commands.spawn(accordion_context).id()
        };
        widget_context
            .set_context_entity::<AccordionContext>(Some(widget_entity), context_entity);
        children.process(&widget_context, &mut commands, Some(widget_entity));
    }

    true
}
