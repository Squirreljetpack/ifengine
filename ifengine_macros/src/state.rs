use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, Ident, Result, Token, parse_macro_input};

use crate::helpers::{condition_of, expand_string_expr, unwrap_paren};
use crate::nodes::ExprAndOptional;

pub struct OnceInput {
    pub limit: Option<Expr>,
    pub body: proc_macro2::TokenStream,
}

impl Parse for OnceInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let limit = crate::nodes::parse_prefix_max_clicks(input)?;
        let body: proc_macro2::TokenStream = input.parse()?;
        Ok(OnceInput { limit, body })
    }
}

pub fn once(input: TokenStream) -> TokenStream {
    let OnceInput { limit, body } = parse_macro_input!(input as OnceInput);

    let expanded = match limit {
        Some(lim) => {
            quote! {{
                let __limit = (#lim) as u64;
                if __limit == 0 {
                    if __ifengine_page_state.fresh() {
                        #body
                    }
                } else {
                    let __key = __ifengine_page_state.auto_key();
                    let __current = __ifengine_page_state.get(__key).unwrap_or(0);
                    if __current < __limit {
                        __ifengine_page_state.insert(__key, __current + 1);
                        #body
                    }
                }
            }}
        }
        None => {
            quote! {{
                let __key = __ifengine_page_state.auto_key();
                let __current = __ifengine_page_state.get(__key).unwrap_or(0);
                if __current < 1 {
                    __ifengine_page_state.insert(__key, __current + 1);
                    #body
                }
            }}
        }
    };

    expanded.into()
}

enum GetInput {
    KeyOnly(Expr),
    Conditional {
        key_expr: Expr,
        when_none: Option<Expr>,
        when_some: Expr,
    },
}

/// Parses one side of a `get!` arrow: exactly one expression implementing `Into<Line>`.
fn parse_line_expr(tokens: proc_macro2::TokenStream) -> Result<Expr> {
    syn::parse2(tokens)
}

fn build_line(expr: &Expr) -> proc_macro2::TokenStream {
    let expr = expand_string_expr(expr);
    quote! {
        ifengine::view::Line::from(#expr)
    }
}

fn parse_get_input(input: ParseStream) -> Result<GetInput> {
    let key_expr: Expr = input.parse()?;

    if input.is_empty() {
        return Ok(GetInput::KeyOnly(key_expr));
    }

    let _comma: Token![,] = input.parse()?;

    let mut first_tokens = proc_macro2::TokenStream::new();
    let mut has_arrow = false;
    let mut is_reverse = false;

    while !input.is_empty() {
        if input.peek(Token![=>]) {
            let _arrow: Token![=>] = input.parse()?;
            has_arrow = true;
            break;
        }
        if input.peek(Token![<=]) {
            let _arrow: Token![<=] = input.parse()?;
            has_arrow = true;
            is_reverse = true;
            break;
        }
        let tt: proc_macro2::TokenTree = input.parse()?;
        first_tokens.extend(std::iter::once(tt));
    }

    let (when_none, when_some) = if has_arrow {
        let second_tokens = input.parse::<proc_macro2::TokenStream>()?;
        let first = parse_line_expr(first_tokens)?;
        let second = parse_line_expr(second_tokens)?;

        if is_reverse {
            (Some(second), first)
        } else {
            (Some(first), second)
        }
    } else {
        (None, parse_line_expr(first_tokens)?)
    };

    Ok(GetInput::Conditional {
        key_expr,
        when_none,
        when_some,
    })
}

pub fn get(input: TokenStream) -> TokenStream {
    if input.is_empty() {
        return Error::new(
            proc_macro2::Span::call_site(),
            "get! expects at least 1 argument: get!(key) or get!(key, [when_none =>] when_some)",
        )
        .to_compile_error()
        .into();
    }

    let parsed =
        match syn::parse::Parser::parse2(parse_get_input, proc_macro2::TokenStream::from(input)) {
            Ok(parsed) => parsed,
            Err(e) => return e.to_compile_error().into(),
        };

    let expanded = match parsed {
        GetInput::KeyOnly(key_expr) => {
            quote! {
                __ifengine_page_state.get(#key_expr)
            }
        }
        GetInput::Conditional {
            key_expr,
            when_none,
            when_some,
        } => {
            let condition = match condition_of(&key_expr) {
                Ok(condition) => condition,
                Err(e) => return e.to_compile_error().into(),
            };
            let when_none_line = match &when_none {
                Some(expr) => build_line(expr),
                None => quote! { ifengine::view::Line::default() },
            };
            let when_some_line = build_line(&when_some);

            quote! {
                if #condition {
                    #when_some_line
                } else {
                    #when_none_line
                }
            }
        }
    };

    expanded.into()
}

pub fn read_key_mask(input: TokenStream) -> TokenStream {
    let ExprAndOptional { expr: key, n } = syn::parse_macro_input!(input as ExprAndOptional);

    let n = n.unwrap_or_else(|| syn::parse_quote!(64));

    quote! {
        __ifengine_page_state.get_mask::<#n>(#key)
    }
    .into()
}

pub fn v(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as Expr);

    match condition_of(&expr) {
        Ok(condition) => condition.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

pub fn set(input: TokenStream) -> TokenStream {
    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    if parts.is_empty() || parts.len() > 2 {
        return Error::new_spanned(
            parts,
            "set! expects 1 or 2 arguments: set!(key) or set!(key, value)",
        )
        .to_compile_error()
        .into();
    }

    let (key, val) = if parts.len() == 1 {
        let unwrapped = unwrap_paren(&parts[0]);
        if let Expr::Tuple(tuple) = unwrapped {
            if tuple.elems.len() == 2 {
                (&tuple.elems[0], Some(&tuple.elems[1]))
            } else {
                (&parts[0], None)
            }
        } else {
            (&parts[0], None)
        }
    } else {
        (&parts[0], Some(&parts[1]))
    };

    let val_expr = match val {
        Some(v) => quote!(#v),
        None => quote!(0u64),
    };

    let expanded = quote! {
        __ifengine_page_state.insert(#key, #val_expr)
    };

    expanded.into()
}

pub fn set_key_mask(input: TokenStream) -> TokenStream {
    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut iter = parts.iter();
    let key = if let Some(key) = iter.next() {
        key
    } else {
        return Error::new_spanned(parts, "expected key")
            .to_compile_error()
            .into();
    };
    let bits: Vec<&Expr> = iter.collect();

    let mut mask = 0u64;
    for expr in &bits {
        if let Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) = expr
        {
            match i.base10_parse::<usize>() {
                Ok(bit) => mask |= 1u64 << bit,
                Err(_) => {
                    return Error::new_spanned(i, "failed to parse bit position")
                        .to_compile_error()
                        .into();
                }
            }
        } else {
            return Error::new_spanned(expr, "bit positions must be integer literals")
                .to_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        {
            let k = #key;
            let old = __ifengine_page_state.get(k).unwrap_or(0u64);
            __ifengine_page_state.insert(k, old | #mask);
        }
    };

    expanded.into()
}

pub fn unset_key_mask(input: TokenStream) -> TokenStream {
    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut iter = parts.iter();
    let key = if let Some(key) = iter.next() {
        key
    } else {
        return Error::new_spanned(parts, "expected key")
            .to_compile_error()
            .into();
    };
    let bits: Vec<&Expr> = iter.collect();

    let mut mask = 0u64;
    for expr in &bits {
        if let Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(i),
            ..
        }) = expr
        {
            match i.base10_parse::<usize>() {
                Ok(bit) => mask |= 1u64 << bit,
                Err(_) => {
                    return Error::new_spanned(i, "failed to parse bit position")
                        .to_compile_error()
                        .into();
                }
            }
        } else {
            return Error::new_spanned(expr, "bit positions must be integer literals")
                .to_compile_error()
                .into();
        }
    }

    let expanded = quote! {
        {
            let k = #key;
            let old = __ifengine_page_state.get(k).unwrap_or(0u64);
            __ifengine_page_state.insert(k, old & !#mask);
        }
    };

    expanded.into()
}

pub fn inc_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        {
            let k = #expr;
            let v = __ifengine_page_state.get(k).unwrap_or(0);
            __ifengine_page_state.insert(k, v.wrapping_add(1));
        }
    };

    expanded.into()
}

pub fn reset_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.remove(#expr)
    };

    expanded.into()
}

#[derive(Clone, Copy)]
pub enum TagMode {
    Once,
    Sticky,
    Game,
}

pub struct TagInput {
    pub expr: Expr,
    pub mode: Option<TagMode>,
}

fn parse_tag_mode(input: ParseStream) -> Result<Option<TagMode>> {
    if !input.peek(Ident) {
        return Ok(None);
    }
    let fork = input.fork();
    let Ok(ident) = fork.parse::<Ident>() else {
        return Ok(None);
    };
    if !fork.peek(Token![:]) || fork.peek(Token![::]) {
        return Ok(None);
    }
    let _ident: Ident = input.parse()?;
    let _colon: Token![:] = input.parse()?;
    match ident.to_string().as_str() {
        "Once" => Ok(Some(TagMode::Once)),
        "Sticky" => Ok(Some(TagMode::Sticky)),
        "Game" => Ok(Some(TagMode::Game)),
        _ => Err(Error::new(
            ident.span(),
            "expected `Once:`, `Sticky:` or `Game:` prefix",
        )),
    }
}

fn tag_scope(mode: Option<TagMode>) -> proc_macro2::TokenStream {
    match mode {
        Some(TagMode::Once) => quote! { ifengine::core::TagScope::Once },
        Some(TagMode::Game) => quote! { ifengine::core::TagScope::Game },
        _ => quote! { ifengine::core::TagScope::Sticky },
    }
}

impl Parse for TagInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mode = parse_tag_mode(input)?;
        let expr: Expr = input.parse()?;
        Ok(TagInput { expr, mode })
    }
}

pub fn tag(input: TokenStream) -> TokenStream {
    let TagInput { expr, mode } = parse_macro_input!(input as TagInput);

    let scope = tag_scope(mode);

    let expanded = quote! {
        __ifengine_page_state.tag(#expr, #scope)
    };

    expanded.into()
}

pub fn untag(input: TokenStream) -> TokenStream {
    let TagInput { expr, mode } = parse_macro_input!(input as TagInput);

    let scope = tag_scope(mode);

    let expanded = quote! {
        __ifengine_page_state.untag(#expr, #scope)
    };

    expanded.into()
}

pub fn in_sim(_: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.simulating
    };

    expanded.into()
}

pub fn page_dbg(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        dbg!(&__ifengine_page_state)
    };
    expanded.into()
}

pub fn view_dbg(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        dbg!(&__ifengine_page_state.view)
    };
    expanded.into()
}
