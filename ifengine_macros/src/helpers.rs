use quote::quote;
use syn::{Expr, ExprLit, Lit, LitStr};

/// Expands string literals containing `{}` into multiple `Span::from_lingual(...)` tokens.
/// Non-string-literal expressions are passed directly to `Span::from_lingual(#expr)`.
pub fn expand_spans(exprs: impl IntoIterator<Item = Expr>) -> Vec<proc_macro2::TokenStream> {
    let mut spans = Vec::new();
    for expr in exprs {
        if let Expr::Lit(ExprLit {
            lit: Lit::Str(lit_str),
            ..
        }) = &expr
        {
            expand_format_literal(lit_str, &mut spans);
        } else {
            spans.push(quote! {
                ifengine::view::Span::from_lingual(#expr)
            });
        }
    }
    spans
}

/// Expands a single expression into a `Line` token stream. If it is a string literal,
/// parses any `{}` interpolations into spans and calls `Line::from_spans(...)`.
/// Otherwise, converts the expression via `Line::from_lingual(#expr)`.
pub fn expand_line_expr(expr: &Expr) -> proc_macro2::TokenStream {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        let mut spans = Vec::new();
        expand_format_literal(lit_str, &mut spans);
        quote! {
            ifengine::view::Line::from_spans(vec![#(#spans),*])
        }
    } else {
        quote! {
            ifengine::view::Line::from_lingual(#expr)
        }
    }
}

/// Expands a single string expression. If it is a string literal containing `{...}`,
/// it rewrites it into `format!(...)`. Otherwise returns the expression unchanged.
pub fn expand_string_expr(expr: &Expr) -> proc_macro2::TokenStream {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        let s = lit_str.value();
        if s.contains('{') {
            let mut format_str = String::new();
            let mut args = Vec::new();
            let mut rest = s.as_str();

            while !rest.is_empty() {
                if let Some(start_idx) = rest.find('{') {
                    format_str.push_str(&rest[..start_idx]);
                    let after_open = &rest[start_idx + 1..];
                    if let Some(end_idx) = after_open.find('}') {
                        let expr_str = after_open[..end_idx].trim();
                        format_str.push_str("{}");
                        match syn::parse_str::<syn::Expr>(expr_str) {
                            Ok(e) => {
                                args.push(quote!(#e));
                            }
                            Err(err) => {
                                return syn::Error::new(
                                    lit_str.span(),
                                    format!("invalid expression '{{{expr_str}}}' in string template: {err}"),
                                )
                                .to_compile_error();
                            }
                        }
                        rest = &after_open[end_idx + 1..];
                    } else {
                        format_str.push_str(rest);
                        break;
                    }
                } else {
                    format_str.push_str(rest);
                    break;
                }
            }

            return quote! { format!(#format_str, #(#args),*) };
        }
    }
    quote! { #expr }
}

pub fn expand_format_literal(lit_str: &LitStr, spans: &mut Vec<proc_macro2::TokenStream>) {
    let s = lit_str.value();
    let span = lit_str.span();
    let mut rest = s.as_str();

    while !rest.is_empty() {
        if let Some(start_idx) = rest.find('{') {
            // Push any static text before the `{`
            if start_idx > 0 {
                let leading_text = &rest[..start_idx];
                spans.push(quote! {
                    ifengine::view::Span::from_lingual(#leading_text)
                });
            }

            let after_open = &rest[start_idx + 1..];
            if let Some(end_idx) = after_open.find('}') {
                let expr_str = after_open[..end_idx].trim();
                match syn::parse_str::<syn::Expr>(expr_str) {
                    Ok(expr) => {
                        spans.push(quote! {
                            ifengine::view::Span::from_lingual(&(#expr))
                        });
                    }
                    Err(err) => {
                        let compile_err = syn::Error::new(
                            span,
                            format!(
                                "invalid expression '{{{expr_str}}}' in string template: {err}"
                            ),
                        )
                        .to_compile_error();
                        spans.push(compile_err);
                    }
                }

                rest = &after_open[end_idx + 1..];
            } else {
                // Unterminated `{`, emit the remainder as literal text
                spans.push(quote! {
                    ifengine::view::Span::from_lingual(#rest)
                });
                break;
            }
        } else {
            // No more `{`, emit remaining text
            spans.push(quote! {
                ifengine::view::Span::from_lingual(#rest)
            });
            break;
        }
    }
}
