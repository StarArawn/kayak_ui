use crate::{
    widget::{widget_update, widget_update_with_context, EmptyState, Widget},
    KayakUIPlugin,
};

mod context;
mod details;
mod summary;

pub use self::details::{AccordionDetails, AccordionDetailsBundle};
pub use context::{AccordionContext, AccordionContextBundle, AccordionContextProvider};
pub use summary::{AccordionSummary, AccordionSummaryBundle};

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
