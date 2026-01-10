//! See [`ifengine::elements`]
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Arm, Block, Error, Expr, ExprClosure, ItemFn, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};
mod nodes;
use nodes::*;

/// Decorate your function with this.
///
/// The function must take your game state as a parameter, and return `()`.
/// This macro will rewrite your function to receive a &mut [`ifengine::Game`] and return a [`ifengine::core::Response`], as well as enabling usage of [`ifengine::elements`] to produce that response (which in most cases will be a [`ifengine::View`]).
///
/// # Examples
///```rust
/// #[ifview]
/// pub fn p1(s: &mut State) {
///     h!("SALTWRACK", 3); // heading level 3
///     p!(link!("BEGIN", p2)); // Link to the next page
/// }
///
/// // ----- mod.rs -----
/// pub type Game = ifengine::Game<State>;
/// pub fn new() -> Game {
///    ifengine::Game!(chap1::p1)
/// }
///```
#[proc_macro_attribute]
pub fn ifview(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let name = &input.sig.ident;
    let original_block = &input.block;

    if input.sig.inputs.len() != 1 {
        return Error::new_spanned(
            &input.sig.inputs,
            "ifview functions must have exactly one input: the context type C",
        )
        .to_compile_error()
        .into();
    }

    let ctx_arg = input.sig.inputs.first().unwrap();
    let ctx_type = if let syn::FnArg::Typed(pat_type) = ctx_arg
        && let syn::Type::Reference(ty_ref) = &*pat_type.ty
        && ty_ref.mutability.is_some()
    {
        &*ty_ref.elem
    } else {
        return Error::new_spanned(ctx_arg, "Expected a &mut C type")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        pub fn #name(__ifengine_game: &mut ifengine::Game<#ctx_type>)
        -> ifengine::core::Response
        {
            let __ifengine_simulating = __ifengine_game.simulating();
            #[allow(unused_variables)]
            let #ctx_arg = &mut __ifengine_game.context;
            let __ifengine_game_tags = &mut __ifengine_game.tags;
            let __ifengine_game = &mut __ifengine_game.inner;
            let mut __ifengine_page_state = ifengine::core::PageState::new(
                format!("{}::{}", module_path!(), stringify!(#name)),
                __ifengine_game.state.get_page_mut(format!("{}::{}", module_path!(), stringify!(#name))),
                __ifengine_game_tags,
                __ifengine_game.fresh,
                __ifengine_simulating
            );

            #original_block

            #[allow(unreachable_code)]
            __ifengine_page_state.into_response()
        }
    };

    expanded.into()
}

// ----------- CHOICES -------------------------

// Expr instead of Pattern
struct LineArm {
    line: Expr,
    block: Option<Expr>,
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
struct ChoiceInput {
    maybe_key: MaybeKey,
    arms: Vec<LineArm>,
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

/// Conditionally display one of several choices based on user selection.
///
/// Returns true if it has resolved, otherwise false.
///
/// # Description
/// The `choice!` macro takes a list of arms in the form `LHS => RHS`, where both
/// sides implement `Into<`[`Line`](ifengine::view::Line)`>`. It works as follows:
///
/// - If no arm is selected, the LHS values are displayed as a list of lines.
/// - Once a choice is selcted, subsequent renders execute the corresponding RHS expression and
///   display its result.
///
/// # Additional
/// A [`MaybeKey`] can be specified as the first argument
///   When a choice is clicked, it sets the value of its key to (the u8 value of) its id in [`PageState`].
///   It is discouraged to specify this: by default, it will be automatically generated.
/// Multiple LHS values can be specified for the same RHS using `|`
///
/// # Example
/// ```rust
/// choice! {
///     "1" => "Chose 1",
///     "2" | "3" => {
///         "Chose 2 or 3"
///     },
/// };
/// ```
#[proc_macro]
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
        if let Some(__ifengine_tmp_idx) = __ifengine_page_state.get_mask_last(#key_tokens) {
            #[allow(unreachable_code)]
            __ifengine_page_state.push(
                ifengine::view::Object::Paragraph(
                    match __ifengine_tmp_idx {
                        #(#index_arms),*,
                        _ => unreachable!(),
                    }
                )
            );
            true
        } else {
            __ifengine_page_state.push(
                ifengine::view::Object::Choice(
                    #key_tokens,
                    vec![
                    #(#lines),*
                    ]
                )
            );
            false
        }
    };

    expanded.into()
}

/// Execute a set of conditional expressions based on user-selected choices.
///
/// Each arm has the form `Choice => Expr`. If a choice was selected, its
/// corresponding expression (the RHS) is executed (executions occur in order), regardless of whether
/// the choice's key (the LHS) is currently visible.
///
/// Each LHS key is a [`ifengine::elements::ChoiceVariant`], dictating its visibility.
/// Any type that implements `Into<`[`Line`](ifengine::view::Line)`>` will coerce to `Choice::Once`.
/// Any `Option<Into<Line>>` will coerce to `Choice::None` or `Choice::Always`.
///
/// The return type is a [bool ;n] representing which of the options were hidden (NOT displayed).

#[proc_macro]
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
            let __ifengine_tmp_mask = __ifengine_page_state.get(#key).unwrap_or(0u64);
            let mut __ifengine_tmp_lines = Vec::new();
            let mut __ifengine_visible_mask = [true; #n];

            #(#arm_blocks)*

            if ! __ifengine_tmp_lines.is_empty() {
                __ifengine_page_state.push(
                    ifengine::view::Object::Choice(#key, __ifengine_tmp_lines)
                );
            }

            __ifengine_visible_mask
        }
    };

    expanded.into()
}

/// Executes code for a set of selectable choices. Prefer to use [`dchoice`] for brevity.
///
/// # Overview
/// This macro displays list of choices, and registers a corresponding handler
/// for each selection. The handler is specified as a `match` expression, where
/// each arm corresponds to a choice and contains the code to execute when
/// that choice is selected. Unlike the other choice elements ([`choice`], [`choices`]),
/// the conditional expression is evaluated only the first time it's choice is selected.
/// The intent is that the arms are used to set values for the user's custom [`ifengine::core::GameContext`].
///
/// # Arguments
/// - [`MaybeKey`] (Optional)
/// - **Choices list**: A `Vec<(Id, Line)>` representing the selectable options. The Id can either be a [#repr(u8)] Unit Enum or a pure u8.
/// - **Handler**: A `match` statement handling each choice.
///
/// # Match statement
/// The match token of the match statement should be given with your custom enum type, or not given if you identify your choices with pure u8's.
///
/// # Additional
/// A [`MaybeKey`] can be specified in the first argument:
///   When a choice is clicked, it sets the value of its key to its id (cast as a u8) in [`PageState`].
///   When the page is next rendered, this value is removed, and the corresponding match arm is run.
///   It is discouraged to specify this: by default, it will be automatically generated.
///
/// # Example
/// ```rust
/// #[derive(Clone, Copy)]
/// enum DChoices { A, B, C }
///
/// let choices = vec![
///     (DChoices::A, line!("A")),
///     (DChoices::B, line!("B")),
///     (DChoices::C, line!("C")),
/// ];
///
/// if let Some(x) = dynamic_choice!(choices) {
///     match x {
///         DChoices::A => "A clicked",
///         DChoices::B => "B clicked",
///         DChoices::C => "C clicked",
///     }
/// }
/// ```

#[proc_macro]
pub fn dynamic_choice(input: TokenStream) -> TokenStream {
    let KeyExpr { maybe_key, expr } = syn::parse_macro_input!(input as KeyExpr);
    let key_tokens = maybe_key.into_tokens();

    let expanded = quote! {
        {
            // Push the DynamicChoice object
            __ifengine_page_state.push(ifengine::view::Object::Choice(
                #key_tokens,
                #expr
                .into_iter()
                .map(|(t, l)| (t as u8, ifengine::view::Line::from(l)))
                .collect()
            ));

            __ifengine_page_state.remove_mask_last(#key_tokens).map(|x|
                unsafe { std::mem::transmute::<u8, _>(x) }
            )
        }
    };

    expanded.into()
}

struct DChoicesInput {
    pub maybe_key: MaybeKey,
    pub expr: Expr,
    pub arms: Vec<Arm>,
}

impl Parse for DChoicesInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let KeyExpr { maybe_key, expr } = input.parse()?;
        input.parse::<Token![,]>()?;

        let mut arms = Vec::new();
        while !input.is_empty() {
            let arm: syn::Arm = input.parse()?;
            arms.push(arm);
        }

        Ok(DChoicesInput {
            maybe_key,
            expr,
            arms: arms.into_iter().collect(),
        })
    }
}

/// A version of [`dynamic_choice`] with slightly abbreviated syntax.
///
/// # Example
/// ```rust
/// #[derive(Clone, Copy)]
/// enum DChoices { A, B, C }
///
/// let choices = vec![
///     (DChoices::A, line!("A")),
///     (DChoices::B, line!("B")),
///     (DChoices::C, line!("C")),
/// ];
/// dchoice!{ choices,
///     DChoices::A => "A clicked",
///     DChoices::B => "B clicked",
///     DChoices::C => "C clicked",
/// }
/// ```
#[proc_macro]
pub fn dchoice(input: TokenStream) -> TokenStream {
    let DChoicesInput {
        maybe_key,
        expr,
        arms,
    } = parse_macro_input!(input as DChoicesInput);

    let key_tokens = maybe_key.into_tokens();

    let match_tokens = quote! {
        match unsafe { std::mem::transmute::<u8, _>(__ifengine_chosen_discriminant) } {
            #(#arms),*
        }
    };

    let expanded = quote! {
        // Push the DynamicChoice object
        __ifengine_page_state.push(ifengine::view::Object::Choice(
            #key_tokens,
            #expr
            .into_iter()
            .map(|(t, l)| (t as u8, ifengine::view::Line::from(l)))
            .collect()
        ));

        // Execute user block if a choice was selected
        if let Some(__ifengine_chosen_discriminant) = __ifengine_page_state.remove_mask_last(#key_tokens) {
            #match_tokens
        }
    };

    expanded.into()
}
/// Create a paragraph with interactive elements from a string.
///
/// Interactive text sections are automatically added from text delimited by [[ and ]] (Also see: [`mparagraph`]).
/// The return type is the value of whichever text token that was clicked.
///
/// # Syntax
/// ```text
/// dparagraph!(maybe_key, expr1, expr2, ..., exprN)
///
/// # Additional
/// Text is trimmed
/// Multiple inputs are accepted, and produce multiple paragraphs
/// The interactive elements must not change between renders.
#[proc_macro]
pub fn dparagraph(input: TokenStream) -> TokenStream {
    let KeyExprs { maybe_key, exprs } = syn::parse_macro_input!(input as KeyExprs);

    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let mut ret = None;

        #(
            let mut __ifengine_tmp_strings =
            ifengine::utils::split_braced(&ifengine::utils::trim_lines(&#exprs));

            if let Some(__ifengine_tmp_val) = __ifengine_page_state
            .remove(#key)
            .and_then(|k| {
                ifengine::utils::find_hash_match(__ifengine_tmp_strings.iter().step_by(2), k).cloned()
            }) {
                ret = Some(__ifengine_tmp_val);
            }

            __ifengine_page_state.push(
                ifengine::view::Object::Paragraph(
                    ifengine::view::Line::from_interleaved_actions::<false>(
                        (__ifengine_page_state.id(), #key),
                        __ifengine_tmp_strings
                    )
                )
            );
        )*

        ret
    }};

    expanded.into()
}

/// Create a paragraph with interactive elements from a string.
///
/// Interactive text sections are automatically added from text delimited by [[ and ]] (Also see: [`dparagraph`]).
/// This macro tracks and returns which of the interactive elements had been clicked since page load as a `Vec<bool>`.
///
/// # Note
/// The interactive elements must not change between renders.

#[proc_macro]
pub fn mparagraph(input: TokenStream) -> TokenStream {
    let KeyExpr { maybe_key, expr } = syn::parse_macro_input!(input as KeyExpr);

    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        let strings =
        ifengine::utils::split_braced(&ifengine::utils::trim_lines(&#expr));
        let count = strings.len() / 2;

        __ifengine_page_state.push(
            ifengine::view::Object::Paragraph(
                ifengine::view::Line::from_interleaved_actions::<true>(
                    (__ifengine_page_state.id(), #key),
                    strings
                )
            )
        );

        __ifengine_page_state.get_mask::<64>(#key)[..count].to_vec()
    }};

    expanded.into()
}

// ----------------- ELEMENTS -------------------

/// Push a (Object)[ifengine::view::Object] to the current (View)[ifengine::View]
#[proc_macro]
pub fn push(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);

    let expanded = quote! {
        __ifengine_page_state.push(
            #expr
        );
    };

    expanded.into()
}

struct LineArgs {
    exprs: Vec<Expr>,
    trailer: Option<LitStr>,
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

/// Pure text element, constructed from a sequence of spans.
///
/// # Additional
/// A trailing [`ifengine::view::RenderData`] can be specified following `::`.
///
/// # Example
/// ```rust
/// text!("Hello, world!");
/// text!("Hello, ", "world!" :: "my_render_data");
/// ```
#[proc_macro]
pub fn text(input: TokenStream) -> TokenStream {
    let LineArgs { exprs, trailer } = syn::parse_macro_input!(input as LineArgs);

    let string_expr = match trailer {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::Object::Text(
                ifengine::view::Line::from_spans(
                    vec![#(#exprs.into()),*]
                ),
                #string_expr
            )
        );
    };

    TokenStream::from(expanded)
}

/// A sequence of text elements. See [`text`].
///
/// # Additional
/// A trailing [`ifengine::view::RenderData`] can be specified following `::`.
///
/// # Example
/// ```rust
/// texts!("Line 1", "Line 2");
/// ```
#[proc_macro]
pub fn texts(input: TokenStream) -> TokenStream {
    let LineArgs { exprs, trailer } = syn::parse_macro_input!(input as LineArgs);

    let string_expr = match trailer {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let expanded = quote! {
        #(
            __ifengine_page_state.push(
                ifengine::view::Object::Text(
                    ifengine::view::Line::from(#exprs),
                    #string_expr
                )
            );
        )*
    };

    TokenStream::from(expanded)
}

/// Create a paragraph from a sequence of spans.
///
/// # Example
/// ```rust
/// paragraph!(span1, span2, span3);
/// ```
#[proc_macro]
pub fn paragraph(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs_parsed.into_iter().collect();

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::Object::Paragraph(
                ifengine::view::Line::from_spans(vec![#(ifengine::view::Span::from_lingual(#exprs)),*])
            )
        );
    };

    TokenStream::from(expanded)
}

/// Shorthand for creating multiple paragraphs from a sequence of [`crate::view::Line`]'s.
///
/// # Example
/// ```rust
/// paragraphs!(line1, line2, line3);
/// ```
///
/// # Additional
/// Any type implementing `Into<Line>` is accepted.
#[proc_macro]
pub fn paragraphs(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::punctuated::Punctuated;
    use syn::{Expr, Token, parse_macro_input};

    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<Expr> = exprs_parsed.into_iter().collect();

    let expanded = quote! {
        #(
            __ifengine_page_state.push(
                ifengine::view::Object::Paragraph(
                    ifengine::view::Line::from_lingual(#exprs)
                )
            );
        )*
    };

    TokenStream::from(expanded)
}

/// Push an image from a string literal.
///
/// # Example
/// ```rust
/// img!("assets/logo.png");
/// img!("https://example.com/logo.png", (100, 50));
/// ```
#[proc_macro]
pub fn img(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::punctuated::Punctuated;
    use syn::{Expr, Lit, Token, parse_macro_input};

    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<&Expr> = exprs_parsed.iter().collect();

    let (path_expr, size_expr) = match exprs.len() {
        1 => (exprs[0], None),
        2 => (exprs[0], Some(exprs[1])),
        _ => {
            return syn::Error::new_spanned(exprs_parsed, "image! macro expects 1 or 2 arguments")
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
        return syn::Error::new_spanned(path_expr, "expected string literal")
            .to_compile_error()
            .into();
    };

    let expanded = quote! {
        __ifengine_page_state.push(ifengine::view::Object::Image(#image_tokens));
    };

    TokenStream::from(expanded)
}

/// Markdown heading.
///
/// # Example
/// ```rust
/// h!("Title", 2)
/// ```
#[proc_macro]
pub fn h(input: TokenStream) -> TokenStream {
    let exprs_parsed = parse_macro_input!(input with Punctuated<Expr, Token![,]>::parse_terminated);
    let exprs: Vec<&Expr> = exprs_parsed.iter().collect();

    if exprs.len() != 2 {
        return syn::Error::new_spanned(
            exprs_parsed,
            "macro expects exactly 2 arguments: text and level",
        )
        .to_compile_error()
        .into();
    }

    let text = exprs[0];
    let level = exprs[1];

    let expanded = quote! {
        __ifengine_page_state.push(
            ifengine::view::Object::Heading(ifengine::view::Span::from_lingual(#text), #level)
        );
    };

    TokenStream::from(expanded)
}

/// Horizontal rule (`<hr/>`).
///
/// # Example
/// ```rust
/// hr!()
/// ```
#[proc_macro]
pub fn hr(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.push(ifengine::view::Object::Break);
    };

    TokenStream::from(expanded)
}

// --------------- ALTS -------------------------

#[derive(Clone)]
enum AltVariant {
    Stop,
    Shuffle,
    Cycle,
}

impl Default for AltVariant {
    fn default() -> Self {
        AltVariant::Stop
    }
}

impl Parse for AltVariant {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "Stop" => Ok(AltVariant::Stop),
            "Shuffle" => Ok(AltVariant::Shuffle),
            "Cycle" => Ok(AltVariant::Cycle),
            _ => Err(syn::Error::new(
                ident.span(),
                "expected AltVariant: Stop | Shuffle | Cycle",
            )),
        }
    }
}

struct AltsInput {
    maybe_key: MaybeKey,
    list: Vec<Expr>,
    variant: Option<AltVariant>,
}

impl Parse for AltsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
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

        // optional variant
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

/// Cycle between multiple alternative spans on click.
///
/// ## Behavior
///
/// - The first span is shown when no prior state exists.
/// - The active index is stored in page state under the provided key.
///
/// ## Variants
///
/// ### `stop` (default)
/// Advances until the last span, then stops:
///
/// ```text
/// A → B → C (stops)
/// ```
///
/// ### `cycle`
/// Advances to the next span on activation, wrapping around:
///
/// ```text
/// A → B → C → A → …
/// ```
///
/// ### `shuffle`
/// Chooses a random span each time, avoiding immediate repetition.
/// The internal state uses the low bit as a regeneration flag.
///
/// ## Syntax
///
/// ```ignore
/// alts!(key?, variant?, [expr, expr, ...])
/// ```
///
/// - `key` (optional): Explicit state key
/// - `variant` (optional): `cycle`, `stop`, or `shuffle`
/// - `expr`: Any value convertible into a `Span`
///
/// ## Examples
///
/// Basic cycling:
///
/// ```ignore
/// alts!([
///     "Look around",
///     "Open the door",
///     "Wait",
/// ])
/// ```
///
/// With an explicit key and variant:
///
/// ```ignore
/// alts!(
///     (5),
///     shuffle,
///     [
///         "Attack",
///         "Defend",
///         "Flee",
///     ]
/// )
/// ```
///
/// ## Notes
///
/// - State is updated via [`ifengine::Action::Inc`] or [`ifengine::Action::Set`].
/// - Random selection uses the page state's RNG.
/// - The macro expands to an expression producing a `Span`.
/// - Shuffle and Cycle are hidden during simulation.
#[proc_macro]
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
                let alts = #list_init;

                if let Some(idx) = __ifengine_page_state.get(#key) {
                    ifengine::view::Span::from(
                        alts[(idx as usize + 1).min(alts.len() - 1)]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
                }
            }}
        }

        AltVariant::Shuffle => {
            quote! {{
                let alts = #list_init;

                // Determine tmp index
                let idx = if let Some(prev) = __ifengine_page_state.get(#key) {
                    if prev & 1 == 0 {
                        (prev as usize) >> 1
                    } else {
                        // regenerate, excluding previous index
                        let new_idx = __ifengine_page_state.rand(alts.len(), &[(prev as usize) >> 1]);
                        __ifengine_page_state.insert(#key, (new_idx as u64) << 1);
                        new_idx
                    }
                } else {
                    let new_idx = __ifengine_page_state.rand(alts.len(), &[]);
                    __ifengine_page_state.insert(#key, (new_idx as u64) << 1);
                    new_idx
                } ;

                // Use it and store back with last bit set
                ifengine::view::Span::from(alts[idx])
                .with_action(ifengine::Action::Set(
                    (__ifengine_page_state.id(), #key),
                    ((idx as u64) << 1) + 1
                ))
                .no_sim()
            }}
        }

        AltVariant::Cycle => {
            quote! {{
                let alts = #list_init;

                if let Some(idx) = __ifengine_page_state.get(#key) {
                    ifengine::view::Span::from(
                        alts[(idx as usize) % alts.len()]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
                    .no_sim()
                } else {
                    ifengine::view::Span::from(
                        alts[0]
                    )
                    .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
                    .no_sim()
                }
            }}
        }
    };

    expanded.into()
}
//

// ------------- SPANS/CLOSURES ----------------------

struct CountInput {
    maybe_key: MaybeKey,
    closure: ExprClosure,
}

impl Parse for CountInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let maybe_key = input.parse()?;
        let closure = input.parse()?;

        Ok(CountInput { maybe_key, closure })
    }
}

/// Use a closure to compute a span based on how many times the span has been clicked.
///
/// # Syntax
/// ```rust
/// let span_count = read_key!(6); // Can be called before
/// let span = count!((6), |val| "span");
/// ```
///
/// # Arguments
/// - [`MaybeKey`]
/// - `closure`: A closure taking the current value and returning a `Span`.
#[proc_macro]
pub fn count(input: TokenStream) -> TokenStream {
    let CountInput { maybe_key, closure } = syn::parse_macro_input!(input as CountInput);
    let key = maybe_key.into_tokens();

    let expanded = quote! {{
        ifengine::view::Span::from(
            (#closure)(__ifengine_page_state.get(#key).unwrap_or_default())
        )
        .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
        .no_sim()
    }};

    expanded.into()
}

struct ClickInput {
    maybe_key: MaybeKey,
    expr: Expr,
    block: Block,
}

impl Parse for ClickInput {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse comma-separated expressions first
        let exprs: Punctuated<Expr, Token![,]> = Punctuated::parse_terminated(input)?;
        let mut iter = exprs.into_iter();

        let (maybe_key, expr, block) = match (iter.next(), iter.next(), iter.next()) {
            (Some(key), Some(expr), Some(Expr::Block(block))) => {
                (MaybeKey::Key(key), expr, block.block)
            }
            (Some(expr), Some(Expr::Block(block)), None) => (MaybeKey::Auto, expr, block.block),
            _ => return Err(input.error("expected at least an expression and a block")),
        };

        Ok(ClickInput {
            maybe_key,
            expr,
            block,
        })
    }
}

/// Run code on click.
///
/// # Syntax
/// ```rust
/// p!(click!(span, { block } ))
/// ```
///
/// # Arguments
/// - [`MaybeKey`]
/// - `span`: The element to display. The link style is automatically applied.
/// - `block`: Executed exactly once whenever the key is clicked.
#[proc_macro]
pub fn click(input: TokenStream) -> TokenStream {
    let ClickInput {
        maybe_key,
        expr,
        block,
    } = syn::parse_macro_input!(input as ClickInput);
    let key = maybe_key.into_tokens();
    let stmts = &block.stmts;

    let expanded = quote! {{
        if __ifengine_page_state.remove(#key).is_some() {
            #(#stmts)*
            #[allow(unreachable_code)]
            ifengine::view::Span::from(
                #expr
            )
            .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
            .as_link()
            .no_sim()
        } else {
            ifengine::view::Span::from(
                #expr
            )
            .with_action(ifengine::Action::Inc((__ifengine_page_state.id(), #key)))
            .as_link()
        }
    }};

    expanded.into()
}

/// Run a function only once when the page is first loaded.
///
/// # Syntax
/// ```rust
/// fresh!(|| { /* code */ })
/// ```
#[proc_macro]
pub fn fresh(input: TokenStream) -> TokenStream {
    let closure = parse_macro_input!(input as ExprClosure);

    let expanded = quote! {{
        if __ifengine_page_state.fresh() {
            (#closure)();
        }
    }};

    expanded.into()
}

// -------------- SPANS -------------------------

/// Create a link [`Span`] that navigates backward.
///
/// - `$e`: Display text.
/// - `$n`: Optional number of steps to go back (defaults to 1).
///
/// # Additional
/// This option will be hidden during simulation if no number is specified
#[proc_macro]
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

/// Immediately yield a [`Response::View`] with the current [`View`].
///
/// This returns `!`, exiting the current function.
#[proc_macro]
#[allow(non_snake_case)]
pub fn r#YIELD(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        return __ifengine_page_state.into_response()
    };
    expanded.into()
}

// ------------ KEY OPERATIONS -------------------
/// Read the value of a key of the internal [`PageState`]
///
/// Elements push to the view in the order they are called.
/// This can be used to query their state out of order.
///
/// Beware that the implementation details of the internal page state that these keys index is internal and should not be relied on!
///
/// # Example
/// ```rust
/// let value = read_key!(my_key);
/// ```
#[proc_macro]
pub fn read_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.get(#expr)
    };

    expanded.into()
}

/// Read a key as a bitmask. See [`read_key`].
///
/// # Example
/// ```rust
/// let mask = read_key_mask!(my_key); // [bool; 64]
/// let mask = read_key_mask!(my_key, 5); // [bool; 5]
/// ```
#[proc_macro]
pub fn read_key_mask(input: TokenStream) -> TokenStream {
    let ExprAndOptional { expr: key, n } = syn::parse_macro_input!(input as ExprAndOptional);

    let n = n.unwrap_or_else(|| syn::parse_quote!(64));

    quote! {
        __ifengine_page_state.get_mask::<#n>(#key)
    }
    .into()
}

/// Set a key to a value. See [`read_key`].
///
/// # Example
/// ```rust
/// set_key!(my_key, 42);
/// ```
#[proc_macro]
pub fn set_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.insert(#expr.0, #expr.1)
    };

    expanded.into()
}

/// Set individual bits of a key to true. See [`read_key_mask`].
///
/// # Example
/// ```rust
/// set_key_mask!(my_key, 0, 2, 4);
/// ```
#[proc_macro]
pub fn set_key_mask(input: TokenStream) -> TokenStream {
    use syn::{Expr, Token, parse::Parser, punctuated::Punctuated};

    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut iter = parts.iter();
    let key = if let Some(key) = iter.next() {
        key
    } else {
        return syn::Error::new_spanned(parts, "expected key")
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
                    return syn::Error::new_spanned(i, "failed to parse bit position")
                        .to_compile_error()
                        .into();
                }
            }
        } else {
            return syn::Error::new_spanned(expr, "bit positions must be integer literals")
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

/// Clear individual bits of a key. See [`read_key_mask`].
///
/// # Example
/// ```rust
/// unset_key_mask!(my_key, 1, 3);
/// ```
#[proc_macro]
pub fn unset_key_mask(input: TokenStream) -> TokenStream {
    use syn::{Expr, Token, parse::Parser, punctuated::Punctuated};

    let parts = match Punctuated::<Expr, Token![,]>::parse_terminated.parse(input) {
        Ok(parts) => parts,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut iter = parts.iter();
    let key = if let Some(key) = iter.next() {
        key
    } else {
        return syn::Error::new_spanned(parts, "expected key")
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
                    return syn::Error::new_spanned(i, "failed to parse bit position")
                        .to_compile_error()
                        .into();
                }
            }
        } else {
            return syn::Error::new_spanned(expr, "bit positions must be integer literals")
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

/// Increment the value of a key. See [`read_key`].
///
/// # Example
/// ```rust
/// inc_key!(my_key);
/// ```
#[proc_macro]
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

/// Reset (remove) a key from state. See [`read_key`].
///
/// # Example
/// ```rust
/// reset_key!(my_key);
/// ```
#[proc_macro]
pub fn reset_key(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.remove(#expr)
    };

    expanded.into()
}

// ------------ TAGS ------------------

// note: this doesn't work
/// [Tags](crate::core::GameTags) the current page.
///
/// Pass `Sticky` to persist the tag between pages.
///
/// # Examples
///
/// ```rust
/// tag!(my_value);          // non-sticky tag
/// tag!(my_value, Sticky);  // sticky tag
/// tag!(my_value, Once);    // apply only once
/// ```
#[proc_macro]
pub fn tag(input: TokenStream) -> TokenStream {
    use quote::quote;
    use syn::parse::{Parse, ParseStream, Result};
    use syn::{Expr, Ident, Token, parse_macro_input};

    struct TagInput {
        expr: Expr,
        mode: Option<Ident>,
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

    let TagInput { expr, mode } = parse_macro_input!(input as TagInput);

    let sticky = match mode {
        Some(id) => match id.to_string().as_str() {
            "Sticky" => true,
            "Once" => false,
            _ => {
                return syn::Error::new_spanned(&id, "Expected `Sticky` or `Once`")
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

/// Removes a [tag](`crate::core::GameTags`)
#[proc_macro]
pub fn untag(input: TokenStream) -> TokenStream {
    let expr = syn::parse_macro_input!(input as syn::Expr);

    let expanded = quote! {
        __ifengine_page_state.untag(#expr)
    };

    expanded.into()
}

/// Returns whether the current function is running in a [`crate::run::Simulation`].
#[proc_macro]
pub fn in_sim(_: TokenStream) -> TokenStream {
    let expanded = quote! {
        __ifengine_page_state.simulating
    };

    expanded.into()
}

// ------------ UTILS ------------------

/// Debug display the current [`PageState`]
#[proc_macro]
pub fn page_dbg(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        // #[cfg(debug_assertions)]
        dbg!(&__ifengine_page_state)
    };
    expanded.into()
}

/// Debug display the current view
#[proc_macro]
pub fn view_dbg(_input: TokenStream) -> TokenStream {
    let expanded = quote! {
        dbg!(&__ifengine_page_state.view)
    };
    expanded.into()
}
