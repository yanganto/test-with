use proc_macro::TokenStream;
use proc_macro_error2::abort_call_site;
use syn::{parse_macro_input, Item, ItemFn, ItemMod, ItemStruct, ItemType};

pub(crate) fn runner(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let mod_names: Vec<syn::Ident> = input_str
        .split(",")
        .map(|s| syn::Ident::new(s.trim(), proc_macro2::Span::call_site()))
        .collect();
    quote::quote! {
        fn main() {
            let args = libtest_with::Arguments::from_args();
            let mut no_env_tests = Vec::new();
            #(
                match #mod_names::_runtime_tests() {
                    (Some(env), tests) => {
                        libtest_with::run(&args, tests).exit_if_failed();
                        drop(env);
                    },
                    (None, mut tests) => no_env_tests.append(&mut tests),
                }
            )*
            libtest_with::run(&args, no_env_tests).exit();
        }
    }
    .into()
}

pub(crate) fn tokio_runner(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let mod_names: Vec<syn::Ident> = input_str
        .split(",")
        .map(|s| syn::Ident::new(s.trim(), proc_macro2::Span::call_site()))
        .collect();
    quote::quote! {
        fn main() {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    #(
                        #mod_names::_runtime_tests().await;
                    )*
                })
        }
    }
    .into()
}

