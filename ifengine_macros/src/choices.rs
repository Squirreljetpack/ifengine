use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Arm, Expr, Result, Token, parse_macro_input};

use crate::nodes::{KeyExpr, KeyExprs, MaybeKey};

pub struct LineArm {
    pub line: Expr,
    pub block: Option<Expr>,
}

impl Parse for LineArm {
    fn parse(input: ParseStream) -> Result<Self> {
        let line: Expr = input.parse()?;

        let block = if input.parse::<Token![=>]>().is_ok() {
            Some(input.parse()?)
        } else {
            None
        };

        Ok(LineArm { line, block })
    }
}

pub struct ChoiceInput {
    pub maybe_key: MaybeKey,
    pub arms: Vec<LineArm>,
}

impl Parse for ChoiceInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;

        let mut arms = Vec::new();
        while !input.is_empty() {
            let mut lhs_exprs = vec![input.parse::<Expr>()?];

            while input.peek(Token![|]) {
                let _ = input.parse::<Token![|]>()?;
                lhs_exprs.push(input.parse()?);
            }

            let block = if input.parse::<Token![=>]>().is_ok() {
                Some(input.parse()?)
            } else {
                None
            };

            for line in lhs_exprs {
                arms.push(LineArm {
                    line,
                    block: block.clone(),
                });
            }

            input.parse::<Token![,]>().ok();
        }

        Ok(ChoiceInput { maybe_key, arms })
    }
}

