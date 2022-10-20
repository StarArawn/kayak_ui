use syn::*;



/// A braced block containing Rust statements.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub struct Block {
    pub brace_token: token::Brace,
    /// Statements in a block
    pub stmts: Vec<Stmt>,
}

/// A statement, usually ending in a semicolon.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub enum Stmt {
    /// A local (let) binding.
    Local(Local),

    /// An item definition.
    Item(Item),

    /// Expr without trailing semicolon.
    Expr(Expr),

    /// Expression with trailing semicolon.
    Semi(Expr, Token![;]),
}

/// A local `let` binding: `let x: u64 = s.parse()?`.
///
/// *This type is available only if Syn is built with the `"full"` feature.*
pub struct Local {
    pub attrs: Vec<Attribute>,
    pub let_token: Token![let],
    pub pat: Pat,
    pub init: Option<(Token![=], Box<Expr>)>,
    pub semi_token: Token![;],
}

pub mod parsing {
    use super::*;
    use proc_macro2::{TokenStream, TokenTree, Delimiter};
    use syn::parse::discouraged::Speculative;
    use syn::parse::{ParseStream, Parse, ParseBuffer};
    use syn::punctuated::Punctuated;
    use syn::token::{Paren, Brace, Bracket};
    use syn::{Expr};

    impl Block {
        /// Parse the body of a block as zero or more statements, possibly
        /// including one trailing expression.
        ///
        /// *This function is available only if Syn is built with the `"parsing"`
        /// feature.*
        ///
        /// # Example
        ///
        /// ```
        /// use syn::{braced, token, Attribute, Block, Ident, Result, Stmt, Token};
        /// use syn::parse::{Parse, ParseStream};
        ///
        /// // Parse a function with no generics or parameter list.
        /// //
        /// //     fn playground {
        /// //         let mut x = 1;
        /// //         x += 1;
        /// //         println!("{}", x);
        /// //     }
        /// struct MiniFunction {
        ///     attrs: Vec<Attribute>,
        ///     fn_token: Token![fn],
        ///     name: Ident,
        ///     brace_token: token::Brace,
        ///     stmts: Vec<Stmt>,
        /// }
        ///
        /// impl Parse for MiniFunction {
        ///     fn parse(input: ParseStream) -> Result<Self> {
        ///         let outer_attrs = input.call(Attribute::parse_outer)?;
        ///         let fn_token: Token![fn] = input.parse()?;
        ///         let name: Ident = input.parse()?;
        ///
        ///         let content;
        ///         let brace_token = braced!(content in input);
        ///         let inner_attrs = content.call(Attribute::parse_inner)?;
        ///         let stmts = content.call(Block::parse_within)?;
        ///
        ///         Ok(MiniFunction {
        ///             attrs: {
        ///                 let mut attrs = outer_attrs;
        ///                 attrs.extend(inner_attrs);
        ///                 attrs
        ///             },
        ///             fn_token,
        ///             name,
        ///             brace_token,
        ///             stmts,
        ///         })
        ///     }
        /// }
        /// ```
        pub fn parse_within(input: ParseStream) -> Result<Vec<Stmt>> {
            let mut stmts = Vec::new();
            loop {
                while let Some(semi) = input.parse::<Option<Token![;]>>()? {
                    stmts.push(Stmt::Semi(Expr::Verbatim(TokenStream::new()), semi));
                }
                if input.is_empty() {
                    break;
                }
                let s = parse_stmt(input, true)?;
                let requires_semicolon = if let Stmt::Expr(s) = &s {
                    requires_terminator(s)
                } else {
                    false
                };
                stmts.push(s);
                if input.is_empty() {
                    break;
                } else if requires_semicolon {
                    return Err(input.error("unexpected token"));
                }
            }
            Ok(stmts)
        }
    }

    #[cfg_attr(doc_cfg, doc(cfg(feature = "parsing")))]
    impl Parse for Block {
        fn parse(input: ParseStream) -> Result<Self> {
            let content;
            Ok(Block {
                brace_token: braced!(content in input),
                stmts: content.call(Block::parse_within)?,
            })
        }
    }

    #[cfg_attr(doc_cfg, doc(cfg(feature = "parsing")))]
    impl Parse for Stmt {
        fn parse(input: ParseStream) -> Result<Self> {
            parse_stmt(input, false)
        }
    }

