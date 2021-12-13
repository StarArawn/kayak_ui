use crate::widget_attributes::WidgetAttributes;
use proc_macro_error::abort;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

pub struct OpenTag {
    pub name: syn::Path,
    pub attributes: WidgetAttributes,
    pub self_closing: bool,
    pub is_custom_element: bool,
}

fn name_or_fragment(maybe_name: Result<syn::Path>) -> syn::Path {
    #[cfg(feature = "internal")]
    let kayak_core = "kayak_core";
    #[cfg(not(feature = "internal"))]
    let kayak_core = "kayak_ui::core";

    maybe_name.unwrap_or_else(|_| {
        syn::parse_str::<syn::Path>(&format!("::{}::Fragment", kayak_core)).unwrap()
    })
}

fn is_custom_element_name(path: &syn::Path) -> bool {
    match path.get_ident() {
        None => true,
        Some(ident) => {
            let name = ident.to_string();
            let first_letter = name.get(0..1).unwrap();
            first_letter.to_uppercase() == first_letter
        }
    }
}

impl Parse for OpenTag {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Token![<]>()?;
        let maybe_name = syn::Path::parse_mod_style(input);
        let name = name_or_fragment(maybe_name);
        let is_custom_element = is_custom_element_name(&name);
        let attributes = WidgetAttributes::custom_parse(input)?;
        let self_closing = input.parse::<syn::Token![/]>().is_ok();
        input.parse::<syn::Token![>]>()?;

        Ok(Self {
            name,
            attributes,
            self_closing,
            is_custom_element,
        })
    }
}

pub struct ClosingTag {
    name: syn::Path,
}

impl ClosingTag {
    pub fn validate(&self, open_tag: &OpenTag) {
        let open_tag_path = &open_tag.name;
        let open_tag_path_str = quote!(#open_tag_path).to_string();
        let self_path = &self.name;
        let self_path_str = quote!(#self_path).to_string();
        if self_path_str != open_tag_path_str {
            abort!(
                self.name.span(),
                "Expected closing tag for: <{}>",
                &open_tag_path_str
            );
        }
    }
}

impl Parse for ClosingTag {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Token![<]>()?;
        input.parse::<syn::Token![/]>()?;
        let maybe_name = input.parse::<syn::Path>();
        input.parse::<syn::Token![>]>()?;
        Ok(Self {
            name: name_or_fragment(maybe_name),
        })
    }
}