pub fn choice(input: TokenStream) -> TokenStream {
    let ChoiceInput { maybe_key, arms } = syn::parse_macro_input!(input as ChoiceInput);

    let key_tokens = maybe_key.into_tokens();

    let mut index_arms = Vec::new();
    let mut lines = Vec::new();

    for (i, LineArm { line, block }) in arms.iter().enumerate() {
        let i = i as u8;

        lines.push(quote! { (#i, ifengine::view::Line::from(#line)) });

        let block_tokens = match block {
            Some(b) => quote! { ifengine::view::Line::from({ #b }) },
            None => quote! { unreachable!() },
        };

        index_arms.push(quote! {
            #i => { #block_tokens }
        });
    }

    let expanded = quote! {
        {
            let __ifengine_key = #key_tokens;
            if let Some(__ifengine_tmp_idx) = __ifengine_page_state.get_mask_last(__ifengine_key) {
                #[allow(unreachable_code)]
                {
                    let line: ifengine::view::Line = match __ifengine_tmp_idx {
                        #(#index_arms),*,
                        _ => unreachable!(),
                    };
                    __ifengine_page_state.push(
                        ifengine::view::Object::Paragraph(line.with_id(__ifengine_key))
                    );
                }
                true
            } else {
                __ifengine_page_state.push(
                    ifengine::view::Object::Choice(
                        __ifengine_key,
                        vec![
                        #(#lines),*
                        ]
                    )
                );
                false
            }
        }
    };

    expanded.into()
}

pub fn mchoice(input: TokenStream) -> TokenStream {
    let ChoiceInput { maybe_key, arms } = syn::parse_macro_input!(input as ChoiceInput);

    let key = maybe_key.into_tokens();

    let arm_blocks: Vec<_> = arms
        .iter()
        .enumerate()
        .map(|(i, LineArm { line, block })| {
            let i = i as u8;

            let block_tokens = match block {
                Some(b) => quote! { #b },
                None => quote! {},
            };

            quote! {
                if (__ifengine_tmp_mask & (1u64 << #i)) != 0 {
                    #block_tokens
                }
                if let Some(l) = ifengine::elements::ChoiceVariant::from(#line)
                .as_line((__ifengine_tmp_mask & (1u64 << #i)) != 0)
                {
                    __ifengine_tmp_lines.push((#i, l));
                    __ifengine_visible_mask[#i as usize] = false;
                }
            }
        })
        .collect::<Vec<_>>();

    let n = arms.len();

    let expanded = quote! {
        {
            let __ifengine_key = #key;
            let __ifengine_tmp_mask = __ifengine_page_state.get(__ifengine_key).unwrap_or(0u64);
            let mut __ifengine_tmp_lines = Vec::new();
            let mut __ifengine_visible_mask = [true; #n];

            #(#arm_blocks)*

            if ! __ifengine_tmp_lines.is_empty() {
                __ifengine_page_state.push(
                    ifengine::view::Object::Choice(__ifengine_key, __ifengine_tmp_lines)
                );
            }

            __ifengine_visible_mask
        }
    };

    expanded.into()
}

pub fn dynamic_choice(input: TokenStream) -> TokenStream {
    let KeyExpr { maybe_key, expr } = syn::parse_macro_input!(input as KeyExpr);
    let key_tokens = maybe_key.into_tokens();

    let expanded = quote! {
        {
            let __ifengine_key = #key_tokens;
            // Push the DynamicChoice object
            __ifengine_page_state.push(ifengine::view::Object::Choice(
                __ifengine_key,
                #expr
                .into_iter()
                .map(|(t, l)| (t as u8, ifengine::view::Line::from(l)))
                .collect()
            ));

            __ifengine_page_state.remove_mask_last(__ifengine_key).map(|x|
                unsafe { std::mem::transmute::<u8, _>(x) }
            )
        }
    };

    expanded.into()
}

pub struct DChoicesInput {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
    pub arms: Vec<Arm>,
}

impl Parse for DChoicesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let KeyExpr { maybe_key, expr } = input.parse()?;

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        let mut arms = Vec::new();
        while !input.is_empty() {
            arms.push(input.parse::<Arm>()?);
        }

        Ok(DChoicesInput {
            maybe_key,
            expr,
            arms,
        })
    }
}

pub fn dchoice(input: TokenStream) -> TokenStream {
    let DChoicesInput {
        maybe_key,
        expr,
        arms,
    } = parse_macro_input!(input as DChoicesInput);

    let key_tokens = maybe_key.into_tokens();

    let has_wildcard = arms.iter().any(|arm| matches!(arm.pat, syn::Pat::Wild(_)));
    let catch_all = if has_wildcard {
        quote! {}
    } else {
        quote! { _ => {} }
    };
    let match_block = if arms.is_empty() {
        quote! {}
    } else {
        quote! {
            if let Some(__ifengine_id) = __ifengine_page_state.remove_mask_last(__ifengine_key) {
                match __ifengine_id as usize {
                    #(#arms)*
                    #catch_all
                }
            }
        }
    };

    let expanded = quote! {
        {
            let __ifengine_key = #key_tokens;
            __ifengine_page_state.push(ifengine::view::Object::Choice(
                __ifengine_key,
                #expr
                .iter()
                .enumerate()
                .map(|(i, l)| (i as u8, ifengine::view::Line::from(l.clone())))
                .collect()
            ));

            #match_block
        }
    };

    expanded.into()
}

pub fn dparagraph(input: TokenStream) -> TokenStream {
    let KeyExprs { maybe_key, exprs } = syn::parse_macro_input!(input as KeyExprs);

    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let __ifengine_key = #key;
        let mut ret = None;

        #(
            let mut __ifengine_tmp_strings =
            ifengine::utils::split_braced(&ifengine::utils::trim_lines(&#exprs));

            if let Some(__ifengine_tmp_val) = __ifengine_page_state
            .remove(__ifengine_key)
            .and_then(|k| {
                ifengine::utils::find_hash_match(__ifengine_tmp_strings.iter().step_by(2), k).cloned()
            }) {
                ret = Some(__ifengine_tmp_val);
            }

            __ifengine_page_state.push(
                ifengine::view::Object::Paragraph(
                    ifengine::view::Line::from_interleaved_actions::<false>(
                        (__ifengine_page_state.id(), __ifengine_key),
                        __ifengine_tmp_strings
                    )
                )
            );
        )*

        ret
    }};

    expanded.into()
}

pub fn mparagraph(input: TokenStream) -> TokenStream {
    let KeyExpr { maybe_key, expr } = syn::parse_macro_input!(input as KeyExpr);

    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let __ifengine_key = #key;
        let strings =
        ifengine::utils::split_braced(&ifengine::utils::trim_lines(&#expr));
        let count = strings.len() / 2;

        __ifengine_page_state.push(
            ifengine::view::Object::Paragraph(
                ifengine::view::Line::from_interleaved_actions::<true>(
                    (__ifengine_page_state.id(), __ifengine_key),
                    strings
                )
            )
        );

        __ifengine_page_state.get_mask::<64>(__ifengine_key)[..count].to_vec()
    }};

    expanded.into()
}

pub struct ReplaceInput {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
    pub block: Option<Expr>,
}

impl Parse for ReplaceInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let expr: Expr = input.parse()?;
        let block = if input.parse::<Token![=>]>().is_ok() {
            Some(input.parse()?)
        } else if input.parse::<Token![,]>().is_ok() && !input.is_empty() {
            Some(input.parse()?)
        } else {
            None
        };
        Ok(ReplaceInput {
            maybe_key,
            expr,
            block,
        })
    }
}

pub fn replace(input: TokenStream) -> TokenStream {
    let ReplaceInput {
        maybe_key,
        expr,
        block,
    } = syn::parse_macro_input!(input as ReplaceInput);

    let key = maybe_key.into_tokens();

    let block_token = match block {
        Some(b) => quote! { #b },
        None => quote! { () },
    };

    let expanded = quote! {{
        let __ifengine_key = #key;

        if __ifengine_page_state.get(__ifengine_key).is_some() {
            let __ifengine_replacement: ifengine::view::Line = ifengine::view::Line::from(#block_token);
            if !__ifengine_replacement.spans.is_empty() {
                __ifengine_page_state.push(
                    ifengine::view::Object::Paragraph(__ifengine_replacement.with_id(__ifengine_key))
                );
            }
        } else {
            let __ifengine_raw_str = #expr;
            let __ifengine_trimmed = ifengine::utils::trim_lines(&__ifengine_raw_str);
            let mut __ifengine_spans = Vec::new();

            if let Some((__before, __inside, __after)) = ifengine::utils::split_first_braced(&__ifengine_trimmed) {
                if !__before.is_empty() {
                    __ifengine_spans.push(ifengine::view::Span::from_lingual(__before));
                }
                __ifengine_spans.push(
                    ifengine::view::Span::from_lingual(__inside)
                        .as_link()
                        .with_action(ifengine::Action::Set(
                            (__ifengine_page_state.id(), __ifengine_key),
                            1
                        ))
                );
                if !__after.is_empty() {
                    __ifengine_spans.push(ifengine::view::Span::from_lingual(__after));
                }
            } else {
                __ifengine_spans.push(
                    ifengine::view::Span::from_lingual(&__ifengine_trimmed)
                        .as_link()
                        .with_action(ifengine::Action::Set(
                            (__ifengine_page_state.id(), __ifengine_key),
                            1
                        ))
                );
            }

            let __ifengine_line = ifengine::view::Line::from_spans(__ifengine_spans).with_id(__ifengine_key);
            __ifengine_page_state.push(
                ifengine::view::Object::Paragraph(__ifengine_line)
            );
        }
    }};

    expanded.into()
}