    fn parse_stmt(input: ParseStream, allow_nosemi: bool) -> Result<Stmt> {
        let begin = input.fork();
        let mut attrs = input.call(Attribute::parse_outer)?;

        // brace-style macros; paren and bracket macros get parsed as
        // expression statements.
        let ahead = input.fork();
        if let Ok(path) = ahead.call(Path::parse_mod_style) {
            if ahead.peek(Token![!])
                && (ahead.peek2(token::Brace)
                    && !(ahead.peek3(Token![.]) || ahead.peek3(Token![?]))
                    || ahead.peek2(Ident))
            {
                input.advance_to(&ahead);
                return stmt_mac(input, attrs, path);
            }
        }

        if input.peek(Token![let]) {
            stmt_local(input, attrs, begin)
        } else if input.peek(Token![pub])
            || input.peek(Token![crate]) && !input.peek2(Token![::])
            || input.peek(Token![extern])
            || input.peek(Token![use])
            || input.peek(Token![static])
                && (input.peek2(Token![mut])
                    || input.peek2(Ident)
                        && !(input.peek2(Token![async])
                            && (input.peek3(Token![move]) || input.peek3(Token![|]))))
            || input.peek(Token![const]) && !input.peek2(token::Brace)
            || input.peek(Token![unsafe]) && !input.peek2(token::Brace)
            || input.peek(Token![async])
                && (input.peek2(Token![unsafe])
                    || input.peek2(Token![extern])
                    || input.peek2(Token![fn]))
            || input.peek(Token![fn])
            || input.peek(Token![mod])
            || input.peek(Token![type])
            || input.peek(Token![struct])
            || input.peek(Token![enum])
            || input.peek(Token![union]) && input.peek2(Ident)
            || input.peek(Token![auto]) && input.peek2(Token![trait])
            || input.peek(Token![trait])
            || input.peek(Token![default])
                && (input.peek2(Token![unsafe]) || input.peek2(Token![impl]))
            || input.peek(Token![impl])
            || input.peek(Token![macro])
        {
            let mut item: Item = input.parse()?;
            attrs.extend(replace_attrs_item(&mut item, Vec::new()));
            replace_attrs_item(&mut item, attrs);
            Ok(Stmt::Item(item))
        } else {
            stmt_expr(input, allow_nosemi, attrs)
        }
    }

    fn stmt_mac(input: ParseStream, attrs: Vec<Attribute>, path: Path) -> Result<Stmt> {
        let bang_token: Token![!] = input.parse()?;
        let ident: Option<Ident> = input.parse()?;
        let (delimiter, tokens) = parse_delimiter(input)?;
        let semi_token: Option<Token![;]> = input.parse()?;

        Ok(Stmt::Item(Item::Macro(ItemMacro {
            attrs,
            ident,
            mac: Macro {
                path,
                bang_token,
                delimiter,
                tokens,
            },
            semi_token,
        })))
    }

    fn stmt_local(input: ParseStream, attrs: Vec<Attribute>, begin: ParseBuffer) -> Result<Stmt> {
        let let_token: Token![let] = input.parse()?;

        let mut pat: Pat = multi_pat_with_leading_vert(input)?;
        if input.peek(Token![:]) {
            let colon_token: Token![:] = input.parse()?;
            let ty: Type = input.parse()?;
            pat = Pat::Type(PatType {
                attrs: Vec::new(),
                pat: Box::new(pat),
                colon_token,
                ty: Box::new(ty),
            });
        }

        let init = if input.peek(Token![=]) {
            let eq_token: Token![=] = input.parse()?;
            let init: Expr = input.parse()?;

            if input.peek(Token![else]) {
                input.parse::<Token![else]>()?;
                let content;
                braced!(content in input);
                content.call(Block::parse_within)?;
                let verbatim = Expr::Verbatim(verbatim::between(begin, input));
                let semi_token: Token![;] = input.parse()?;
                return Ok(Stmt::Semi(verbatim, semi_token));
            }

            Some((eq_token, Box::new(init)))
        } else {
            None
        };

        let semi_token: Token![;] = input.parse()?;

        Ok(Stmt::Local(Local {
            attrs,
            let_token,
            pat,
            init,
            semi_token,
        }))
    }

    fn stmt_expr(
        input: ParseStream,
        allow_nosemi: bool,
        mut attrs: Vec<Attribute>,
    ) -> Result<Stmt> {
        let mut e = expr_early(input)?;

        let mut attr_target = &mut e;
        loop {
            attr_target = match attr_target {
                Expr::Assign(e) => &mut e.left,
                Expr::AssignOp(e) => &mut e.left,
                Expr::Binary(e) => &mut e.left,
                _ => break,
            };
        }
        attrs.extend(replace_attrs(attr_target, Vec::new()));
        replace_attrs(attr_target, attrs);

        if input.peek(Token![;]) {
            return Ok(Stmt::Semi(e, input.parse()?));
        }

        if allow_nosemi || !requires_terminator(&e) {
            Ok(Stmt::Expr(e))
        } else {
            Err(input.error("expected semicolon"))
        }
    }

