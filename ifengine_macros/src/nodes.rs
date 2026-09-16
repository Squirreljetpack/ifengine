use quote::quote;
use syn::{
    Expr, Result, Token,
    parse::{Parse, ParseStream},
};

/// Optional u64 key specified in the first position, surrounded in brackets.
/// The internal data describing an element is stored under this key in the page state and can be retrieved for full fine-grained control.
/// # Syntax
/// ```rust,ignore
/// let span_count = read_key!(6); // Can be called before
/// let span = count!((6), |val| "span");
/// ```
pub enum MaybeKey {
    Auto,
    Key(Expr),
}

impl MaybeKey {
    pub fn into_tokens(self) -> proc_macro2::TokenStream {
        match self {
            MaybeKey::Key(key_expr) => quote! {
                ifengine::core::key::IntoPageKey::into_page_key(#key_expr)
            },
            MaybeKey::Auto => {
                quote! {
                    __ifengine_page_state.auto_key()
                }
            }
        }
    }
}

impl Parse for MaybeKey {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(syn::token::Paren) {
            let ahead = input.fork();
            let content;
            syn::parenthesized!(content in ahead);
            if content.parse::<Expr>().is_ok() && ahead.peek(Token![,]) {
                let content;
                syn::parenthesized!(content in input);
                let key_expr: Expr = content.parse()?;
                let _comma: Token![,] = input.parse()?;
                return Ok(MaybeKey::Key(key_expr));
            }
        }
        Ok(MaybeKey::Auto)
    }
}

pub struct KeyExpr {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
}

impl Parse for KeyExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let expr: Expr = input.parse()?;

        Ok(KeyExpr { maybe_key, expr })
    }
}

pub struct ExprAndOptional {
    pub expr: syn::Expr,
    pub n: Option<syn::Expr>,
}

impl syn::parse::Parse for ExprAndOptional {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let key: syn::Expr = input.parse()?;

        let n = if input.peek(syn::Token![,]) {
            input.parse::<syn::Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self { expr: key, n })
    }
}

pub(crate) fn is_trailer_next(input: ParseStream) -> bool {
    let ahead = input.fork();
    if ahead.parse::<Token![::]>().is_ok() {
        if ahead.parse::<syn::LitStr>().is_ok() && ahead.is_empty() {
            return true;
        }
    }
    false
}

pub(crate) fn parse_until_delimiter(input: ParseStream) -> Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();
    while !input.is_empty() && !input.peek(Token![,]) && !is_trailer_next(input) {
        let tt: proc_macro2::TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }
    Ok(tokens)
}

pub struct LineArgs {
    pub maybe_key: MaybeKey,
    pub exprs: Vec<Expr>,
    pub trailer: Option<syn::LitStr>,
}

impl Parse for LineArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let mut exprs = Vec::new();
        let mut trailer = None;

        while !input.is_empty() {
            if is_trailer_next(input) {
                let _coloncolon: Token![::] = input.parse()?;
                let lit: syn::LitStr = input.parse()?;
                trailer = Some(lit);
                break;
            }

            let expr_tokens = parse_until_delimiter(input)?;
            if !expr_tokens.is_empty() {
                exprs.push(syn::parse2(expr_tokens)?);
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
            } else if is_trailer_next(input) {
                let _coloncolon: Token![::] = input.parse()?;
                let lit: syn::LitStr = input.parse()?;
                trailer = Some(lit);
                break;
            } else {
                break;
            }
        }

        Ok(LineArgs {
            maybe_key,
            exprs,
            trailer,
        })
    }
}

