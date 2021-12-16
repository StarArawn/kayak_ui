extern crate proc_macro;

mod function_component;
mod tags;

mod arc_function;
mod attribute;
mod child;
mod children;
mod partial_eq;
mod widget;
mod widget_attributes;

use partial_eq::impl_dyn_partial_eq;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{parse_macro_input, parse_quote};
use widget::ConstructedWidget;

use crate::widget::Widget;

#[proc_macro]
#[proc_macro_error]
pub fn render(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);

    #[cfg(feature = "internal")]
    let kayak_core = quote! { kayak_core };
    #[cfg(not(feature = "internal"))]
    let kayak_core = quote! { kayak_ui::core };

    let result = quote! {
        let parent_id: Option<Index> = None;
        let children: Option<#kayak_core::Children> = None;
        let tree = #kayak_core::WidgetTree::new();
        #widget
    };

    TokenStream::from(result)
}

/// Generate a renderable widget tree, before rendering it
#[proc_macro]
#[proc_macro_error]
pub fn rsx(input: TokenStream) -> TokenStream {
    let widget = parse_macro_input!(input as Widget);
    let result = quote! { #widget };
    TokenStream::from(result)
}

#[proc_macro]
#[proc_macro_error]
pub fn constructor(input: TokenStream) -> TokenStream {
    let el = parse_macro_input!(input as ConstructedWidget);
    let widget = el.widget;
    let result = quote! { #widget };
    TokenStream::from(result)
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn widget(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let f = parse_macro_input!(item as syn::ItemFn);
    function_component::create_function_widget(f)
}

#[proc_macro_derive(DynPartialEq)]
pub fn dyn_partial_eq_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_dyn_partial_eq(&ast)
}

#[proc_macro_attribute]
pub fn dyn_partial_eq(_: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as syn::ItemTrait);

    let name = &input.ident;

    let bound: syn::TypeParamBound = parse_quote! {
      DynPartialEq
    };

    input.supertraits.push(bound);

    (quote! {
      #input

      impl core::cmp::PartialEq for Box<dyn #name> {
        fn eq(&self, other: &Self) -> bool {
          self.box_eq(other.as_any())
        }
      }
    })
    .into()
}
