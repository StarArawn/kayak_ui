use crate::get_core_crate;
use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::quote;
use syn::spanned::Spanned;
use syn::{FnArg, Pat, Type};

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

    if f.sig.inputs.len() != 1 {
        let span = if f.sig.inputs.len() > 0 {
            f.sig.inputs.span()
        } else {
            f.sig.span()
        };
        emit_error!(
            span,
            "Functional widgets expect exactly one argument (their props), but was given {}",
            f.sig.inputs.len()
        );
    }

    let (props, prop_type) = match f.sig.inputs.first().unwrap() {
        FnArg::Typed(typed) => {
            let ident = match *typed.pat.clone() {
                Pat::Ident(ident) => ident.ident,
                err => {
                    emit_error!(err.span(), "Expected identifier, but got {:?}", err);
                    return TokenStream::new();
                }
            };

            let ty = match *typed.ty.clone() {
                Type::Path(type_path) => type_path.path,
                err => {
                    emit_error!(err.span(), "Invalid widget prop type: {:?}", err);
                    return TokenStream::new();
                }
            };

            (ident, ty)
        }
        FnArg::Receiver(receiver) => {
            emit_error!(receiver.span(), "Functional widget cannot use 'self'");
            return TokenStream::new();
        }
    };

    let block = f.block;
    let vis = f.vis;

    let kayak_core = get_core_crate();

    TokenStream::from(quote! {
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
