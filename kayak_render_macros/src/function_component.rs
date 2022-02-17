use crate::get_core_crate;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use proc_macro_error::emit_error;
use quote::{format_ident, quote};
use syn::spanned::Spanned;
use syn::{parse_quote, FnArg, Pat, Signature, Type};

const DEFAULT_PROP_IDENT: &str = "__props";

pub struct WidgetArguments {
    pub focusable: bool,
}

impl Default for WidgetArguments {
    fn default() -> Self {
        Self { focusable: false }
    }
}

pub fn create_function_widget(f: syn::ItemFn, _widget_arguments: WidgetArguments) -> TokenStream {
    let struct_name = f.sig.ident.clone();
    let (impl_generics, ty_generics, where_clause) = f.sig.generics.split_for_impl();

    let (props, prop_type) = if let Some(parsed) = get_props(&f.sig) {
        parsed
    } else {
        return TokenStream::new();
    };

    let attrs = f.attrs;
    let block = f.block;
    let vis = f.vis;

    let kayak_core = get_core_crate();

    TokenStream::from(quote! {
        #(#attrs)*
        #[derive(Default, Debug, PartialEq, Clone)]
        #vis struct #struct_name #impl_generics {
            pub id: #kayak_core::Index,
            pub #props: #prop_type
        }

        impl #impl_generics #kayak_core::Widget for #struct_name #ty_generics #where_clause {

            type Props = #prop_type;

            fn constructor(props: Self::Props) -> Self where Self: Sized {
                Self {
                    id: #kayak_core::Index::default(),
                    #props: props,
                }
            }

            fn get_id(&self) -> #kayak_core::Index {
                self.id
            }

            fn set_id(&mut self, id: #kayak_core::Index) {
                self.id = id;
            }

            fn get_props(&self) -> &Self::Props {
                &self.#props
            }

            fn get_props_mut(&mut self) -> &mut Self::Props {
                &mut self.#props
            }

            fn render(&mut self, context: &mut #kayak_core::KayakContextRef) {
                use #kayak_core::WidgetProps;

                let parent_id = Some(self.get_id());
                let children = self.#props.get_children();
                let mut #props = self.#props.clone();

                #block

                self.#props = #props;
                context.commit();
            }
        }
    })
}

fn get_props(signature: &Signature) -> Option<(Ident, Type)> {
    if signature.inputs.len() > 1 {
        let span = if signature.inputs.len() > 0 {
            signature.inputs.span()
        } else {
            signature.span()
        };
        emit_error!(
            span,
            "Functional widgets expect at most one argument (their props), but was given {}",
            signature.inputs.len()
        );
        return None;
    }

    if signature.inputs.len() == 0 {
        let ident = format_ident!("{}", DEFAULT_PROP_IDENT);
        let ty: Type = parse_quote! {()};
        return Some((ident, ty));
    }

    match signature.inputs.first().unwrap() {
        FnArg::Typed(typed) => {
            let ident = match *typed.pat.clone() {
                Pat::Ident(ident) => ident.ident,
                err => {
                    emit_error!(err.span(), "Expected identifier, but got {:?}", err);
                    return None;
                }
            };

            let ty = *typed.ty.clone();
            match &ty {
                Type::Path(..) => {}
                err => {
                    emit_error!(err.span(), "Invalid widget prop type: {:?}", err);
                    return None;
                }
            };

            Some((ident, ty))
        }
        FnArg::Receiver(receiver) => {
            emit_error!(receiver.span(), "Functional widget cannot use 'self'");
            return None;
        }
    }
}
