#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_http_condition(attr_str: String) -> (bool, String) {
    let links: Vec<&str> = attr_str.split(',').collect();
    let mut missing_links = vec![];
    let client = reqwest::blocking::Client::new();
    for link in links.iter() {
        if client.head(&format!("http://{}", link)).send().is_err() {
            missing_links.push(format!("http://{link:}"));
        }
    }
    let ignore_msg = if missing_links.len() == 1 {
        format!("because {} not response", missing_links[0])
    } else {
        format!(
            "because following links not response: \n{}\n",
            missing_links.join("\n")
        )
    };
    (missing_links.is_empty(), ignore_msg)
}

pub(crate) fn check_https_condition(attr_str: String) -> (bool, String) {
    let links: Vec<&str> = attr_str.split(',').collect();
    let mut missing_links = vec![];
    let client = reqwest::blocking::Client::new();
    for link in links.iter() {
        if client.head(&format!("https://{}", link)).send().is_err() {
            missing_links.push(format!("https://{link:}"));
        }
    }
    let ignore_msg = if missing_links.len() == 1 {
        format!("because {} not response", missing_links[0])
    } else {
        format!(
            "because following links not response: \n{}\n",
            missing_links.join("\n")
        )
    };
    (missing_links.is_empty(), ignore_msg)
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_http(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let links: Vec<&str> = attr_str.split(',').collect();
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(
        &format!("_check_{}", ident.to_string()),
        proc_macro2::Span::call_site(),
    );
    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_links = vec![];
                let client = libtest_with::reqwest::Client::new();
                #(
                    if client.head(&format!("http://{}", #links)).send().await.is_err() {
                        missing_links.push(format!("http://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_links = vec![];
                let client = libtest_with::reqwest::Client::new();
                #(
                    if client.head(&format!("http://{}", #links)).send().await.is_err() {
                        missing_links.push(format!("http://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {

                let mut missing_links = vec![];
                let client = libtest_with::reqwest::blocking::Client::new();
                #(
                    if client.head(&format!("http://{}", #links)).send().is_err() {
                        missing_links.push(format!("http://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        #ident();
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_https(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let links: Vec<&str> = attr_str.split(',').collect();
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(
        &format!("_check_{}", ident.to_string()),
        proc_macro2::Span::call_site(),
    );

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_links = vec![];
                let client = libtest_with::reqwest::Client::new();
                #(
                    if client.head(&format!("https://{}", #links)).send().await.is_err() {
                        missing_links.push(format!("https://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_links = vec![];
                let client = libtest_with::reqwest::Client::new();
                #(
                    if client.head(&format!("https://{}", #links)).send().await.is_err() {
                        missing_links.push(format!("https://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_links = vec![];
                let client = libtest_with::reqwest::blocking::Client::new();
                #(
                    if client.head(&format!("https://{}", #links)).send().is_err() {
                        missing_links.push(format!("https://{}", #links));
                    }
                )*
                match missing_links.len() {
                    0 => {
                        #ident();
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because {} not response",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following links not response: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_links.join(", ")
                    ).into()),
                }
            }
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}
