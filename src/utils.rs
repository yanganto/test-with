use regex::Regex;

use proc_macro::TokenStream;
#[cfg(feature = "ign-msg")]
use proc_macro2::Span;
use proc_macro2::TokenTree;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::Attribute;
#[cfg(feature = "ign-msg")]
use syn::Signature;
use syn::{Ident, Item, ItemFn, ItemMod};

// check for `#[test]`, `#[tokio::test]`, `#[async_std::test]`
pub(crate) fn has_test_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Some(seg) = attr.path.segments.last() {
            if seg.ident == "test" {
                return true;
            }
        }
    }
    false
}

// check for `#[cfg(test)]`
pub(crate) fn has_test_cfg(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        for token in attr.tokens.clone().into_iter() {
            if let TokenTree::Group(group) = token {
                for tt in group.stream().into_iter() {
                    if "test" == tt.to_string() {
                        return true;
                    }
                }
            }
        }
    }
    false
}

#[cfg(feature = "ign-msg")]
pub(crate) fn rewrite_fn_sig_with_msg(sig: &mut Signature, msg: &String) {
    let re = unsafe { Regex::new(r"[^\w]").unwrap_unchecked() };
    let new_fn_name = Ident::new(
        &format!("{}__{}", sig.ident, re.replace_all(msg, "_")),
        Span::call_site(),
    );
    sig.ident = new_fn_name;
}

#[cfg(feature = "ign-msg")]
pub(crate) fn rewrite_fn_ident_with_msg(ident: Ident, msg: &String) -> Ident {
    let re = unsafe { Regex::new(r"[^\w]").unwrap_unchecked() };
    Ident::new(
        &format!("{}__{}", ident.to_string(), re.replace_all(msg, "_")),
        Span::call_site(),
    )
}

pub(crate) fn is_module(context: &TokenStream) -> bool {
    let re = unsafe { Regex::new("(?:pub mod |mod )").unwrap_unchecked() };
    re.is_match(&context.to_string())
}

pub(crate) fn fn_macro(
    attr: TokenStream,
    input: ItemFn,
    check_condition: fn(String) -> (bool, String),
) -> TokenStream {
    #[cfg(feature = "ign-msg")]
    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input;
    #[cfg(not(feature = "ign-msg"))]
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let attr_str = attr.to_string().replace(' ', "");
    let (all_var_exist, ignore_msg) = check_condition(attr_str);
    let has_test = has_test_attr(&attrs);

    return if all_var_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_var_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_sig_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_sig_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
}

pub(crate) fn mod_macro(
    attr: TokenStream,
    input: ItemMod,
    check_condition: fn(String) -> (bool, String),
) -> TokenStream {
    let ItemMod {
        attrs,
        vis,
        mod_token,
        ident,
        content,
        ..
    } = input;
    if let Some(content) = content {
        let content = content.1;
        let attr_str = attr.to_string().replace(' ', "");
        let (all_var_exist, ignore_msg) = check_condition(attr_str);
        let has_test = has_test_cfg(&attrs);

        return if all_var_exist && has_test {
            quote! {
                #(#attrs)*
                #[cfg(test)]
                #vis #mod_token #ident {
                    #(#content)*
                }
            }
            .into()
        } else if all_var_exist {
            quote! {
                #(#attrs)*
                #[cfg(test)]
                #vis #mod_token #ident {
                    #(#content)*
                }
            }
            .into()
        } else if has_test {
            let fn_names: Vec<Ident> = content
                .into_iter()
                .filter_map(|i| match i {
                    Item::Fn(ItemFn { sig, .. }) => {
                        #[cfg(not(feature = "ign-msg"))]
                        let ident = sig.ident;
                        #[cfg(feature = "ign-msg")]
                        let ident = rewrite_fn_ident_with_msg(sig.ident, &ignore_msg);
                        Some(ident)
                    }
                    _ => None,
                })
                .collect();
            quote! {
                #(#attrs)*
                #vis #mod_token #ident {
                    #(
                        #[test]
                        #[ignore = #ignore_msg ]
                        fn #fn_names () {}
                    )*
                }
            }
            .into()
        } else {
            let fn_names: Vec<Ident> = content
                .into_iter()
                .filter_map(|i| match i {
                    Item::Fn(ItemFn { sig, .. }) => {
                        #[cfg(not(feature = "ign-msg"))]
                        let ident = sig.ident;
                        #[cfg(feature = "ign-msg")]
                        let ident = rewrite_fn_ident_with_msg(sig.ident, &ignore_msg);
                        Some(ident)
                    }
                    _ => None,
                })
                .collect();
            quote! {
                #(#attrs)*
                #[cfg(test)]
                #vis #mod_token #ident {
                    #(
                        #[test]
                        #[ignore = #ignore_msg ]
                        fn #fn_names () {}
                    )*
                }
            }
            .into()
        };
    } else {
        abort_call_site!("should use on mod with context")
    }
}
