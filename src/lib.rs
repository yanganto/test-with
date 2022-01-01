use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let var_name = attr.to_string().replace(" ", "");
    let ignore_msg = format!("ignored, because variable `{}` not found", &var_name);
    return if std::env::var(var_name).is_ok() {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else {
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
}