    fn requires_terminator(expr: &Expr) -> bool {
        // see https://github.com/rust-lang/rust/blob/2679c38fc/src/librustc_ast/util/classify.rs#L7-L25
        match *expr {
            Expr::Unsafe(..)
            | Expr::Block(..)
            | Expr::If(..)
            | Expr::Match(..)
            | Expr::While(..)
            | Expr::Loop(..)
            | Expr::ForLoop(..)
            | Expr::Async(..)
            | Expr::TryBlock(..) => false,
            _ => true,
        }
    }

    fn replace_attrs(s: &mut Expr, new: Vec<Attribute>) -> Vec<Attribute> {
        match s {
            Expr::Box(ExprBox { attrs, .. })
            | Expr::Array(ExprArray { attrs, .. })
            | Expr::Call(ExprCall { attrs, .. })
            | Expr::MethodCall(ExprMethodCall { attrs, .. })
            | Expr::Tuple(ExprTuple { attrs, .. })
            | Expr::Binary(ExprBinary { attrs, .. })
            | Expr::Unary(ExprUnary { attrs, .. })
            | Expr::Lit(ExprLit { attrs, .. })
            | Expr::Cast(ExprCast { attrs, .. })
            | Expr::Type(ExprType { attrs, .. })
            | Expr::Let(ExprLet { attrs, .. })
            | Expr::If(ExprIf { attrs, .. })
            | Expr::While(ExprWhile { attrs, .. })
            | Expr::ForLoop(ExprForLoop { attrs, .. })
            | Expr::Loop(ExprLoop { attrs, .. })
            | Expr::Match(ExprMatch { attrs, .. })
            | Expr::Closure(ExprClosure { attrs, .. })
            | Expr::Unsafe(ExprUnsafe { attrs, .. })
            | Expr::Block(ExprBlock { attrs, .. })
            | Expr::Assign(ExprAssign { attrs, .. })
            | Expr::AssignOp(ExprAssignOp { attrs, .. })
            | Expr::Field(ExprField { attrs, .. })
            | Expr::Index(ExprIndex { attrs, .. })
            | Expr::Range(ExprRange { attrs, .. })
            | Expr::Path(ExprPath { attrs, .. })
            | Expr::Reference(ExprReference { attrs, .. })
            | Expr::Break(ExprBreak { attrs, .. })
            | Expr::Continue(ExprContinue { attrs, .. })
            | Expr::Return(ExprReturn { attrs, .. })
            | Expr::Macro(ExprMacro { attrs, .. })
            | Expr::Struct(ExprStruct { attrs, .. })
            | Expr::Repeat(ExprRepeat { attrs, .. })
            | Expr::Paren(ExprParen { attrs, .. })
            | Expr::Group(ExprGroup { attrs, .. })
            | Expr::Try(ExprTry { attrs, .. })
            | Expr::Async(ExprAsync { attrs, .. })
            | Expr::Await(ExprAwait { attrs, .. })
            | Expr::TryBlock(ExprTryBlock { attrs, .. })
            | Expr::Yield(ExprYield { attrs, .. }) => std::mem::replace(attrs, new),
            Expr::Verbatim(_) => Vec::new(),
            _ => unreachable!(),
        }
    }

    fn replace_attrs_item(i: &mut Item, new: Vec<Attribute>) -> Vec<Attribute> {
        match i {
            Item::ExternCrate(ItemExternCrate { attrs, .. })
            | Item::Use(ItemUse { attrs, .. })
            | Item::Static(ItemStatic { attrs, .. })
            | Item::Const(ItemConst { attrs, .. })
            | Item::Fn(ItemFn { attrs, .. })
            | Item::Mod(ItemMod { attrs, .. })
            | Item::ForeignMod(ItemForeignMod { attrs, .. })
            | Item::Type(ItemType { attrs, .. })
            | Item::Struct(ItemStruct { attrs, .. })
            | Item::Enum(ItemEnum { attrs, .. })
            | Item::Union(ItemUnion { attrs, .. })
            | Item::Trait(ItemTrait { attrs, .. })
            | Item::TraitAlias(ItemTraitAlias { attrs, .. })
            | Item::Impl(ItemImpl { attrs, .. })
            | Item::Macro(ItemMacro { attrs, .. })
            | Item::Macro2(ItemMacro2 { attrs, .. }) => std::mem::replace(attrs, new),
            Item::Verbatim(_) => Vec::new(),
            _ => unreachable!(),
        }
    }

