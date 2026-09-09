use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{Error, Expr, Lit, LitStr, Token, parse_macro_input};

pub use crate::helpers::{expand_line_expr, expand_spans, expand_string_expr};

pub struct LineArgs {
    pub exprs: Vec<Expr>,
    pub trailer: Option<LitStr>,
}

impl syn::parse::Parse for LineArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut exprs = Vec::new();
        let mut trailer = None;

        while !input.is_empty() {
            if input.peek(Token![::]) {
                let _coloncolon: Token![::] = input.parse()?;
                let lit: LitStr = input.parse()?;
                trailer = Some(lit);
                break;
            }

            exprs.push(input.parse()?);

            if input.peek(Token![,]) {
                let _ = input.parse::<Token![,]>()?;
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
            ifengine::view::Object::Text(
                ifengine::view::Line::from_spans(
                    vec![#(#spans),*]
                ),
                #string_expr
            )
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
                ifengine::view::Object::Text(
                    #line,
                    #string_expr
                )
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
            ifengine::view::Object::Paragraph(
                ifengine::view::Line::from_spans(vec![#(#spans),*])
            )
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
                ifengine::view::Object::Paragraph(
                    #line
                )
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
            ifengine::view::Object::Heading(
                ifengine::view::Span::from_lingual(#text),
                #level
            )
        );
    };

    TokenStream::from(expanded)
}

pub fn hr(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.push(ifengine::view::Object::Break);
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
            ifengine::view::Object::Image(
                #image_tokens.with_id(__ifengine_page_state.auto_key())
            )
        );
    };

    TokenStream::from(expanded)
}

pub fn embed(input: TokenStream) -> TokenStream {
    let exprs = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs.into_iter().collect();

    let (target_fn, ctx_expr) = match exprs.len() {
        1 => {
            let f = &exprs[0];
            (quote!(#f), quote!(__ifengine_ctx))
        }
        2 => {
            let f = &exprs[0];
            let ctx = &exprs[1];
            (quote!(#f), quote!(#ctx))
        }
        _ => {
            return Error::new(
                proc_macro2::Span::call_site(),
                "EMBED! expects 1 or 2 arguments: EMBED!(target_page) or EMBED!(target_page, ctx)",
            )
            .to_compile_error()
            .into();
        }
    };

    let expanded = quote! {
        match __ifengine_page_state.embed(#target_fn, #ctx_expr) {
            ifengine::core::Response::View(__v) => __v,
            __other => return __other,
        }
    };

    TokenStream::from(expanded)
}
