use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Parser};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprClosure, Ident, Result, Token, parse_macro_input};

use crate::nodes::{ExprAndOptional, MaybeKey};

#[derive(Clone, Default)]
pub enum AltVariant {
    #[default]
    Stop,
    Shuffle,
    Cycle,
}

impl Parse for AltVariant {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "Stop" => Ok(AltVariant::Stop),
            "Shuffle" => Ok(AltVariant::Shuffle),
            "Cycle" => Ok(AltVariant::Cycle),
            _ => Err(Error::new(
                ident.span(),
                "expected AltVariant: Stop | Shuffle | Cycle",
            )),
        }
    }
}

pub struct AltsInput {
    pub maybe_key: MaybeKey,
    pub list: Vec<Expr>,
    pub variant: Option<AltVariant>,
}

impl Parse for AltsInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let content;
        syn::bracketed!(content in input);

        let maybe_key = input.parse()?;

        let mut list = Vec::new();
        while !content.is_empty() {
            list.push(content.parse()?);
            if content.peek(Token![,]) {
                let _: Token![,] = content.parse()?;
            }
        }

        let variant = if !input.is_empty() {
            input.parse::<Token![,]>()?;
            Some(input.parse()?)
        } else {
            None
        };

        Ok(Self {
            maybe_key,
            list,
            variant,
        })
    }
}

pub fn alts(input: TokenStream) -> TokenStream {
    let AltsInput {
        maybe_key,
        list,
        variant,
    } = parse_macro_input!(input as AltsInput);

    let key = maybe_key.into_tokens();

    let variant = variant.unwrap_or_default();
    let list_init = quote! { &[ #(#list),* ] };

    let expanded = match variant {
        AltVariant::Stop => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                if let Some(idx) = __ifengine_page_state.get(__ifengine_key) {
                    ifengine::view::Span::from(
                        alts[(idx as usize + 1).min(alts.len() - 1)]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                }
            }}
        }

        AltVariant::Shuffle => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                // Determine tmp index
                let idx = if let Some(prev) = __ifengine_page_state.get(__ifengine_key) {
                    if prev & 1 == 0 {
                        (prev as usize) >> 1
                    } else {
                        // regenerate, excluding previous index
                        let new_idx = __ifengine_page_state.rand(alts.len(), &[(prev as usize) >> 1]);
                        __ifengine_page_state.insert(__ifengine_key, (new_idx as u64) << 1);
                        new_idx
                    }
                } else {
                    let new_idx = __ifengine_page_state.rand(alts.len(), &[]);
                    __ifengine_page_state.insert(__ifengine_key, (new_idx as u64) << 1);
                    new_idx
                };

                // Use it and store back with last bit set
                ifengine::view::Span::from(alts[idx])
                .with_action(ifengine::Action::Set(
                    (__ifengine_page_state.id(), __ifengine_key),
                    ((idx as u64) << 1) + 1
                ))
                .no_sim()
            }}
        }

        AltVariant::Cycle => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                if let Some(idx) = __ifengine_page_state.get(__ifengine_key) {
                    ifengine::view::Span::from(
                        alts[(idx as usize) % alts.len()]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                    .no_sim()
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                    .no_sim()
                }
            }}
        }
    };

    expanded.into()
}

pub struct CountInput {
    pub maybe_key: MaybeKey,
    pub closure: ExprClosure,
}

impl Parse for CountInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let closure = input.parse()?;

        Ok(CountInput { maybe_key, closure })
    }
}

pub fn count(input: TokenStream) -> TokenStream {
    let CountInput { maybe_key, closure } = syn::parse_macro_input!(input as CountInput);
    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let __ifengine_key = #key;
        ifengine::view::Span::from(
            (#closure)(__ifengine_page_state.get(__ifengine_key).unwrap_or_default())
        )
        .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
        .no_sim()
    }};

    expanded.into()
}

pub struct ClickInput {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
    pub block: Expr,
}

impl Parse for ClickInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;

        let expr: Expr = input.parse()?;

        let block = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            input.parse::<Expr>()?
        } else {
            syn::parse_quote!({})
        };

        Ok(ClickInput {
            maybe_key,
            expr,
            block,
        })
    }
}

pub fn click(input: TokenStream) -> TokenStream {
    let ClickInput {
        maybe_key,
        expr,
        block,
    } = syn::parse_macro_input!(input as ClickInput);
    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let __ifengine_key = #key;
        if __ifengine_page_state.was_zero(__ifengine_key) {
            let _ = #block;
        };

        let span = ifengine::view::Span::from(
            #expr
        )
        .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
        .as_link();

        // sim the handler once
        if __ifengine_page_state.get(__ifengine_key).is_some() {
            span.no_sim()
        } else {
            span
        }
    }};

    expanded.into()
}

pub fn fresh(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as ExprClosure);

    let expanded = quote! {{
        if __ifengine_page_state.fresh() {
            (#closure)();
        }
    }};

    expanded.into()
}

pub fn back(input: TokenStream) -> TokenStream {
    let ExprAndOptional { expr, n } = parse_macro_input!(input as ExprAndOptional);

    let expanded = if let Some(n_expr) = n {
        quote! {
            ifengine::view::Span::from(#expr)
            .as_link()
            .with_action(ifengine::Action::Back(#n_expr))
        }
    } else {
        quote! {
            ifengine::view::Span::from(#expr)
            .as_link()
            .with_action(ifengine::Action::Back(1))
            .no_sim()
        }
    };

    TokenStream::from(expanded)
}

#[allow(non_snake_case)]
pub fn r#YIELD(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        return __ifengine_page_state.into_response()
    };
    expanded.into()
}

pub fn read_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.get(#expr)
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

pub fn set_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.insert(#expr.0, #expr.1)
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
            let old = __ifengine_page_state.get(#key).unwrap_or(0u64);
            __ifengine_page_state.insert(#key, old | #mask);
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
            let old = __ifengine_page_state.get(#key).unwrap_or(0u64);
            __ifengine_page_state.insert(#key, old & !#mask);
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
