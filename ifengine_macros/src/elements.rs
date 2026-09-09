use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Error, Expr, Lit, LitStr, Token, parse_macro_input};

pub use crate::helpers::{expand_line_expr, expand_spans, expand_string_expr};

fn is_trailer_next(input: syn::parse::ParseStream) -> bool {
    let ahead = input.fork();
    if ahead.parse::<Token![::]>().is_ok() {
        if ahead.parse::<LitStr>().is_ok() && ahead.is_empty() {
            return true;
        }
    }
    false
}

fn parse_until_delimiter(input: syn::parse::ParseStream) -> syn::Result<proc_macro2::TokenStream> {
    let mut tokens = proc_macro2::TokenStream::new();
    while !input.is_empty() && !input.peek(Token![,]) && !is_trailer_next(input) {
        let tt: proc_macro2::TokenTree = input.parse()?;
        tokens.extend(std::iter::once(tt));
    }
    Ok(tokens)
}

pub struct LineArgs {
    pub exprs: Vec<Expr>,
    pub trailer: Option<LitStr>,
}

impl syn::parse::Parse for LineArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut exprs = Vec::new();
        let mut trailer = None;

        while !input.is_empty() {
            if is_trailer_next(input) {
                let _coloncolon: Token![::] = input.parse()?;
                let lit: LitStr = input.parse()?;
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
                let lit: LitStr = input.parse()?;
                trailer = Some(lit);
                break;
            } else {
                break;
            }
        }

        Ok(LineArgs { exprs, trailer })
    }
}

// ----------------

pub fn push(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        __ifengine_page_state.push(
            #expr
        );
    };

    expanded.into()
}

pub fn text(input: TokenStream) -> TokenStream {
    let LineArgs { exprs, trailer } = syn::parse_macro_input!(input as LineArgs);

    let string_expr = match trailer {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let spans = expand_spans(exprs);

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(__ifengine_page_state.auto_key()),
                object: ifengine::view::Object::Text(
                    ifengine::view::Line::from_spans(
                        vec![#(#spans),*]
                    ),
                    #string_expr
                ),
            }
        );
    };

    TokenStream::from(expanded)
}

pub fn texts(input: TokenStream) -> TokenStream {
    let LineArgs { exprs, trailer } = syn::parse_macro_input!(input as LineArgs);

    let string_expr = match trailer {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let push_lines = exprs.iter().map(|expr| {
        let line = expand_line_expr(expr);
        quote! {
            __ifengine_page_state.push(
                ifengine::view::StampedObject {
                    id: Some(__ifengine_page_state.auto_key()),
                    object: ifengine::view::Object::Text(
                        #line,
                        #string_expr
                    ),
                }
            );
        }
    });

    let expanded = quote! {
        #(#push_lines)*
    };

    TokenStream::from(expanded)
}

pub fn paragraph(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let spans = expand_spans(exprs_parsed);

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(__ifengine_page_state.auto_key()),
                object: ifengine::view::Object::Paragraph(
                    ifengine::view::Line::from_spans(vec![#(#spans),*])
                ),
            }
        );
    };

    TokenStream::from(expanded)
}

pub fn paragraphs(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);

    let push_lines = exprs_parsed.iter().map(|expr| {
        let line = expand_line_expr(expr);
        quote! {
            __ifengine_page_state.push(
                ifengine::view::StampedObject {
                    id: Some(__ifengine_page_state.auto_key()),
                    object: ifengine::view::Object::Paragraph(
                        #line
                    ),
                }
            );
        }
    });

    let expanded = quote! {
        #(#push_lines)*
    };

    TokenStream::from(expanded)
}

pub fn s(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
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
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let spans = expand_spans(exprs_parsed);

    let expanded = quote! {
        ifengine::view::Line::from_spans(vec![#(#spans),*])
            .with_id(__ifengine_page_state.auto_key())
    };

    expanded.into()
}

pub fn link(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
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
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
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

pub fn h(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<&Expr> = exprs_parsed.iter().collect();

    if exprs.len() != 2 {
        return Error::new_spanned(
            exprs_parsed,
            "macro expects exactly 2 arguments: text and level",
        )
        .to_compile_error()
        .into();
    }

    let text = expand_string_expr(exprs[0]);
    let level = exprs[1];

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(__ifengine_page_state.auto_key()),
                object: ifengine::view::Object::Heading(
                    ifengine::view::Span::from_lingual(#text),
                    #level
                ),
            }
        );
    };

    TokenStream::from(expanded)
}

pub fn hr(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(__ifengine_page_state.auto_key()),
                object: ifengine::view::Object::Break,
            }
        );
    };

    TokenStream::from(expanded)
}

