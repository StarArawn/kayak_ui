use crate::get_core_crate;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::{Iter, Punctuated};
use syn::{bracketed, Token};

pub(crate) struct UseEffect {
    pub closure: syn::ExprClosure,
    pub dependencies: Punctuated<Ident, Token![,]>,
}

impl Parse for UseEffect {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let raw_deps;
        let closure = input.parse()?;
        let _: Token![,] = input.parse()?;
        let _ = bracketed!(raw_deps in input);
        let dependencies = raw_deps.parse_terminated(Ident::parse)?;

        Ok(Self {
            closure,
            dependencies,
        })
    }
}

impl UseEffect {
    fn get_deps(&self) -> Iter<Ident> {
        self.dependencies.iter()
    }

    fn get_clone_dep_idents(&self) -> impl Iterator<Item = Ident> + '_ {
        self.get_deps()
            .map(|dep| format_ident!("{}_dependency_clone", dep))
    }

    fn create_clone_deps(&self) -> proc_macro2::TokenStream {
        let deps = self.get_deps();
        let cloned_deps = self.get_clone_dep_idents();
        quote! {
            #(let #cloned_deps = #deps.clone());*
        }
    }

    fn create_dep_array(&self) -> proc_macro2::TokenStream {
        let cloned_deps = self.get_clone_dep_idents();
        quote! {
            &[#(&#cloned_deps),*]
        }
    }

    /// Build the output token stream, creating the actual use_effect code
    pub fn build(self) -> TokenStream {
        let kayak_core = get_core_crate();

        let dep_array = self.create_dep_array();
        let cloned_deps = self.create_clone_deps();
        let closure = self.closure;
        let result = quote! {{
            use #kayak_core::{Bound, MutableBound};
            #cloned_deps;
            context.create_effect(
                #closure,
                #dep_array
            );
        }};
        TokenStream::from(result)
    }
}
