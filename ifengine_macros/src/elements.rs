use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Error, Expr, ExprClosure, Ident, Result, Token, parse_macro_input};

pub use crate::helpers::{expand_lines, expand_string_expr};
use crate::nodes::{ExprAndOptional, MaybeKey};

pub fn s(input: TokenStream) -> TokenStream {
    let input2 = proc_macro2::TokenStream::from(input.clone());
    if let Ok(replace_input) = syn::parse2::<crate::choices::ReplaceInput>(input2)
        && replace_input.has_arrow
    {
        return crate::choices::replace_span(input);
    }

    let (maybe_key, exprs) = syn::parse::Parser::parse2(
        |input: ParseStream| {
            let maybe_key: MaybeKey = input.parse()?;
            let exprs = Punctuated::<Expr, syn::Token![,]>::parse_terminated(input)?;
            Ok((maybe_key, exprs.into_iter().collect::<Vec<_>>()))
        },
        proc_macro2::TokenStream::from(input),
    )
    .expect("failed to parse s! arguments");

    let id_tokens = maybe_key.into_tokens();

    let expanded = match exprs.len() {
        _ => {
            let expanded_exprs: Vec<_> = exprs.iter().map(expand_string_expr).collect();
            quote! {
                ifengine::view::Span::from([#( (#expanded_exprs).to_string() ),*].join(""))
                    .with_id(#id_tokens)
            }
        }
    };

    expanded.into()
}

pub fn l(input: TokenStream) -> TokenStream {
    let input2 = proc_macro2::TokenStream::from(input.clone());
    if let Ok(replace_input) = syn::parse2::<crate::choices::ReplaceInput>(input2)
        && replace_input.has_arrow
    {
        return crate::choices::replace_line(input);
    }

    let (maybe_key, exprs) = syn::parse::Parser::parse2(
        |input: ParseStream| {
            let maybe_key: MaybeKey = input.parse()?;
            let exprs = Punctuated::<Expr, syn::Token![,]>::parse_terminated(input)?;
            Ok((maybe_key, exprs))
        },
        proc_macro2::TokenStream::from(input),
    )
    .expect("failed to parse l! arguments");

    let id_tokens = maybe_key.into_tokens();
    let lines = expand_lines(exprs);

    let expanded = quote! {
        ifengine::view::Line::from_iter(vec![#(#lines),*])
            .delimit_group(#id_tokens)
    };

    expanded.into()
}

pub fn link(input: TokenStream) -> TokenStream {
    let exprs_parsed =
        parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
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
    let exprs_parsed =
        parse_macro_input!(input with Punctuated<Expr, syn::Token![,]>::parse_terminated);
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

impl AltVariant {
    /// Parses an optional `Stop:` / `Shuffle:` / `Cycle:` prefix.
    fn parse_prefixed(input: ParseStream<'_>) -> Result<Option<Self>> {
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
            "Stop" => Ok(Some(AltVariant::Stop)),
            "Shuffle" => Ok(Some(AltVariant::Shuffle)),
            "Cycle" => Ok(Some(AltVariant::Cycle)),
            _ => Err(Error::new(
                ident.span(),
                "expected AltVariant prefix: Stop: | Shuffle: | Cycle:",
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
        let variant = AltVariant::parse_prefixed(input)?;
        let expr: Expr = input.parse()?;

        let mut closure = None;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if !input.is_empty() {
                closure = Some(input.parse::<ExprClosure>()?);
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

pub struct ClickInput {
    pub maybe_key: MaybeKey,
    pub max_clicks: Option<Expr>,
    pub expr: Expr,
    pub block: Option<Expr>,
}

impl Parse for ClickInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let max_clicks = crate::nodes::parse_prefix_max_clicks(input)?;

        let expr: Expr = input.parse()?;

        let block = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                None
            } else {
                Some(input.parse::<Expr>()?)
            }
        } else {
            None
        };

        Ok(ClickInput {
            maybe_key,
            max_clicks,
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
        max_clicks,
    } = syn::parse_macro_input!(input as ClickInput);
    let key = maybe_key.into_tokens();
    let is_closure = matches!(crate::helpers::unwrap_paren(&expr), Expr::Closure(_));

    let is_unbounded = max_clicks.is_none() && (block.is_some() || is_closure);

    let max_clicks_tokens = match max_clicks {
        Some(m) => quote! { ((#m) as u64) },
        None => {
            if is_unbounded {
                quote! { u64::MAX }
            } else {
                quote! { 1u64 }
            }
        }
    };

    // A body-less click has no handler of its own: the only observable effect of the
    // click is its location key, which is not reachable from `get!`/`Y!` by content.
    // Record it under the hash of the span text as well, so an anchor like `click!("rocky")`
    // can be read back with `get!("rocky")`.
    let content_inc = if block.is_none() {
        quote! {
            let __ifengine_content_key =
                ifengine::core::key::hash_key(&span.content);
            span = span.with_action(ifengine::Action::Inc(__ifengine_content_key));
        }
    } else {
        quote! {}
    };

    let poll_action = match block {
        Some(b) => quote! {
            let _ = #b;
        },
        None => quote! {},
    };

    let span_init = if is_closure {
        quote! {
            if __ifengine_page_state.poll_click(__ifengine_key, #max_clicks_tokens, None) {
                #poll_action
            };
            let __clicks = __ifengine_page_state
                .get(__ifengine_key)
                .map(|v| v & !ifengine::core::key::DIRTY_BIT)
                .unwrap_or(0);
            let mut span = ifengine::view::Span::from(
                (#expr)(__clicks)
            )
            .with_id(__ifengine_key);
        }
    } else {
        let expr_tokens = crate::helpers::expand_string_expr(&expr);
        quote! {
            if __ifengine_page_state.poll_click(__ifengine_key, #max_clicks_tokens, None) {
                #poll_action
            };
            let mut span = ifengine::view::Span::from(
                #expr_tokens
            )
            .with_id(__ifengine_key);
        }
    };

    let expanded = quote! {{
        let __ifengine_key = #key;
        #span_init

        let __ifengine_max = #max_clicks_tokens;
        let __ifengine_unbounded = #is_unbounded;
        let __ifengine_count = __ifengine_page_state
            .get(__ifengine_key)
            .map(|v| v & !ifengine::core::key::DIRTY_BIT)
            .unwrap_or(0);

        let __ifengine_exceeded = __ifengine_count >= __ifengine_max;

        if !__ifengine_exceeded {
            span = span.with_action(ifengine::Action::SetBit(
                __ifengine_key,
                ifengine::core::key::DIRTY_BIT_POS,
            ));
            #content_inc
        }

        if __ifengine_exceeded || (__ifengine_unbounded && __ifengine_count >= 1) {
            span = span.no_sim();
        }

        span
    }};

    expanded.into()
}
