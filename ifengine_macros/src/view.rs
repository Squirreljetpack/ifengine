use proc_macro::TokenStream;
use quote::quote;
use syn::{Error, ItemFn, parse_macro_input};

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
                __ifengine_game.fresh(),
                __ifengine_simulating,
                __ifengine_game.state.get_page_mut(format!("{}::{}", module_path!(), stringify!(#name))),
                __ifengine_game_tags,
            );

            #original_block

            #[allow(unreachable_code)]
            __ifengine_page_state.into_response()
        }
    };

    expanded.into()
}
