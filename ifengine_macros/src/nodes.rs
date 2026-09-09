use quote::quote;
use syn::{
    Expr, Result, Token,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
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
            MaybeKey::Key(key_expr) => quote!(((#key_expr) as u64 & 0x0000_FFFF_FFFF_FFFF)),
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
            let content;
            syn::parenthesized!(content in input);
            let key_expr: Expr = content.parse()?;
            let _ = input.parse::<Token![,]>();
            Ok(MaybeKey::Key(key_expr))
        } else {
            Ok(MaybeKey::Auto)
        }
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

pub struct KeyExprs {
    pub maybe_key: MaybeKey,
    pub exprs: Vec<Expr>,
}

impl Parse for KeyExprs {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let exprs: Punctuated<Expr, Token![,]> = Punctuated::parse_terminated(input)?;
        Ok(KeyExprs {
            maybe_key,
            exprs: exprs.into_iter().collect(),
        })
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