pub fn img(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<&Expr> = exprs_parsed.iter().collect();

    let (path_expr, size_expr) = match exprs.len() {
        1 => (exprs[0], None),
        2 => (exprs[0], Some(exprs[1])),
        _ => {
            return Error::new_spanned(exprs_parsed, "image! macro expects 1 or 2 arguments")
                .to_compile_error()
                .into();
        }
    };

    let image_tokens = if let Expr::Lit(lit) = path_expr
        && let Lit::Str(s) = &lit.lit
    {
        let path = s.value();
        if path.starts_with("http://") || path.starts_with("https://") {
            if let Some(size) = size_expr {
                quote! { ifengine::view::Image::new_url(#path).with_size(#size) }
            } else {
                quote! { ifengine::view::Image::new_url(#path) }
            }
        } else {
            if let Some(size) = size_expr {
                quote! { ifengine::view::Image::new_local(#path, include_bytes!(#path)).with_size(#size) }
            } else {
                quote! { ifengine::view::Image::new_local(#path, include_bytes!(#path)) }
            }
        }
    } else {
        return Error::new_spanned(path_expr, "expected string literal")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(__ifengine_page_state.auto_key()),
                object: ifengine::view::Object::Image(#image_tokens),
            }
        );
    };

    TokenStream::from(expanded)
}

pub struct EmbedInput {
    pub target_fn: Option<Expr>,
    pub ctx: Option<Expr>,
    pub render_data: Option<LitStr>,
}

impl syn::parse::Parse for EmbedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut target_fn = None;
        let mut ctx = None;
        let mut render_data = None;

        if is_trailer_next(input) {
            let _ = input.parse::<Token![::]>()?;
            render_data = Some(input.parse()?);
            return Ok(EmbedInput {
                target_fn,
                ctx,
                render_data,
            });
        }

        if !input.is_empty() {
            let fn_tokens = parse_until_delimiter(input)?;
            if !fn_tokens.is_empty() {
                target_fn = Some(syn::parse2(fn_tokens)?);
            }

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
                if !is_trailer_next(input) && !input.is_empty() {
                    let ctx_tokens = parse_until_delimiter(input)?;
                    if !ctx_tokens.is_empty() {
                        ctx = Some(syn::parse2(ctx_tokens)?);
                    }
                }
            }

            if is_trailer_next(input) {
                let _ = input.parse::<Token![::]>()?;
                render_data = Some(input.parse()?);
            }
        }

        Ok(EmbedInput {
            target_fn,
            ctx,
            render_data,
        })
    }
}

pub fn embed(input: TokenStream) -> TokenStream {
    let EmbedInput {
        target_fn,
        ctx,
        render_data,
    } = syn::parse_macro_input!(input as EmbedInput);

    let data = match render_data {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let expanded = match target_fn {
        None => {
            quote! {
                __ifengine_page_state.push_empty_embed(#data)
            }
        }
        Some(f) => {
            let ctx_expr = match ctx {
                Some(c) => quote!(#c),
                None => quote!(__ifengine_ctx),
            };

            quote! {
                match __ifengine_page_state.embed_with_render_data(#f, #ctx_expr, #data) {
                    ifengine::core::Response::View(__v) => __v,
                    __other => return __other,
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExtendKind {
    Choice,
    Object,
    Span,
}

pub struct ExtendInput {
    pub kind: ExtendKind,
    pub exprs: Vec<Expr>,
}

impl syn::parse::Parse for ExtendInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut kind = ExtendKind::Span;

        let ahead = input.fork();
        let mut matched_prefix = false;

        if let Ok(lit) = ahead.parse::<LitStr>() {
            if ahead.peek(Token![:]) {
                let s = lit.value();
                match s.as_str() {
                    "object" => {
                        kind = ExtendKind::Object;
                        matched_prefix = true;
                    }
                    "choice" => {
                        kind = ExtendKind::Choice;
                        matched_prefix = true;
                    }
                    "span" => {
                        kind = ExtendKind::Span;
                        matched_prefix = true;
                    }
                    _ => {}
                }
            }
        } else if let Ok(ident) = ahead.parse::<syn::Ident>() {
            if ahead.peek(Token![:]) {
                let s = ident.to_string();
                match s.as_str() {
                    "object" => {
                        kind = ExtendKind::Object;
                        matched_prefix = true;
                    }
                    "choice" => {
                        kind = ExtendKind::Choice;
                        matched_prefix = true;
                    }
                    "span" => {
                        kind = ExtendKind::Span;
                        matched_prefix = true;
                    }
                    _ => {}
                }
            }
        }

        if matched_prefix {
            let _: proc_macro2::TokenTree = input.parse()?;
            let _: Token![:] = input.parse()?;
        }

        let exprs = Punctuated::<Expr, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect();

        Ok(ExtendInput { kind, exprs })
    }
}

pub fn extend(input: TokenStream) -> TokenStream {
    let ExtendInput { kind, exprs } = parse_macro_input!(input as ExtendInput);
    let mut evals = Vec::new();
    let mut pushes = Vec::new();

    match kind {
        ExtendKind::Choice => {
            for (i, expr) in exprs.into_iter().enumerate() {
                let temp_ident = syn::Ident::new(
                    &format!("__ifengine_choice_{i}"),
                    proc_macro2::Span::call_site(),
                );
                evals.push(quote! {
                    let #temp_ident = ifengine::view::IntoNumberedLine::into_numbered_line(#expr);
                });
                pushes.push(quote! {
                    let (__num, __line) = #temp_ident;
                    let __num = match __num {
                        Some(__n) => __n,
                        None => __choices.last().map_or(0, |(prev, _)| prev.saturating_add(1)),
                    };
                    __choices.push((__num, __line));
                });
            }

            let expanded = quote! {
                {
                    #(#evals)*
                    if let Some(__ifengine_last) = __ifengine_page_state.last_mut() {
                        if let ifengine::view::Object::Choice(__choices) = &mut __ifengine_last.object {
                            #(#pushes)*
                        }
                    }
                }
            };
            TokenStream::from(expanded)
        }
        ExtendKind::Object => {
            for (i, expr) in exprs.into_iter().enumerate() {
                let temp_ident = syn::Ident::new(
                    &format!("__ifengine_obj_{i}"),
                    proc_macro2::Span::call_site(),
                );
                evals.push(quote! {
                    let #temp_ident = #expr;
                });
                pushes.push(quote! {
                    __view.push(ifengine::view::StampedObject::from(#temp_ident));
                });
            }

            let expanded = quote! {
                {
                    #(#evals)*
                    if let Some(__ifengine_last) = __ifengine_page_state.last_mut() {
                        if let ifengine::view::Object::Embed(__view, _) = &mut __ifengine_last.object {
                            #(#pushes)*
                        }
                    }
                }
            };
            TokenStream::from(expanded)
        }
        ExtendKind::Span => {
            for (i, expr) in exprs.into_iter().enumerate() {
                if let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(_),
                    ..
                }) = &expr
                {
                    let spans = expand_spans(std::iter::once(expr));
                    for (j, span) in spans.into_iter().enumerate() {
                        let sub_ident = syn::Ident::new(
                            &format!("__ifengine_span_{i}_{j}"),
                            proc_macro2::Span::call_site(),
                        );
                        evals.push(quote! {
                            let #sub_ident = #span;
                        });
                        pushes.push(quote! {
                            __line.push(#sub_ident);
                        });
                    }
                } else {
                    let temp_ident = syn::Ident::new(
                        &format!("__ifengine_span_{i}"),
                        proc_macro2::Span::call_site(),
                    );
                    evals.push(quote! {
                        let #temp_ident = #expr;
                    });
                    pushes.push(quote! {
                        __line.push(#temp_ident);
                    });
                }
            }

            let expanded = quote! {
                {
                    #(#evals)*
                    if let Some(__ifengine_last) = __ifengine_page_state.last_mut() {
                        match &mut __ifengine_last.object {
                            ifengine::view::Object::Text(__line, _)
                            | ifengine::view::Object::Paragraph(__line) => {
                                #(#pushes)*
                            }
                            _ => {}
                        }
                    }
                }
            };
            TokenStream::from(expanded)
        }
    }
}
