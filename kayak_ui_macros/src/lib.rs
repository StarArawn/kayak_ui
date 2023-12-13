use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::parse_macro_input;
use widget::{ConstructedWidget, Widget};

pub(crate) mod attribute;
pub(crate) mod child;
pub(crate) mod children;
pub(crate) mod tags;
pub(crate) mod widget;
pub(crate) mod widget_attributes;
pub(crate) mod widget_builder;
// mod block;

/// A proc macro that turns RSX syntax into structure constructors and calls the
/// context to create the widgets.
/// Note: This macro expects the following:
///
/// `parent_id`: Variable in scope. Typed as: `Option<Entity>`. This should always
/// be None for the root node or `Some(parent_entity)` for any other widget
///
/// `widget_context`: Variable in scope. Can either be a `WidgetRootContext` or a `WidgetContext`.
/// This is used to automatically add widgets to the tree when parsing the XML syntax into code.
///
/// `commands`: Variable in scope. Must be a variable to a mutable bevy `Commands`. This is used to
/// spawn entities and insert bundles onto entities automatically after parsing the XML syntax to code.
#[proc_macro]
#[proc_macro_error]
pub fn rsx(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);
    let result = quote! { #widget };
    TokenStream::from(result)
}

/// A proc macro that turns RSX syntax into structure constructors and calls the
/// context to create the widgets.
#[proc_macro]
#[proc_macro_error]
pub fn constructor(input: TokenStream) -> TokenStream {
    let el = parse_macro_input!(input as ConstructedWidget);
    let widget = el.widget;
    let result = quote! {
        let widget_entity = #widget;
        children.add(widget_entity);
    };
    TokenStream::from(result)
}

// / Helper method for getting the core crate
// /
// / Depending on the usage of the macro, this will become `crate`, `kayak_core`,
// / or `kayak_ui::core`.
// /
// / # Examples
// /
// / ```
// / fn my_macro() -> proc_macro2::TokenStream {
// /   let kayak_core = get_core_crate();
// /   quote! {
// /     let foo = #kayak_core::Foo;
// /   }
// / }
// / ```
// fn get_core_crate() -> proc_macro2::TokenStream {
//     let found_crate = proc_macro_crate::crate_name("kayak_ui");
//     if let Ok(found_crate) = found_crate {
//         let result = match found_crate {
//             proc_macro_crate::FoundCrate::Itself => {
//                 // let crate_name = find_crate::find_crate(|s| s == "kayak_ui" || s == "crate");
//                 // dbg!(crate_name);
//                 // if path.contains("example") {
//                 //quote! { kayak_ui }
//                 // } else {
//                 quote! { crate }
//                 // }
//             },
//             proc_macro_crate::FoundCrate::Name(name) => {
//                 let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
//                 quote!(#ident)
//             }
//         };
//         result
//     } else {
//         quote!(kayak_ui)
//     }
// }
