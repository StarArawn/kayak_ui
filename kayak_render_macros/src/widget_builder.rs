use proc_macro2::TokenStream;
use quote::quote;

/// Creates a token stream for building a widget
///
/// # Arguments
///
/// * `widget_name`: The name of the widget to build
/// * `widget_constructor`: The widget constructor token stream
/// * `index`: The sibling index of this widget (starting from 0)
///
/// returns: TokenStream
///
/// # Examples
///
/// ```
/// build_widget(quote! { my_widget }, quote!{ <MyWidget as Widget>::constructor(props) }, 0);
/// // Outputs token stream:
/// //   let my_widget = <MyWidget as Widget>::constructor(props);
/// //   context.add_widget(my_widget, 0);
/// ```
pub fn build_widget_stream(
    widget_name: TokenStream,
    widget_constructor: TokenStream,
    index: usize,
) -> TokenStream {
    quote! {
        let #widget_name = #widget_constructor;
        context.add_widget(#widget_name, #index);
    }
}
