use quote::{quote, ToTokens};
use syn::parse::{ParseStream, Result};

use crate::widget::Widget;

#[derive(Clone, Debug)]
pub enum Child {
    Widget((Widget, usize)),
    RawBlock((syn::Block, usize)),
}

impl Child {
    pub fn custom_parse(input: ParseStream, index: usize) -> Result<Self> {
        match Widget::custom_parse(input, true, false, index) {
            Ok(widget) => Ok(Self::Widget((widget, index))),
            Err(_) => {
                let block = input.parse::<syn::Block>()?;
                Ok(Self::RawBlock((block, index)))
            }
        }
    }
}

impl ToTokens for Child {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Widget((widget, _)) => widget.to_tokens(tokens),
            Self::RawBlock((block, _)) => {
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

// pub fn walk_block_to_variable(block: &syn::Block) -> Option<proc_macro2::TokenStream> {
//     if let Some(statement) = block.stmts.first() {
//         return walk_statement(statement);
//     }

//     return None;
// }

// pub fn walk_statement(statement: &syn::Stmt) -> Option<proc_macro2::TokenStream> {
//     match statement {
//         syn::Stmt::Expr(expr) => match expr {
//             syn::Expr::Call(call) => Some(call.args.to_token_stream()),
//             syn::Expr::Path(path) => Some(path.to_token_stream()),
//             _ => None,
//         },
//         _ => None,
//     }
// }
