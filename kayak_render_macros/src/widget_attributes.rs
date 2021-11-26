use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use std::collections::HashSet;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
};

use crate::{attribute::Attribute, children::Children};

#[derive(Debug, Clone)]
pub struct WidgetAttributes {
    pub attributes: HashSet<Attribute>,
}

impl WidgetAttributes {
    pub fn new(attributes: HashSet<Attribute>) -> Self {
        Self { attributes }
    }

    pub fn for_custom_element<'c>(&self, children: &'c Children) -> CustomWidgetAttributes<'_, 'c> {
        CustomWidgetAttributes {
            attributes: &self.attributes,
            children,
        }
    }

    pub fn custom_parse(input: ParseStream) -> Result<Self> {
        let mut parsed_self = input.parse::<Self>()?;
        let new_attributes: HashSet<Attribute> = parsed_self
            .attributes
            .drain()
            .filter_map(|attribute| match attribute.validate() {
                Ok(x) => Some(x),
                Err(err) => {
                    emit_error!(err.span(), "Invalid attribute: {}", err);
                    None
                }
            })
            .collect();

        Ok(WidgetAttributes::new(new_attributes))
    }
}

impl Parse for WidgetAttributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attributes: HashSet<Attribute> = HashSet::new();
        while input.peek(syn::Ident::peek_any) {
            let attribute = input.parse::<Attribute>()?;
            let ident = attribute.ident();
            if attributes.contains(&attribute) {
                emit_error!(
                    ident.span(),
                    "There is a previous definition of the {} attribute",
                    quote!(#ident)
                );
            }
            attributes.insert(attribute);
        }
        Ok(WidgetAttributes::new(attributes))
    }
}

pub struct CustomWidgetAttributes<'a, 'c> {
    attributes: &'a HashSet<Attribute>,
    children: &'c Children,
}

impl<'a, 'c> ToTokens for CustomWidgetAttributes<'a, 'c> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let mut attrs: Vec<_> = self
            .attributes
            .iter()
            .map(|attribute| {
                let ident = attribute.ident();
                let value = attribute.value_tokens();

                quote! {
                    #ident: #value
                }
            })
            .collect();

        {
            let children_tuple = self.children.as_option_of_tuples_tokens();
            attrs.push(quote! {
                children: #children_tuple
            });
        }

        let missing = vec![
            ("styles", quote! { styles: None }),
            ("on_event", quote! { on_event: None }),
        ];

        for missed in missing {
            if !self.attributes.iter().any(|attribute| {
                attribute
                    .ident()
                    .to_token_stream()
                    .to_string()
                    .contains(missed.0)
            }) {
                attrs.push(missed.1);
            }
        }

        let quoted = if attrs.len() == 0 {
            quote!({ id: kayak_core::Index::default(), styles: None, children: None, on_event: None, })
        } else {
            if !self
                .attributes
                .iter()
                .any(|attribute| attribute.ident().to_token_stream().to_string() == "styles")
            {
                quote!({ #(#attrs),*, id: kayak_core::Index::default() })
            } else {
                quote!({ #(#attrs),*, id: kayak_core::Index::default() })
            }
        };

        quoted.to_tokens(tokens);
    }
}
