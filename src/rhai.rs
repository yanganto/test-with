use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::ItemFn;

pub(crate) fn rhai_fn_macro(attr: TokenStream, input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;

    let mut token_stream = proc_macro2::TokenStream::new();
    block.to_tokens(&mut token_stream);
    let block_string = token_stream.to_string();

    quote! {
        #(#attrs)*
        #[test]
        #vis #sig {
            let engine = rhai::Engine::new();
            engine.run(#block_string);
        }
    }
    .into()
}
