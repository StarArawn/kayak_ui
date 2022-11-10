use proc_macro2::{Ident, TokenStream};
use proc_macro_error::emit_error;
use quote::quote;
use std::collections::HashSet;
use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
};

use crate::child::Child;
use crate::{attribute::Attribute, children::Children};

#[derive(Clone, Debug)]
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
    pub attributes: &'a HashSet<Attribute>,
    pub children: &'c Children,
}

impl<'a, 'c> CustomWidgetAttributes<'a, 'c> {
    /// Assign this widget's attributes to the given ident
    ///
    /// This takes the form: `IDENT.ATTR_NAME = ATTR_VALUE;`
    ///
    /// # Arguments
    ///
    /// * `ident`: The ident to assign to (i.e. "props")
    ///
    /// returns: TokenStream
    pub fn assign_attributes(&self, _ident: &Ident) -> TokenStream {
        let attrs = self
            .attributes
            .iter()
            .filter_map(|attribute| {
                let key = attribute.ident();
                let value = attribute.value_tokens();
                let key_name = quote! { #key }.to_string();
                if key_name == "id" {
                    None
                } else {
                    Some(quote! {
                        #key: #value,
                    })
                }
            })
            .collect::<Vec<_>>();

        let result = quote! {
            #( #attrs )*
        };

        result
    }

    /// Determines whether `children` should be added to this widget or not
    pub fn should_add_children(&self) -> bool {
        if self.children.nodes.is_empty() {
            // No children
            false
        } else if self.children.nodes.len() == 1 {
            let child = self.children.nodes.first().unwrap();
            match child {
                Child::RawBlock((block, _)) => {
                    // Is child NOT an empty block? (`<Foo>{}</Foo>`)
                    !block.stmts.is_empty()
                }
                // Child is a widget
                _ => true,
            }
        } else {
            // Multiple children
            true
        }
    }
}