    fn parse_delimiter(input: ParseStream) -> Result<(MacroDelimiter, TokenStream)> {
        input.step(|cursor| {
            if let Some((TokenTree::Group(g), rest)) = cursor.token_tree() {
                let span = g.span();
                let delimiter = match g.delimiter() {
                    Delimiter::Parenthesis => MacroDelimiter::Paren(Paren(span)),
                    Delimiter::Brace => MacroDelimiter::Brace(Brace(span)),
                    Delimiter::Bracket => MacroDelimiter::Bracket(Bracket(span)),
                    Delimiter::None => {
                        return Err(cursor.error("expected delimiter"));
                    }
                };
                Ok(((delimiter, g.stream()), rest))
            } else {
                Err(cursor.error("expected delimiter"))
            }
        })
    }

    fn multi_pat_with_leading_vert(input: ParseStream) -> Result<Pat> {
        let leading_vert: Option<Token![|]> = input.parse()?;
        multi_pat_impl(input, leading_vert)
    }

    fn multi_pat_impl(input: ParseStream, leading_vert: Option<Token![|]>) -> Result<Pat> {
        let mut pat: Pat = input.parse()?;
        if leading_vert.is_some()
            || input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=])
        {
            let mut cases = Punctuated::new();
            cases.push_value(pat);
            while input.peek(Token![|]) && !input.peek(Token![||]) && !input.peek(Token![|=]) {
                let punct = input.parse()?;
                cases.push_punct(punct);
                let pat: Pat = input.parse()?;
                cases.push_value(pat);
            }
            pat = Pat::Or(PatOr {
                attrs: Vec::new(),
                leading_vert,
                cases,
            });
        }
        Ok(pat)
    }

    fn expr_attrs(input: ParseStream) -> Result<Vec<Attribute>> {
        let mut attrs = Vec::new();
        loop {
            if input.peek(token::Group) {
                let ahead = input.fork();
                let group = parse_group(&ahead)?;
                if !group.content.peek(Token![#]) || group.content.peek2(Token![!]) {
                    break;
                }
                let attr = group.content.call(attr::parsing::single_parse_outer)?;
                if !group.content.is_empty() {
                    break;
                }
                attrs.push(attr);
            } else if input.peek(Token![#]) {
                attrs.push(input.call(attr::parsing::single_parse_outer)?);
            } else {
                break;
            }
        }
        Ok(attrs)
    }

    fn expr_early(input: ParseStream) -> Result<Expr> {
        let mut attrs = input.call(expr_attrs)?;
        let mut expr = if input.peek(Token![if]) {
            Expr::If(input.parse()?)
        } else if input.peek(Token![while]) {
            Expr::While(input.parse()?)
        } else if input.peek(Token![for])
            && !(input.peek2(Token![<]) && (input.peek3(Lifetime) || input.peek3(Token![>])))
        {
            Expr::ForLoop(input.parse()?)
        } else if input.peek(Token![loop]) {
            Expr::Loop(input.parse()?)
        } else if input.peek(Token![match]) {
            Expr::Match(input.parse()?)
        } else if input.peek(Token![try]) && input.peek2(token::Brace) {
            Expr::TryBlock(input.parse()?)
        } else if input.peek(Token![unsafe]) {
            Expr::Unsafe(input.parse()?)
        } else if input.peek(Token![const]) {
            Expr::Verbatim(input.call(expr_const)?)
        } else if input.peek(token::Brace) {
            Expr::Block(input.parse()?)
        } else {
            let allow_struct = AllowStruct(true);
            let mut expr = unary_expr(input, allow_struct)?;

            attrs.extend(expr.replace_attrs(Vec::new()));
            expr.replace_attrs(attrs);

            return parse_expr(input, expr, allow_struct, Precedence::Any);
        };

        if input.peek(Token![.]) && !input.peek(Token![..]) || input.peek(Token![?]) {
            expr = trailer_helper(input, expr)?;

            attrs.extend(expr.replace_attrs(Vec::new()));
            expr.replace_attrs(attrs);

            let allow_struct = AllowStruct(true);
            return parse_expr(input, expr, allow_struct, Precedence::Any);
        }

        attrs.extend(expr.replace_attrs(Vec::new()));
        expr.replace_attrs(attrs);
        Ok(expr)
    }
}
