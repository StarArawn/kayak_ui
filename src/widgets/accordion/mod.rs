use crate::{
    widget::{EmptyState, Widget, widget_update, widget_update_with_context},
    KayakUIPlugin,
};

mod summary;
mod context;
mod details;

pub use context::{AccordionContextProvider, AccordionContext, AccordionContextBundle};
pub use summary::{AccordionSummary, AccordionSummaryBundle};
pub use self::details::{AccordionDetails, AccordionDetailsBundle};

use super::TransitionState;

pub struct AccordionPlugin;
impl KayakUIPlugin for AccordionPlugin {
    fn build(&self, context: &mut crate::context::KayakRootContext) {
        context.add_widget_data::<AccordionContextProvider, EmptyState>();
        context.add_widget_system(
            AccordionContextProvider::default().get_name(),
            widget_update::<AccordionContextProvider, EmptyState>,
            context::render,
        );

        context.add_widget_data::<AccordionSummary, EmptyState>();
        context.add_widget_system(
            AccordionSummary::default().get_name(),
            widget_update_with_context::<AccordionSummary, EmptyState, AccordionContext>,
            summary::render,
        );

        context.add_widget_data::<AccordionDetails, EmptyState>();
        context.add_widget_system(
            AccordionDetails::default().get_name(),
            widget_update_with_context::<AccordionDetails, TransitionState, AccordionContext>,
            details::render,
        );
    }
}
