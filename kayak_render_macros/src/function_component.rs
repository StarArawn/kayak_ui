use proc_macro::TokenStream;
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

pub fn create_function_widget(f: syn::ItemFn) -> TokenStream {
    let struct_name = f.sig.ident;
    let (impl_generics, ty_generics, where_clause) = f.sig.generics.split_for_impl();
    let inputs = f.sig.inputs;
    let block = f.block;
    let vis = f.vis;

    #[cfg(feature = "internal")]
    let kayak_core = quote! { kayak_core };
    #[cfg(not(feature = "internal"))]
    let kayak_core = quote! { kayak_ui::core };

    let mut input_names: Vec<_> = inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Typed(typed) => {
                let typed_info = typed.ty.to_token_stream().to_string();
                let attr_info = typed.pat.to_token_stream().to_string();
                if (typed_info.contains("KayakContext") && !typed_info.contains("Fn"))
                    || (attr_info.contains("styles") && typed_info.contains("Style"))
                {
                    None
                } else {
                    Some(typed)
                }
            }
            syn::FnArg::Receiver(rec) => {
                emit_error!(rec.span(), "Don't use `self` on widgets");
                None
            }
        })
        .map(|value| {
            let pat = &value.pat;
            quote!(#pat)
        })
        .collect();

    let mut input_block_names: Vec<_> = inputs
        .iter()
        .filter(|input| {
            let input = (quote! { #input }).to_string();
            !(input.contains("parent_styles")
                || (input.contains("KayakContext") && !input.contains("Fn")))
        })
        .map(|item| quote! { #item })
        .collect();
    input_block_names.iter_mut().for_each(|input| {
        let input_string = (quote! { #input }).to_string();
        if input_string.contains("children : Children") {
            *input = quote! {
                 #[derivative(Debug = "ignore", PartialEq = "ignore")]
                 pub children: Children
            };
        } else {
            *input = quote! {
                pub #input
            }
        }
    });

    let missing_struct_inputs = vec![
        (
            vec![
                "styles : Option < Style >",
                "styles : Option< kayak_core :: styles :: Style >",
            ],
            quote! {
                pub styles: Option<#kayak_core::styles::Style>
            },
        ),
        (
            vec!["children : Children"],
            quote! {
                #[derivative(Debug = "ignore", PartialEq = "ignore")]
                pub children: #kayak_core::Children
            },
        ),
        (
            vec![
                "on_event: Option<OnEvent>",
                "on_event : Option<kayak_core::OnEvent>",
            ],
            quote! {
                #[derivative(Debug = "ignore", PartialEq = "ignore")]
                pub on_event: Option<#kayak_core::OnEvent>
            },
        ),
    ];

    for (names, token) in missing_struct_inputs {
        if !input_block_names.iter().any(|block_name| {
            names
                .iter()
                .any(|name| block_name.to_string().contains(name))
        }) {
            input_block_names.push(token);
        } else {
        }
    }

    let inputs_block = quote!(
        #(#input_block_names),*
    );

    if !input_names
        .iter()
        .any(|item_name| item_name.to_string().contains("children"))
    {
        input_names.push(quote! {
            children
        });
    }

    let inputs_reading_ref = if inputs.len() == 0 {
        quote! {
            let #struct_name { children, styles, .. } = self;
        }
    } else {
        quote!(
            let #struct_name { #(#input_names),*, styles, .. } = self;
        )
    };

    TokenStream::from(quote! {
        use #kayak_core::derivative::*;

        #[derive(Derivative)]
        #[derivative(Debug, PartialEq, Clone)]
        #vis struct #struct_name #impl_generics {
            pub id: #kayak_core::Index,
            #inputs_block
        }

        impl #impl_generics #kayak_core::Widget for #struct_name #ty_generics #where_clause {
            fn get_id(&self) -> #kayak_core::Index {
                self.id
            }

            fn set_id(&mut self, id: #kayak_core::Index) {
                self.id = id;
            }

            fn get_styles(&self) -> Option<#kayak_core::styles::Style> {
                self.styles.clone()
            }

            fn get_name(&self) -> String {
                String::from(stringify!(#struct_name))
            }

            fn on_event(&mut self, context: &mut #kayak_core::context::KayakContext, event: &mut #kayak_core::Event) {
                if let Some(on_event) = self.on_event.as_ref() {
                    if let Ok(mut on_event) = on_event.0.write() {
                        on_event(context, event);
                    }
                }
            }

            fn render(&mut self, context: &mut #kayak_core::context::KayakContext) {
                let parent_id = self.get_id();
                context.set_current_id(parent_id);
                let parent_id = Some(parent_id);
                #inputs_reading_ref
                let children = children.clone();
                let tree = #kayak_core::WidgetTree::new();

                #block

                // Consume the widget tree taking the inner value
                let tree = tree.take();

                // Evaluate changes to the tree.
                let changes = context
                    .widget_manager
                    .tree
                    .diff_children(&tree, self.get_id());

                context
                    .widget_manager
                    .tree
                    .merge(&tree, self.get_id(), changes);
            }
        }
    })
}
