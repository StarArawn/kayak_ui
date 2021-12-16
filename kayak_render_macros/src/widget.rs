use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Result};

use crate::arc_function::build_arc_function;
use crate::children::Children;
use crate::tags::ClosingTag;
use crate::{tags::OpenTag, widget_attributes::WidgetAttributes};

#[derive(Debug, Clone)]
pub struct Widget {
    pub attributes: WidgetAttributes,
    pub children: Children,
    declaration: TokenStream,
}

#[derive(Debug, Clone)]
pub struct ConstructedWidget {
    pub widget: Widget,
}

impl Parse for ConstructedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            widget: Widget::custom_parse(input, true, true).unwrap(),
        })
    }
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::custom_parse(input, false, true)
    }
}

impl Widget {
    pub fn is_custom_element(name: &syn::Path) -> bool {
        match name.get_ident() {
            None => true,
            Some(ident) => {
                let name = ident.to_string();
                let first_letter = name.get(0..1).unwrap();
                first_letter.to_uppercase() == first_letter
            }
        }
    }

    pub fn custom_parse(input: ParseStream, as_prop: bool, has_parent: bool) -> Result<Widget> {
        let open_tag = input.parse::<OpenTag>()?;

        let children = if open_tag.self_closing {
            Children::new(vec![])
        } else {
            let children = input.parse::<Children>()?;
            let closing_tag = input.parse::<ClosingTag>()?;
            closing_tag.validate(&open_tag);
            children
        };

        let name = open_tag.name;
        let declaration = if Self::is_custom_element(&name) {
            let attrs = &open_tag.attributes.for_custom_element(&children);
            let attrs = attrs.to_token_stream();
            if !as_prop {
                let attrs = quote! { #name #attrs };
                let widget_block =
                    build_arc_function(quote! { built_widget }, attrs, has_parent, 0, true);
                quote! {
                    #widget_block
                }
            } else {
                quote! {
                    #name #attrs
                }
            }
        } else {
            panic!("Couldn't find widget!");
        };

        Ok(Widget {
            attributes: open_tag.attributes,
            children,
            declaration,
        })
    }
}

impl ToTokens for Widget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.declaration.to_tokens(tokens);
    }
}
