use std::{fs::metadata, path::Path};

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_path_condition(attr_str: String) -> (bool, String) {
    let paths: Vec<&str> = attr_str.split(',').collect();
    let mut missing_paths = vec![];
    for path in paths.iter() {
        if metadata(path.trim_matches('"')).is_err() {
            missing_paths.push(path.to_string());
        }
    }
    let ignore_msg = if missing_paths.len() == 1 {
        format!("because path not found: {}", missing_paths[0])
    } else {
        format!(
            "because following paths not found: \n{}\n",
            missing_paths.join("\n")
        )
    };
    (missing_paths.is_empty(), ignore_msg)
}

pub(crate) fn check_file_condition(attr_str: String) -> (bool, String) {
    let files: Vec<&str> = attr_str.split(',').collect();
    let mut missing_files = vec![];
    for file in files.iter() {
        if !Path::new(file.trim_matches('"')).is_file() {
            missing_files.push(file.to_string());
        }
    }
    let ignore_msg = if missing_files.len() == 1 {
        format!("because file not found: {}", missing_files[0])
    } else {
        format!(
            "because following files not found: \n{}\n",
            missing_files.join("\n")
        )
    };
    (missing_files.is_empty(), ignore_msg)
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_file(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let files: Vec<&str> = attr_str.split(',').collect();
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
                let mut missing_files = vec![];
                #(
                    if !std::path::Path::new(#files.trim_matches('"')).is_file() {
                        missing_files.push(#files);
                    }
                )*

                match missing_files.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because file not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following files not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files.join(", ")
                    ).into()),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_files = vec![];
                #(
                    if !std::path::Path::new(#files.trim_matches('"')).is_file() {
                        missing_files.push(#files);
                    }
                )*

                match missing_files.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
                    1 => Err(
                        format!("{}because file not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following files not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files.join(", ")
                    ).into()),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_files = vec![];
                #(
                    if !std::path::Path::new(#files.trim_matches('"')).is_file() {
                        missing_files.push(#files);
                    }
                )*

                match missing_files.len() {
                    0 => {
                        #ident();
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because file not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following files not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_files.join(", ")
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
pub(crate) fn runtime_path(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let paths: Vec<&str> = attr_str.split(',').collect();
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
                let mut missing_paths = vec![];
                #(
                    if std::fs::metadata(#paths.trim_matches('"')).is_err() {
                        missing_paths.push(#paths.to_string());
                    }
                )*

                match missing_paths.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because path not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following paths not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths.join(", ")
                    ).into()),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_paths = vec![];
                #(
                    if std::fs::metadata(#paths.trim_matches('"')).is_err() {
                        missing_paths.push(#paths.to_string());
                    }
                )*

                match missing_paths.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
                    1 => Err(
                        format!("{}because path not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following paths not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths.join(", ")
                    ).into()),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_paths = vec![];
                #(
                    if std::fs::metadata(#paths.trim_matches('"')).is_err() {
                        missing_paths.push(#paths.to_string());
                    }
                )*

                match missing_paths.len() {
                    0 => {
                        #ident();
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because path not found: {}",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following paths not found: \n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_paths.join(", ")
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
