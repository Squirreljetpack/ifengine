use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::visit_mut::VisitMut;
use syn::{Error, Expr, ItemFn, Lit, LitStr, Token, parse_macro_input};

use crate::helpers::{expand_line_expr, expand_spans, expand_string_expr};
use crate::nodes::{LineArgs, is_trailer_next, parse_until_delimiter};

pub fn ifview(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemFn);

    let name = &input.sig.ident;

    if input.sig.inputs.len() != 1 {
        return Error::new_spanned(
            &input.sig.inputs,
            "ifview functions must have exactly one input: the context type C",
        )
        .to_compile_error()
        .into();
    }

    let ctx_arg = input.sig.inputs.first().unwrap();
    let (ctx_type, ctx_ident) = if let syn::FnArg::Typed(pat_type) = ctx_arg
        && let syn::Type::Reference(ty_ref) = &*pat_type.ty
        && ty_ref.mutability.is_some()
    {
        let ident = match &*pat_type.pat {
            syn::Pat::Ident(p) => p.ident.clone(),
            _ => syn::Ident::new("__ifengine_ctx", proc_macro2::Span::call_site()),
        };
        (&*ty_ref.elem, ident)
    } else {
        return Error::new_spanned(ctx_arg, "Expected a &mut C type")
            .to_compile_error()
            .into();
    };

    struct EmbedRewriter<'a> {
        ctx_ident: &'a syn::Ident,
    }

    impl<'a> VisitMut for EmbedRewriter<'a> {
        fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
            if mac.path.is_ident("EMBED")
                || mac
                    .path
                    .segments
                    .last()
                    .map_or(false, |s| s.ident == "EMBED")
            {
                if let Ok(embed_input) = syn::parse2::<EmbedInput>(mac.tokens.clone()) {
                    if embed_input.ctx.is_none() && embed_input.target_fn.is_some() {
                        let target = embed_input.target_fn.unwrap();
                        let ctx = self.ctx_ident;
                        let trailer = match embed_input.render_data {
                            Some(rd) => quote!(:: #rd),
                            None => quote!(),
                        };
                        mac.tokens = quote!(#target, #ctx #trailer);
                    }
                }
            }
            syn::visit_mut::visit_macro_mut(self, mac);
        }
    }

    EmbedRewriter {
        ctx_ident: &ctx_ident,
    }
    .visit_block_mut(&mut input.block);

    let original_block = &input.block;

    let bind_ctx = quote! {
        #[allow(unused_variables, unused_mut)]
        let mut __ifengine_ctx = &mut __ifengine_game.context;
        #[allow(unused_variables)]
        let #ctx_arg = &mut *__ifengine_ctx;
    };

    let expanded = quote! {
        pub fn #name(__ifengine_game: &mut ifengine::Game<#ctx_type>)
        -> ifengine::core::Response
        {
            let __ifengine_simulating = __ifengine_game.simulating();
            #bind_ctx
            let __ifengine_game_tags = &mut __ifengine_game.tags;
            let __ifengine_game = &mut __ifengine_game.inner;
            let __ifengine_page_id: &'static str = concat!(module_path!(), "::", stringify!(#name));
            let mut __ifengine_page_state = ifengine::core::PageState::new(
                __ifengine_game.page_id(__ifengine_page_id),
                __ifengine_game.fresh(),
                __ifengine_simulating,
                __ifengine_game.state.get_page_mut(__ifengine_page_id),
                __ifengine_game_tags,
            );

            #original_block

            #[allow(unreachable_code)]
            __ifengine_page_state.into_response()
        }

        ifengine::inventory::submit! {
            ifengine::core::RegisteredPage {
                id: concat!(module_path!(), "::", stringify!(#name)),
                factory: |id: ifengine::core::PageId| ifengine::core::PageHandle::new::<#ctx_type>(id, #name),
            }
        }
    };

    expanded.into()
}

// ---------------- View Direct Push Macros ----------------

pub fn push(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        __ifengine_page_state.push(
            #expr
        );
    };

    expanded.into()
}

pub fn clear(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.clear();
    };

    TokenStream::from(expanded)
}

pub fn paragraph(input: TokenStream) -> TokenStream {
    let LineArgs { maybe_key, exprs, trailer } = syn::parse_macro_input!(input as LineArgs);

    let key = maybe_key.into_tokens();
    let string_expr = match trailer {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let spans = expand_spans(exprs);

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::StampedObject {
                id: Some(#key),
                object: ifengine::view::Object::Paragraph(
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

pub fn paragraphs(input: TokenStream) -> TokenStream {
    let LineArgs { exprs, trailer, .. } = syn::parse_macro_input!(input as LineArgs);

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
                    object: ifengine::view::Object::Paragraph(
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
                    ifengine::view::Span::from(#text),
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

#[allow(non_snake_case)]
pub fn r#YIELD(input: TokenStream) -> TokenStream {
    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    if parts.is_empty() {
        let expanded = quote! {
            return __ifengine_page_state.into_response()
        };
        return expanded.into();
    }

    if parts.len() > 1 {
        return Error::new_spanned(
            parts,
            "YIELD! expects 0 or 1 argument: YIELD!() or YIELD!(key)",
        )
        .to_compile_error()
        .into();
    }

    let key_expr = &parts[0];
    let expanded = quote! {
        if __ifengine_page_state.get(#key_expr).is_none() {
            return __ifengine_page_state.into_response();
        }
    };
    expanded.into()
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
                    lit: Lit::Str(_), ..
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
                            ifengine::view::Object::Paragraph(__line, _) => {
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
