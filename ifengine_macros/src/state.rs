use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprClosure, Ident, Result, Token, parse_macro_input};

use crate::helpers::{expand_string_expr, unwrap_paren};
use crate::nodes::ExprAndOptional;

pub fn fresh(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as ExprClosure);

    let expanded = quote! {{
        if __ifengine_page_state.fresh() {
            (#closure)();
        }
    }};

    expanded.into()
}

pub fn get(input: TokenStream) -> TokenStream {
    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    if parts.is_empty() || parts.len() > 3 {
        return Error::new_spanned(
            parts,
            "get! expects 1, 2, or 3 arguments: get!(key), get!(key, when_some), or get!(key, when_some, when_none)",
        )
        .to_compile_error()
        .into();
    }

    let key_expr = &parts[0];

    let expanded = match parts.len() {
        1 => quote! {
            __ifengine_page_state.get(#key_expr)
        },
        2 => {
            let when_some = expand_string_expr(&parts[1]);
            quote! {
                match __ifengine_page_state.get(#key_expr) {
                    Some(_) => ifengine::view::Span::from(#when_some),
                    None => ifengine::view::Span::default(),
                }
            }
        }
        3 => {
            let when_some = expand_string_expr(&parts[1]);
            let when_none = expand_string_expr(&parts[2]);
            quote! {
                match __ifengine_page_state.get(#key_expr) {
                    Some(_) => ifengine::view::Span::from(#when_some),
                    None => ifengine::view::Span::from(#when_none),
                }
            }
        }
        _ => unreachable!(),
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

pub struct TagInput {
    pub expr: Expr,
    pub mode: Option<Ident>,
}

impl Parse for TagInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let expr: Expr = input.parse()?;
        let mode: Option<Ident> = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else if !input.is_empty() {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(TagInput { expr, mode })
    }
}

pub fn tag(input: TokenStream) -> TokenStream {
    let TagInput { expr, mode } = parse_macro_input!(input as TagInput);

    let sticky = match mode {
        Some(id) => match id.to_string().as_str() {
            "Sticky" => true,
            "Once" => false,
            _ => {
                return Error::new_spanned(&id, "Expected `Sticky` or `Once`")
                    .to_compile_error()
                    .into();
            }
        },
        None => false,
    };

    let expanded = quote! {
        __ifengine_page_state.tag(#expr, #sticky)
    };

    expanded.into()
}

pub fn untag(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.untag(#expr)
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
