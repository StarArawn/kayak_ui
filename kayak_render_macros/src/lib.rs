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
use syn::{parse_macro_input, parse_quote, token::Comma};

use crate::widget::Widget;

// use crate::prebuilt::element::Element;

#[proc_macro]
#[proc_macro_error]
pub fn render(input: TokenStream) -> TokenStream {
    let mut input = input.into_iter();
    let context = proc_macro2::TokenStream::from(TokenStream::from(input.next().unwrap()));
    let comma_input = TokenStream::from(input.next().unwrap());
    let _ = parse_macro_input!(comma_input as Comma);
    let rsx_data = proc_macro2::TokenStream::from_iter(
        input.map(|token_tree| proc_macro2::TokenStream::from(TokenStream::from(token_tree))),
    );
    let el = proc_macro2::TokenStream::from(rsx(TokenStream::from(rsx_data)));
    #[cfg(feature = "internal")]
    let kayak_core = quote! { kayak_core };
    #[cfg(not(feature = "internal"))]
    let kayak_core = quote! { kayak_ui::core };

    let result = quote! { #kayak_core::Render::render_into(&#el, #context, None) };
    TokenStream::from(result)
}

/// Generate a renderable widget tree, before rendering it
#[proc_macro]
#[proc_macro_error]
pub fn rsx(input: TokenStream) -> TokenStream {
    let el = parse_macro_input!(input as Widget);
    let result = quote! { #el };
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
