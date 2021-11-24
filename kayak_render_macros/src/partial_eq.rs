extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

pub fn impl_dyn_partial_eq(ast: &syn::DeriveInput) -> TokenStream {
  let name = &ast.ident;
  let gen = quote! {
      impl DynPartialEq for #name {
          fn box_eq(&self, other: &dyn core::any::Any) -> bool {
            other.downcast_ref::<Self>().map_or(false, |a| self == a)
          }
      }
  };
  gen.into()
}
