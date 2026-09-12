use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprClosure, Ident, Result, Token, parse_macro_input};

pub use crate::helpers::{expand_spans, expand_string_expr};
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
    let spans = expand_spans(exprs_parsed);

    let expanded = quote! {
        ifengine::view::Line::from_spans(vec![#(#spans),*])
            .with_id(__ifengine_page_state.auto_key())
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
    pub list: Vec<Expr>,
    pub variant: Option<AltVariant>,
}

impl Parse for AltsInput {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let maybe_key = input.parse()?;

        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);

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
        } else {
            let mut exprs: Vec<Expr> = Vec::new();
            while !input.is_empty() {
                exprs.push(input.parse()?);
                if input.peek(Token![,]) {
                    let _: Token![,] = input.parse()?;
                } else {
                    break;
                }
            }

            let mut variant = None;
            if let Some(last_expr) = exprs.last() {
                if let Expr::Path(syn::ExprPath {
                    path, qself: None, ..
                }) = last_expr
                {
                    if let Some(ident) = path.get_ident() {
                        let ident_str = ident.to_string();
                        if ident_str == "Stop" {
                            variant = Some(AltVariant::Stop);
                            exprs.pop();
                        } else if ident_str == "Shuffle" {
                            variant = Some(AltVariant::Shuffle);
                            exprs.pop();
                        } else if ident_str == "Cycle" {
                            variant = Some(AltVariant::Cycle);
                            exprs.pop();
                        }
                    }
                }
            }

            if exprs.is_empty() {
                return Err(Error::new(
                    input.span(),
                    "alts! requires at least one alternative",
                ));
            }

            Ok(Self {
                maybe_key,
                list: exprs,
                variant,
            })
        }
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
                    .with_id(__ifengine_key)
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_id(__ifengine_key)
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
                .with_id(__ifengine_key)
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
                    .with_id(__ifengine_key)
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
                    .no_sim()
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_id(__ifengine_key)
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
        .with_id(__ifengine_key)
        .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), __ifengine_key)))
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
        .with_action(ifengine::Action::SetBit((__ifengine_page_state.id(), __ifengine_key), 63))
        .as_link();

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
