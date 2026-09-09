use proc_macro::TokenStream;
use quote::quote;
use syn::visit_mut::VisitMut;
use syn::{Error, ItemFn, parse_macro_input};

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
                if let Ok(embed_input) =
                    syn::parse2::<crate::elements::EmbedInput>(mac.tokens.clone())
                {
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
            let mut __ifengine_page_state = ifengine::core::PageState::new(
                __ifengine_game.page_id(format!("{}::{}", module_path!(), stringify!(#name))),
                __ifengine_game.fresh(),
                __ifengine_simulating,
                __ifengine_game.state.get_page_mut(format!("{}::{}", module_path!(), stringify!(#name))),
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
