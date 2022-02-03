use proc_macro::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::{quote, ToTokens};
use syn::{FnArg, Pat, Type};
use syn::spanned::Spanned;
use crate::get_core_crate;

pub struct WidgetArguments {
    pub focusable: bool,
}

impl Default for WidgetArguments {
    fn default() -> Self {
        Self { focusable: false }
    }
}

pub fn create_function_widget(f: syn::ItemFn, widget_arguments: WidgetArguments) -> TokenStream {
    let struct_name = f.sig.ident.clone();
    let (impl_generics, ty_generics, where_clause) = f.sig.generics.split_for_impl();

    if f.sig.inputs.len() != 1 {
        let span = if f.sig.inputs.len() > 0 {
            f.sig.inputs.span()
        } else {
            f.sig.span()
        };
        emit_error!(span, "Functional widgets expect exactly one argument (their props), but was given {}", f.sig.inputs.len());
    }

    let (props, prop_type) = match f.sig.inputs.first().unwrap() {
        FnArg::Typed(typed) => {
            let ident = match *typed.pat.clone() {
                Pat::Ident(ident) => {
                    ident.ident
                }
                err => {
                    emit_error!(err.span(), "Expected identifier, but got {:?}", err);
                    return TokenStream::new()
                }
            };

            let ty = match *typed.ty.clone() {
                Type::Path(type_path) => {
                    type_path.path
                },
                err => {
                    emit_error!(err.span(), "Invalid widget prop type: {:?}", err);
                    return TokenStream::new()
                }
            };

            (ident, ty)
        },
        FnArg::Receiver(receiver) => {
            emit_error!(receiver.span(), "Functional widget cannot use 'self'");
            return TokenStream::new()
        }
    };

    let block = f.block;
    let vis = f.vis;

    let kayak_core = get_core_crate();


    // TODO: See if this is still needed. If not, remove it
    // let mut input_names: Vec<_> = inputs
    //     .iter()
    //     .filter_map(|argument| match argument {
    //         syn::FnArg::Typed(typed) => {
    //             let typed_info = typed.ty.to_token_stream().to_string();
    //             let attr_info = typed.pat.to_token_stream().to_string();
    //             if (typed_info.contains("KayakContext") && !typed_info.contains("Fn"))
    //                 || (attr_info.contains("styles") && typed_info.contains("Style"))
    //             {
    //                 None
    //             } else {
    //                 Some(typed)
    //             }
    //         }
    //         syn::FnArg::Receiver(rec) => {
    //             emit_error!(rec.span(), "Don't use `self` on widgets");
    //             None
    //         }
    //     })
    //     .map(|value| {
    //         let pat = &value.pat;
    //         quote!(#pat)
    //     })
    //     .collect();
    //
    // let mut input_block_names: Vec<_> = inputs
    //     .iter()
    //     .filter(|input| {
    //         let input = (quote! { #input }).to_string();
    //         !(input.contains("parent_styles")
    //             || (input.contains("KayakContext") && !input.contains("Fn")))
    //     })
    //     .map(|item| quote! { #item })
    //     .collect();
    // input_block_names.iter_mut().for_each(|input| {
    //     let input_string = (quote! { #input }).to_string();
    //     if input_string.contains("children : Children") {
    //         *input = quote! {
    //             #[derivative(Debug = "ignore", PartialEq = "ignore")]
    //             pub children: Children
    //         };
    //     } else if input_string.contains("on_event : Option < OnEvent >") {
    //         *input = quote! {
    //             #[derivative(Debug = "ignore", PartialEq = "ignore")]
    //             pub on_event: Option<#kayak_core::OnEvent>
    //         };
    //     } else {
    //         *input = quote! {
    //             pub #input
    //         }
    //     }
    // });

    // let focusable_default = if widget_arguments.focusable {
    //     "Some(true)"
    // } else {
    //     "None"
    // };
    //
    // let missing_struct_inputs = vec![
    //     (
    //         vec![
    //             "styles : Option < Style >",
    //             "styles : Option< kayak_ui :: core :: styles :: Style >",
    //         ],
    //         quote! {
    //             #[derivative(Default(value="None"))]
    //             pub styles: Option<#kayak_core::styles::Style>
    //         },
    //     ),
    //     (
    //         vec!["children : Children"],
    //         quote! {
    //             #[derivative(Default(value="None"), Debug = "ignore", PartialEq = "ignore")]
    //             pub children: #kayak_core::Children
    //         },
    //     ),
    //     (
    //         vec![
    //             "on_event : Option < OnEvent >",
    //             "on_event : Option < kayak_ui :: core :: OnEvent >",
    //             "on_event : Option <\nkayak_ui :: core :: OnEvent >",
    //         ],
    //         quote! {
    //             #[derivative(Default(value="None"), Debug = "ignore", PartialEq = "ignore")]
    //             pub on_event: Option<#kayak_core::OnEvent>
    //         },
    //     ),
    //     (
    //         vec!["focusable : Option < bool >"],
    //         quote! {
    //             #[derivative(Default(value=#focusable_default))]
    //             pub focusable: Option<bool>
    //         },
    //     ),
    // ];

    // for (names, token) in missing_struct_inputs {
    //     if !input_block_names.iter().any(|block_name| {
    //         names
    //             .iter()
    //             .any(|name| block_name.to_string().contains(name))
    //     }) {
    //         input_block_names.push(token);
    //     } else {
    //     }
    // }

    // let inputs_block = quote!(
    //     #(#input_block_names),*
    // );
    //
    // if !input_names
    //     .iter()
    //     .any(|item_name| item_name.to_string().contains("children"))
    // {
    //     input_names.push(quote! {
    //         children
    //     });
    // }

    // let inputs_reading_ref = if inputs.len() == 0 {
    //     quote! {
    //         let #struct_name { children, styles, .. } = self;
    //     }
    // } else {
    //     quote!(
    //         let #struct_name { #(#input_names),*, styles, .. } = self;
    //         #(let #input_names = #input_names.clone();)*
    //     )
    // };

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
