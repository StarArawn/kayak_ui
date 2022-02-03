use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream, Result};
use syn::spanned::Spanned;

use crate::widget::Widget;

#[derive(Clone)]
pub enum Child {
    Widget(Widget),
    RawBlock(syn::Block),
}

impl ToTokens for Child {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Widget(widget) => widget.to_tokens(tokens),
            Self::RawBlock(block) => {
                let ts = if block.stmts.len() == 1 {
                    let first = &block.stmts[0];
                    quote!(#first)
                } else {
                    quote!(#block)
                };
                ts.to_tokens(tokens);
            }
        }
    }
}

impl Parse for Child {
    fn parse(input: ParseStream) -> Result<Self> {
        match Widget::custom_parse(input, true) {
            Ok(widget) => Ok(Self::Widget(widget)),
            Err(_) => {
                let block = input.parse::<syn::Block>()?;
                Ok(Self::RawBlock(block))
            }
        }
    }
}

pub fn walk_block_to_variable(block: &syn::Block) -> Option<proc_macro2::TokenStream> {
    if let Some(statement) = block.stmts.first() {
        return walk_statement(statement);
    }

    return None;
}

pub fn walk_statement(statement: &syn::Stmt) -> Option<proc_macro2::TokenStream> {
    match statement {
        syn::Stmt::Expr(expr) => match expr {
            syn::Expr::Call(call) => Some(call.args.to_token_stream()),
            syn::Expr::Path(path) => Some(path.to_token_stream()),
            _ => None,
        },
        _ => None,
    }
}
