use proc_macro2::TokenStream;
use quote::quote;

pub fn build_arc_function(
    widget_name: TokenStream,
    children_quotes: TokenStream,
    has_parent: bool,
    index: usize,
) -> TokenStream {
    let parent = if has_parent {
        quote! { parent_id }
    } else {
        quote! { None }
    };

    quote! {
        let #widget_name = #children_quotes;
        let (should_rerender, child_id) =
            context
                .widget_manager
                .create_widget(#index, #widget_name, #parent);
        if should_rerender {
            let mut child_widget = context.widget_manager.take(child_id);
            child_widget.render(context);
            context.widget_manager.repossess(child_widget);
        }
    }
}
