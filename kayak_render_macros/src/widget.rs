use proc_macro2::TokenStream;
use proc_macro_error::{emit_error, emit_warning};
use quote::{format_ident, quote};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream, Result};
use syn::Path;
use syn::spanned::Spanned;

use crate::widget_builder::build_widget_stream;
use crate::children::Children;
use crate::tags::ClosingTag;
use crate::{get_core_crate, tags::OpenTag, widget_attributes::WidgetAttributes};
use crate::widget_attributes::CustomWidgetAttributes;

#[derive(Clone)]
pub struct Widget {
    pub attributes: WidgetAttributes,
    pub children: Children,
    declaration: TokenStream,
}

#[derive(Clone)]
pub struct ConstructedWidget {
    pub widget: Widget,
}

impl Parse for ConstructedWidget {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            widget: Widget::custom_parse(input, true).unwrap(),
        })
    }
}

impl Parse for Widget {
    fn parse(input: ParseStream) -> Result<Self> {
        Self::custom_parse(input, false)
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

    pub fn custom_parse(input: ParseStream, as_prop: bool) -> Result<Widget> {
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
            let (props, constructor) = Self::construct(&name, attrs);
            if !as_prop {
                let widget_block = build_widget_stream(quote! { built_widget }, constructor, 0);
                quote! {{
                    #props
                    #widget_block
                }}
            } else {
                quote! {{
                    #props
                    let widget = #constructor;
                    widget
                }}
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


    /// Constructs a widget and its props
    ///
    /// The returned tuple contains:
    /// 1. The props constructor and assignment
    /// 2. The widget constructor
    ///
    /// # Arguments
    ///
    /// * `name`: The full-path name of the widget
    /// * `attrs`: The attributes (props) to apply to this widget
    ///
    /// returns: (TokenStream, TokenStream)
    fn construct(name: &Path, attrs: &CustomWidgetAttributes) -> (TokenStream, TokenStream) {
        let kayak_core = get_core_crate();

        let prop_ident = format_ident!("props");
        let attrs = attrs.assign_attributes(&prop_ident);

        let props = quote! {
            let mut #prop_ident = <#name as kayak_core::Widget>::Props::default();
            #attrs
        };

        let constructor = quote! {
            <#name as kayak_core::Widget>::constructor(#prop_ident)
        };

        (props, constructor)
    }
}

impl ToTokens for Widget {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.declaration.to_tokens(tokens);
    }
}
