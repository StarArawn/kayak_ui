use crate::{child::Child, widget_builder::build_widget_stream};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};

#[derive(Clone, Debug)]
pub struct Children {
    pub nodes: Vec<Child>,
}

impl Children {
    pub fn new(nodes: Vec<Child>) -> Self {
        Children { nodes }
    }

    pub fn is_block(&self) -> bool {
        self.nodes.iter().any(|node| match node {
            Child::RawBlock(_) => true,
            _ => false,
        })
    }

    // pub fn get_clonable_attributes(&self, index: usize) -> Vec<proc_macro2::TokenStream> {
    //     let mut tokens = Vec::new();

    //     let regular_tokens: Vec<_> = match &self.nodes[index] {
    //         Child::Widget(widget) => widget
    //             .attributes
    //             .attributes
    //             .iter()
    //             .filter_map(|attr| match attr {
    //                 Attribute::WithValue(_, block) => walk_block_to_variable(block),
    //                 _ => None,
    //             })
    //             .collect(),
    //         _ => vec![],
    //     };
    //     tokens.extend(regular_tokens);

    //     let children_tokens: Vec<proc_macro2::TokenStream> = match &self.nodes[index] {
    //         Child::Widget(widget) => (0..widget.children.nodes.len())
    //             .into_iter()
    //             .map(|child_id| widget.children.get_clonable_attributes(child_id))
    //             .flatten()
    //             .collect(),
    //         _ => vec![],
    //     };

    //     tokens.extend(children_tokens);

    //     tokens.dedup_by(|a, b| a.to_string().eq(&b.to_string()));

    //     tokens
    // }

    pub fn as_option_of_tuples_tokens(&self, only_children: bool) -> proc_macro2::TokenStream {
        let children_quotes: Vec<_> = self
            .nodes
            .iter()
            .map(|child| {
                let (entity_id, index) = match child {
                    Child::Widget((widget, index)) => (widget.entity_id.clone(), *index),
                    _ => (quote! {}, 0usize),
                };
                (
                    entity_id,
                    quote! { #child },
                    match child {
                        Child::Widget(_) => true,
                        _ => false,
                    },
                    index,
                )
            })
            .collect();

        match children_quotes.len() {
            0 => quote! { None },
            1 => {
                let child = if children_quotes[0].1.to_string() == "{ }" {
                    quote! { None }
                } else {
                    // let children_attributes: Vec<_> = self.get_clonable_attributes(0);

                    // I think this is correct.. It needs more testing though..
                    // let clonable_children = children_attributes
                    //     .iter()
                    //     .filter(|ts| syn::parse_str::<syn::Path>(&ts.to_string()).is_ok())
                    //     .collect::<Vec<_>>();

                    // let cloned_attrs = quote! {
                    //     #(let #clonable_children = #clonable_children.clone();)*;
                    // };
                    let id = children_quotes[0].0.clone();
                    let name: proc_macro2::TokenStream = format!("child{}", 0).parse().unwrap();
                    let id = if id.to_string().contains("widget_entity") {
                        name
                    } else {
                        id
                    };
                    let child_dec = children_quotes[0].1.clone();

                    if child_dec.to_string() == "children" {
                        quote! {
                            #child_dec
                        }
                    } else {
                        let children_builder = build_widget_stream(
                            quote! { #id },
                            quote! { #child_dec },
                            0,
                            !only_children || children_quotes[0].2,
                        );

                        quote! {
                            // Some(#kayak_core::Children::new(move |parent_id: Option<bevy::prelude::Entity>, context: &mut #kayak_core::WidgetContext, commands: &mut bevy::prelude::Commands| {
                                // #cloned_attrs
                                #children_builder
                            // }))
                        }
                    }
                };
                quote! {
                    #child
                }
            }
            _ => {
                // First get shared and non-shared attributes..
                // let mut child_attributes_list = Vec::new();
                // for i in 0..children_quotes.len() {
                //     let ts_vec = self.get_clonable_attributes(i);

                //     // I think this is correct.. It needs more testing though..
                //     let clonable_children = ts_vec
                //         .into_iter()
                //         .filter(|ts| syn::parse_str::<syn::Path>(&ts.to_string()).is_ok())
                //         .collect::<Vec<_>>();

                //     child_attributes_list.push(clonable_children);
                // }

                // let mut all_attributes = HashSet::new();
                // for child_attributes in child_attributes_list.iter() {
                //     for child_attribute in child_attributes {
                //         all_attributes.insert(child_attribute.to_string());
                //     }
                // }

                // all_attributes.insert("children".to_string());

                // let base_matching: Vec<proc_macro2::TokenStream> = all_attributes
                //     .iter()
                //     .map(|a| format!("base_{}", a).to_string().parse().unwrap())
                //     .collect();

                // let all_attributes: Vec<proc_macro2::TokenStream> =
                //     all_attributes.iter().map(|a| a.parse().unwrap()).collect();

                // let base_clone = quote! {
                //     #(let #base_matching = #all_attributes.clone();)*
                // };

                // let base_clones_inner = quote! {
                //     #(let #all_attributes = #base_matching.clone();)*
                // };

                let mut output = Vec::new();
                // output.push(quote! { #base_clone });
                for i in 0..children_quotes.len() {
                    // output.push(quote! { #base_clones_inner });
                    let name: proc_macro2::TokenStream = format!("child{}", i).parse().unwrap();
                    let entity_id = if children_quotes[i].0.to_string().contains("widget_entity") {
                        name
                    } else {
                        children_quotes[i].0.clone()
                    };
                    let child = build_widget_stream(
                        quote! { #entity_id },
                        children_quotes[i].1.clone(),
                        i,
                        children_quotes[i].2,
                    );

                    output.push(quote! { #child });
                }

                quote! {
                    // Some(#kayak_core::Children::new(move |parent_id: Option<bevy::prelude::Entity>, context: &mut #kayak_core::WidgetContext, commands: &mut bevy::prelude::Commands| {
                        #(#output)*
                        // context.commit();
                    // }))
                }
            }
        }
    }
}

impl Parse for Children {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut nodes = vec![];
        let mut index: usize = 0;
        while !input.peek(syn::Token![<]) || !input.peek2(syn::Token![/]) {
            let child = Child::custom_parse(input, index)?;
            nodes.push(child);
            index += 1;
        }

        Ok(Self::new(nodes))
    }
}

impl ToTokens for Children {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.as_option_of_tuples_tokens(false).to_tokens(tokens);
    }
}