pub(crate) fn module(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    let ItemMod {
        attrs,
        vis,
        mod_token,
        ident,
        content,
        ..
    } = parse_macro_input!(stream as ItemMod);

    if let Some(content) = content {
        let content = content.1;
        if crate::utils::has_test_cfg(&attrs) {
            abort_call_site!("should not use `#[cfg(test)]` on the mod with `#[test_with::module]`")
        } else {
            let mut test_env_type = None;
            let test_metas: Vec<(String, bool)> = content
                .iter()
                .filter_map(|c| match c {
                    Item::Fn(ItemFn {
                        sig:
                            syn::Signature {
                                ident, asyncness, ..
                            },
                        attrs,
                        ..
                    }) => match crate::utils::test_with_attrs(&attrs) {
                        (true, true, _) => abort_call_site!(
                            "should not use #[test] for method in `#[test_with::module]`"
                        ),
                        (_, true, false) => abort_call_site!(
                            "use `#[test_with::runtime_*]` for method in `#[test_with::module]`"
                        ),
                        (false, true, true) => Some((ident.to_string(), asyncness.is_some())),
                        _ => None,
                    },
                    Item::Struct(ItemStruct { ident, vis, .. })
                    | Item::Type(ItemType { ident, vis, .. }) => {
                        if ident.to_string() == "TestEnv" {
                            match vis {
                                syn::Visibility::Public(_) => test_env_type = Some(ident),
                                _ => abort_call_site!("TestEnv should be pub for testing"),
                            }
                        }
                        None
                    }
                    _ => None,
                })
                .collect();

            let runtime_test_fn = match (test_env_type, test_metas.iter().any(|m| m.1)) {
                (Some(test_env_type), false) => {
                    let test_names: Vec<String> = test_metas.into_iter().map(|m| m.0).collect();
                    let check_names: Vec<syn::Ident> = test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    quote::quote! {
                        pub fn _runtime_tests() -> (Option<#test_env_type>, Vec<libtest_with::Trial>) {
                            use libtest_with::Trial;
                            (
                                Some(#test_env_type::default()),
                                vec![
                                    #(Trial::test(#test_names, #check_names),)*
                                ]
                            )
                        }
                    }
                }
                (Some(test_env_type), true) => {
                    let async_test_names: Vec<String> = test_metas
                        .iter()
                        .filter(|m| m.1)
                        .map(|m| m.0.clone())
                        .collect();
                    let sync_test_names: Vec<String> = test_metas
                        .into_iter()
                        .filter(|m| !m.1)
                        .map(|m| m.0)
                        .collect();
                    let total = async_test_names.len() + sync_test_names.len();
                    let async_check_names: Vec<syn::Ident> = async_test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    let sync_check_names: Vec<syn::Ident> = sync_test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    quote::quote! {
                        pub async fn _runtime_tests() {
                            let env = #test_env_type::default();
                            let mut failed = 0;
                            let mut passed = 0;
                            let mut ignored = 0;
                            println!("running {} tests of {}\n", #total, stringify!(#ident));
                            #(
                                print!("test {}::{} ... ", stringify!(#ident), #async_test_names);
                                if let Err(e) = #async_check_names().await {
                                    if let Some(msg) = e.message() {
                                        if msg.starts_with(libtest_with::RUNTIME_IGNORE_PREFIX) {
                                            println!("ignored, {}", msg[12..].to_string());
                                            ignored += 1;
                                        } else {
                                            println!("FAILED, {msg}");
                                            failed += 1;
                                        }
                                    } else {
                                        println!("FAILED");
                                        failed += 1;
                                    }
                                } else {
                                    println!("ok");
                                    passed += 1;
                                }
                            )*
                            #(
                                print!("test {}::{} ... ", stringify!(#ident), #sync_test_names);
                                if let Err(e) = #sync_check_names() {
                                    if let Some(msg) = e.message() {
                                        if msg.starts_with(libtest_with::RUNTIME_IGNORE_PREFIX) {
                                            println!("ignored, {}", msg[12..].to_string());
                                            ignored += 1;
                                        } else {
                                            println!("FAILED, {msg}");
                                            failed += 1;
                                        }
                                    } else {
                                        println!("FAILED");
                                        failed += 1;
                                    }
                                } else {
                                    println!("ok");
                                    passed += 1;
                                }
                            )*
                            drop(env);
                            if failed > 0 {
                                println!("\ntest result: failed. {passed} passed; {failed} failed; {ignored} ignored;\n");
                                std::process::exit(1);
                            } else {
                                println!("\ntest result: ok. {passed} passed; {failed} failed; {ignored} ignored;\n");
                            }
                        }
                    }
                }
                (None, false) => {
                    let test_names: Vec<String> = test_metas.into_iter().map(|m| m.0).collect();
                    let check_names: Vec<syn::Ident> = test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    quote::quote! {
                        pub fn _runtime_tests() -> (Option<()>, Vec<libtest_with::Trial>) {
                            use libtest_with::Trial;
                            (
                                None,
                                vec![
                                    #(Trial::test(#test_names, #check_names),)*
                                ]
                            )
                        }
                    }
                }
                (None, true) => {
                    let async_test_names: Vec<String> = test_metas
                        .iter()
                        .filter(|m| m.1)
                        .map(|m| m.0.clone())
                        .collect();
                    let sync_test_names: Vec<String> = test_metas
                        .into_iter()
                        .filter(|m| !m.1)
                        .map(|m| m.0)
                        .collect();
                    let total = async_test_names.len() + sync_test_names.len();
                    let async_check_names: Vec<syn::Ident> = async_test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    let sync_check_names: Vec<syn::Ident> = sync_test_names
                        .iter()
                        .map(|c| {
                            syn::Ident::new(
                                &format!("_check_{}", c.to_string()),
                                proc_macro2::Span::call_site(),
                            )
                        })
                        .collect();
                    quote::quote! {
                        pub async fn _runtime_tests() {
                            let mut failed = 0;
                            let mut passed = 0;
                            let mut ignored = 0;
                            println!("running {} tests of {}\n", #total, stringify!(#ident));
                            #(
                                print!("test {}::{} ... ", stringify!(#ident), #async_test_names);
                                if let Err(e) = #async_check_names().await {
                                    if let Some(msg) = e.message() {
                                        if msg.starts_with(libtest_with::RUNTIME_IGNORE_PREFIX) {
                                            println!("ignored, {}", msg[12..].to_string());
                                            ignored += 1;
                                        } else {
                                            println!("FAILED, {msg}");
                                            failed += 1;
                                        }
                                    } else {
                                        println!("FAILED");
                                        failed += 1;
                                    }
                                } else {
                                    println!("ok");
                                    passed += 1;
                                }
                            )*
                            #(
                                print!("test {}::{} ... ", stringify!(#ident), #sync_test_names);
                                if let Err(e) = #sync_check_names() {
                                    if let Some(msg) = e.message() {
                                        if msg.starts_with(libtest_with::RUNTIME_IGNORE_PREFIX) {
                                            println!("ignored, {}", msg[12..].to_string());
                                            ignored += 1;
                                        } else {
                                            println!("FAILED, {msg}");
                                            failed += 1;
                                        }
                                    } else {
                                        println!("FAILED");
                                        failed += 1;
                                    }
                                } else {
                                    println!("ok");
                                    passed += 1;
                                }
                            )*
                            if failed > 0 {
                                println!("\ntest result: failed. {passed} passed; {failed} failed; {ignored} ignored;\n");
                                std::process::exit(1);
                            } else {
                                println!("\ntest result: ok. {passed} passed; {failed} failed; {ignored} ignored;\n");
                            }
                        }
                    }
                }
            };
            quote::quote! {
                #(#attrs)*
                #vis #mod_token #ident {
                    use super::*;
                    #runtime_test_fn
                    #(#content)*
                }
            }
            .into()
        }
    } else {
        abort_call_site!("should use on mod with context")
    }
}
