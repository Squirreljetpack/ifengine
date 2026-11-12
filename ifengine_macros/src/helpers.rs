use quote::quote;
use syn::{Expr, ExprLit, Lit, LitStr};

/// Strips outer parenthesized expression (e.g. `(expr)` -> `expr`).
pub fn unwrap_paren(expr: &Expr) -> &Expr {
    match expr {
        Expr::Paren(p) => &p.expr,
        _ => expr,
    }
}

#[derive(Clone)]
pub enum KeyExprTree {
    Key(Expr),
    And(Vec<KeyExprTree>),
    Or(Vec<KeyExprTree>),
}

pub fn parse_keys(expr: &Expr) -> syn::Result<KeyExprTree> {
    parse_key_tree(expr)
}

/// Converts a parsed key expression tree into a boolean condition. Path operands
/// name a page and are checked as visited; all other operands are page-state keys
/// checked for presence. `And`/`Or` nodes combine with `&&`/`||`.
pub fn tree_to_condition(tree: &KeyExprTree) -> proc_macro2::TokenStream {
    match tree {
        KeyExprTree::Key(k) => match k {
            Expr::Path(_) => quote! {
                ifengine::core::resolve_page_id(#k)
                    .is_some_and(|__id| __ifengine_page_state.is_tagged(__id))
            },
            _ => quote! {
                __ifengine_page_state.get(#k).is_some()
            },
        },
        KeyExprTree::And(items) => {
            let sub = items.iter().map(tree_to_condition);
            quote! { ( #(#sub)&&* ) }
        }
        KeyExprTree::Or(items) => {
            let sub = items.iter().map(tree_to_condition);
            quote! { ( #(#sub)||* ) }
        }
    }
}

/// Parses an expression of keys and/or pages combined with `|`/`&` and expands it
/// into a boolean condition using the same evaluation as [`tree_to_condition`].
pub fn condition_of(expr: &Expr) -> syn::Result<proc_macro2::TokenStream> {
    Ok(tree_to_condition(&parse_keys(expr)?))
}

fn parse_key_tree(expr: &Expr) -> syn::Result<KeyExprTree> {
    let unwrapped = unwrap_paren(expr);
    match unwrapped {
        Expr::Binary(syn::ExprBinary { left: _, op, .. }) => match op {
            syn::BinOp::BitOr(_) => {
                let mut items = Vec::new();
                collect_or(unwrapped, &mut items)?;
                Ok(KeyExprTree::Or(items))
            }
            syn::BinOp::BitAnd(_) => {
                let mut items = Vec::new();
                collect_and(unwrapped, &mut items)?;
                Ok(KeyExprTree::And(items))
            }
            _ => Ok(KeyExprTree::Key(unwrapped.clone())),
        },
        _ => Ok(KeyExprTree::Key(unwrapped.clone())),
    }
}

fn collect_or(expr: &Expr, items: &mut Vec<KeyExprTree>) -> syn::Result<()> {
    if let Expr::Binary(syn::ExprBinary {
        left,
        op: syn::BinOp::BitOr(_),
        right,
        ..
    }) = expr
    {
        collect_or(left, items)?;
        collect_or(right, items)?;
    } else {
        items.push(parse_key_tree(expr)?);
    }
    Ok(())
}

fn collect_and(expr: &Expr, items: &mut Vec<KeyExprTree>) -> syn::Result<()> {
    if let Expr::Binary(syn::ExprBinary {
        left,
        op: syn::BinOp::BitAnd(_),
        right,
        ..
    }) = expr
    {
        collect_and(left, items)?;
        collect_and(right, items)?;
    } else {
        items.push(parse_key_tree(expr)?);
    }
    Ok(())
}

/// Expands an iterator of expressions into tokens evaluating to `Line`.
pub fn expand_lines(exprs: impl IntoIterator<Item = Expr>) -> Vec<proc_macro2::TokenStream> {
    exprs.into_iter().map(|e| expand_line_expr(&e)).collect()
}

/// Expands a single expression into a `Line` token stream. If it is a string literal,
/// parses any `{}` interpolations into spans and calls `Line::from_iter(...)`.
/// Otherwise, converts the expression via `Line::from(#expr)`.
pub fn expand_line_expr(expr: &Expr) -> proc_macro2::TokenStream {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(lit_str),
        ..
    }) = expr
    {
        let mut spans = Vec::new();
        expand_format_literal(lit_str, &mut spans);
        quote! {
            ifengine::view::Line::from_iter(vec![#(#spans),*])
        }
    } else {
        quote! {
            ifengine::view::Line::from(#expr)
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
                    ifengine::view::Span::from(#leading_text)
                });
            }

            let after_open = &rest[start_idx + 1..];
            if let Some(end_idx) = after_open.find('}') {
                let expr_str = after_open[..end_idx].trim();
                match syn::parse_str::<syn::Expr>(expr_str) {
                    Ok(expr) => {
                        spans.push(quote! {
                            ifengine::view::Span::from((#expr).to_string())
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
                    ifengine::view::Span::from(#rest)
                });
                break;
            }
        } else {
            // No more `{`, emit remaining text
            spans.push(quote! {
                ifengine::view::Span::from(#rest)
            });
            break;
        }
    }
}
