use quote::quote;
use std::hash::{Hash, Hasher};

use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream, Result},
    spanned::Spanned,
};

pub type AttributeKey = syn::punctuated::Punctuated<syn::Ident, syn::Token![-]>;

#[derive(Clone)]
pub enum Attribute {
    Punned(AttributeKey),
    WithValue(AttributeKey, syn::Block),
}

impl Attribute {
    pub fn ident(&self) -> &AttributeKey {
        match self {
            Self::Punned(ident) | Self::WithValue(ident, _) => ident,
        }
    }

    pub fn value_tokens(&self) -> proc_macro2::TokenStream {
        match self {
            Self::WithValue(_, value) => {
                if value.stmts.len() == 1 {
                    let first = &value.stmts[0];
                    quote!(#first)
                } else {
                    quote!(#value)
                }
            }
            Self::Punned(ident) => quote!(#ident),
        }
    }

    pub fn idents(&self) -> Vec<&syn::Ident> {
        self.ident().iter().collect::<Vec<_>>()
    }

    pub(crate) fn validate(self) -> Result<Self> {
        if self.idents().len() < 2 {
            Ok(self)
        } else {
            let alternative_name = self
                .idents()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("_");

            let error_message = format!(
                "Can't use dash-delimited values on custom widgets. Did you mean `{}`?",
                alternative_name
            );

            Err(syn::Error::new(self.ident().span(), error_message))
        }
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Self) -> bool {
        let self_idents: Vec<_> = self.ident().iter().collect();
        let other_idents: Vec<_> = other.ident().iter().collect();
        self_idents == other_idents
    }
}

impl Eq for Attribute {}

impl Hash for Attribute {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ident = self.idents();
        Hash::hash(&ident, state)
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = AttributeKey::parse_separated_nonempty_with(input, syn::Ident::parse_any)?;
        let not_punned = input.peek(syn::Token![=]);

        if !not_punned {
            return Ok(Self::Punned(name));
        }

        input.parse::<syn::Token![=]>()?;
        let value = input.parse::<syn::Block>()?;

        Ok(Self::WithValue(name, value))
    }
}
