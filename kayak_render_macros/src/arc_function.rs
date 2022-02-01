use proc_macro2::TokenStream;
use quote::quote;

pub fn build_arc_function(
    widget_name: TokenStream,
    children_quotes: TokenStream,
    index: usize,
) -> TokenStream {
    quote! {
        let children = children.clone();
        let #widget_name = #children_quotes;
        context.add_widget(#widget_name, #index);
    }
}
