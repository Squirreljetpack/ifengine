use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprClosure, Ident, Result, Token, parse_macro_input};

pub use crate::helpers::{expand_lines, expand_string_expr};
use crate::nodes::{ExprAndOptional, MaybeKey};

pub fn s(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs_parsed.into_iter().collect();

    let expanded = match exprs.len() {
        0 => quote! {
            ifengine::view::Span::new(String::new())
                .with_id(__ifengine_page_state.auto_key())
        },
        1 => {
            let expr = expand_string_expr(&exprs[0]);
            quote! {
                ifengine::view::Span::from(#expr)
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
        _ => {
            let expanded_exprs: Vec<_> = exprs.iter().map(expand_string_expr).collect();
            quote! {
                ifengine::view::Span::from([#( (#expanded_exprs).to_string() ),*].join(""))
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
    };

    expanded.into()
}

pub fn l(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
    let lines = expand_lines(exprs_parsed);

    let expanded = quote! {
        ifengine::view::Line::from_iter(vec![#(#lines),*])
    };

    expanded.into()
}

pub fn link(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs_parsed.into_iter().collect();

    let expanded = match exprs.len() {
        1 => {
            let text = expand_string_expr(&exprs[0]);
            quote! {
                ifengine::view::Span::from(#text)
                    .as_link()
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
        2 => {
            let text = expand_string_expr(&exprs[0]);
            let target = &exprs[1];
            quote! {
                ifengine::view::Span::from(#text)
                    .as_link()
                    .with_action(ifengine::Action::Next(ifengine::core::PageHandle::new(
                        stringify!(#target).into(),
                        #target,
                    )))
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "link! takes 1 or 2 arguments: link!(text) or link!(text, target)",
            )
            .to_compile_error()
            .into();
        }
    };

    expanded.into()
}

pub fn tun(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs_parsed.into_iter().collect();

    let expanded = match exprs.len() {
        1 => {
            let text = expand_string_expr(&exprs[0]);
            quote! {
                ifengine::view::Span::from(#text)
                    .as_link()
                    .with_action(ifengine::Action::Exit)
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
        2 => {
            let text = expand_string_expr(&exprs[0]);
            let target = &exprs[1];
            quote! {
                ifengine::view::Span::from(#text)
                    .as_link()
                    .with_action(ifengine::Action::Tunnel(ifengine::core::PageHandle::new(
                        stringify!(#target).into(),
                        #target,
                    )))
                    .with_id(__ifengine_page_state.auto_key())
            }
        }
        _ => {
            return syn::Error::new(
                proc_macro2::Span::call_site(),
                "tun! takes 1 or 2 arguments: tun!(text) or tun!(text, target)",
            )
            .to_compile_error()
            .into();
        }
    };

    expanded.into()
}

pub fn back(input: TokenStream) -> TokenStream {
    let ExprAndOptional { expr, n } = parse_macro_input!(input as ExprAndOptional);
    let text = expand_string_expr(&expr);

    let expanded = if let Some(n_expr) = n {
        quote! {
            ifengine::view::Span::from(#text)
            .as_link()
            .with_action(ifengine::Action::Back(#n_expr))
            .with_id(__ifengine_page_state.auto_key())
        }
    } else {
        quote! {
            ifengine::view::Span::from(#text)
            .as_link()
            .with_action(ifengine::Action::Back(1))
            .with_id(__ifengine_page_state.auto_key())
            .no_sim()
        }
    };

    TokenStream::from(expanded)
}

// ---------------- Interactive Spans ----------------

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
    pub expr: Expr,
    pub variant: Option<AltVariant>,
    pub closure: Option<ExprClosure>,
}

impl Parse for AltsInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let maybe_key = input.parse()?;
        let expr: Expr = input.parse()?;

        let mut variant = None;
        let mut closure = None;

        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                let is_variant = if input.peek(syn::Ident) {
                    let fork = input.fork();
                    if let Ok(ident) = fork.parse::<syn::Ident>() {
                        matches!(ident.to_string().as_str(), "Stop" | "Shuffle" | "Cycle")
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_variant {
                    variant = Some(input.parse::<AltVariant>()?);
                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                        if !input.is_empty() {
                            closure = Some(input.parse::<ExprClosure>()?);
                        }
                    }
                } else {
                    closure = Some(input.parse::<ExprClosure>()?);
                }
            }
        }

        Ok(Self {
            maybe_key,
            expr,
            variant,
            closure,
        })
    }
}

pub fn alts(input: TokenStream) -> TokenStream {
    let AltsInput {
        maybe_key,
        expr,
        variant,
        closure,
    } = parse_macro_input!(input as AltsInput);

    let key = maybe_key.into_tokens();

    let variant = variant.unwrap_or_default();
    let list_init = quote! { (#expr).as_ref() };

    let closure_call = if let Some(c) = closure {
        quote! {
            {
                fn __ifengine_alts_closure<R>(mut f: impl FnMut(usize) -> R, idx: usize) {
                    let _ = f(idx);
                }
                __ifengine_alts_closure(#c, __idx as usize);
            }
        }
    } else {
        quote! {}
    };

    let expanded = match variant {
        AltVariant::Stop => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                let __idx = if let Some(idx) = __ifengine_page_state.get(__ifengine_key) {
                    (idx as usize).min(alts.len().saturating_sub(1))
                } else {
                    0
                };

                #closure_call

                let span = ifengine::view::Span::from(alts[__idx].clone()).with_id(__ifengine_key);
                if alts.len() > 1 && __idx < alts.len() - 1 {
                    span.with_action(ifengine::Action::Inc(__ifengine_key))
                } else {
                    span
                }
            }}
        }

        AltVariant::Shuffle => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                // Determine tmp index
                let __idx = if let Some(prev) = __ifengine_page_state.get(__ifengine_key) {
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

                #closure_call

                let span = ifengine::view::Span::from(alts[__idx].clone()).with_id(__ifengine_key);
                if alts.len() > 1 {
                    span.with_action(ifengine::Action::Set(
                        __ifengine_key,
                        ((__idx as u64) << 1) + 1
                    ))
                    .no_sim()
                } else {
                    span
                }
            }}
        }

        AltVariant::Cycle => {
            quote! {{
                let __ifengine_key = #key;
                let alts = #list_init;

                let __idx = if let Some(idx) = __ifengine_page_state.get(__ifengine_key) {
                    (idx as usize) % alts.len()
                } else {
                    0
                };

                #closure_call

                let span = ifengine::view::Span::from(alts[__idx].clone()).with_id(__ifengine_key);
                if alts.len() > 1 {
                    span.with_action(ifengine::Action::Inc(__ifengine_key))
                        .no_sim()
                } else {
                    span
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
        .with_id(__ifengine_key)
        .with_action(ifengine::Action::Inc(__ifengine_key))
        .no_sim()
    }};

    expanded.into()
}

pub struct ClickInput {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
    pub block: Expr,
    pub max_clicks: Option<Expr>,
}

impl Parse for ClickInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;

        let expr: Expr = input.parse()?;

        let block = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                syn::parse_quote!({})
            } else {
                input.parse::<Expr>()?
            }
        } else {
            syn::parse_quote!({})
        };

        let max_clicks = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                None
            } else {
                let m = input.parse::<Expr>()?;
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                }
                Some(m)
            }
        } else {
            None
        };

        Ok(ClickInput {
            maybe_key,
            expr,
            block,
            max_clicks,
        })
    }
}

pub fn click(input: TokenStream) -> TokenStream {
    let ClickInput {
        maybe_key,
        expr,
        block,
        max_clicks,
    } = syn::parse_macro_input!(input as ClickInput);
    let key = maybe_key.into_tokens();
    let expr_tokens = crate::helpers::expand_string_expr(&expr);
    let max_clicks_tokens = match max_clicks {
        Some(m) => quote! { ((#m) as u64) },
        None => quote! { 0u64 },
    };

    let expanded = quote! {{
        let __ifengine_key = #key;
        if __ifengine_page_state.poll_click(__ifengine_key, #max_clicks_tokens) {
            let _ = #block;
        };

        let span = ifengine::view::Span::from(
            #expr_tokens
        )
        .with_id(__ifengine_key)
        .with_action(ifengine::Action::SetBit(__ifengine_key, 63));

        // sim the handler
        if __ifengine_page_state.get(__ifengine_key).is_some_and(|v| {
            let max = #max_clicks_tokens;
            let count = v & !(1u64 << 63);
            if max == 0 {
                count >= 1
            } else {
                count >= max
            }
        }) {
            span.no_sim()
        } else {
            span
        }
    }};

    expanded.into()
}
