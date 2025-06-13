use proc_macro_error2::abort_call_site;
use which::which;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_executable_and_condition(attr_str: String) -> (bool, String) {
    let executables: Vec<&str> = attr_str.split(',').collect();
    let mut missing_executables = vec![];
    for exe in executables.iter() {
        if which(exe.trim_matches('"')).is_err() {
            missing_executables.push(exe.to_string());
        }
    }
    let ignore_msg = if missing_executables.len() == 1 {
        format!("because executable not found: {}", missing_executables[0])
    } else {
        format!(
            "because following executables not found: \n{}\n",
            missing_executables.join("\n")
        )
    };
    (missing_executables.is_empty(), ignore_msg)
}

pub(crate) fn check_executable_or_condition(attr_str: String) -> (bool, String) {
    let executables: Vec<&str> = attr_str.split("||").collect();
    for exe in executables.iter() {
        if which(exe.trim_matches('"')).is_ok() {
            return (true, String::new());
        }
    }
    (
        false,
        format!("because none of executables can be found: {}", attr_str),
    )
}

pub(crate) fn check_executable_condition(attr_str: String) -> (bool, String) {
    let has_and_cond = attr_str.contains(',');
    let has_or_cond = attr_str.contains("||");
    if has_and_cond && has_or_cond {
        abort_call_site!("',' and '||' can not be used at the same time")
    } else if has_or_cond {
        check_executable_or_condition(attr_str)
    } else {
        check_executable_and_condition(attr_str)
    }
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_executable(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let has_and_cond = attr_str.contains(',');
    let has_or_cond = attr_str.contains("||");
    if has_and_cond && has_or_cond {
        abort_call_site!("',' and '||' can not be used at the same time")
    }
    let executables: Vec<&str> = if has_or_cond {
        attr_str.split("||").collect()
    } else {
        attr_str.split(',').collect()
    };
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

    let check_fn = match (has_or_cond, &sig.asyncness, &sig.output) {
        (true, Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                #(
                    if libtest_with::which::which(#executables).is_ok() {
                        #ident().await;
                        return Ok(());
                    }
                )*
                Err(format!("{}because none of executables can be found:\n{}\n",
                    libtest_with::RUNTIME_IGNORE_PREFIX, attr_str).into())
            }
        },
        (false, Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_executables = vec![];
                #(
                    if libtest_with::which::which(#executables).is_err() {
                        missing_executables.push(#executables);
                    }
                )*
                match missing_executables.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because executable {} not found",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following executables not found:\n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables.join(", ")
                    ).into()),
                }
            }
        },
        (true, Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                #(
                    if libtest_with::which::which(#executables).is_ok() {
                        if let Err(e) = #ident().await {
                            return Err(format!("{e:?}").into());
                        } else {
                            return Ok(());
                        }
                    }
                )*
                Err(format!("{}because none of executables can be found:\n{}\n",
                    libtest_with::RUNTIME_IGNORE_PREFIX, attr_str).into())
            }
        },
        (false, Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_executables = vec![];
                #(
                    if libtest_with::which::which(#executables).is_err() {
                        missing_executables.push(#executables);
                    }
                )*
                match missing_executables.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
                    1 => Err(
                        format!("{}because executable {} not found",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following executables not found:\n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables.join(", ")
                    ).into()),
                }
            }
        },
        (true, None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                #(
                    if libtest_with::which::which(#executables).is_ok() {
                        #ident();
                        return Ok(());
                    }
                )*
                Err(format!("{}because none of executables can be found:\n{}\n",
                    libtest_with::RUNTIME_IGNORE_PREFIX, attr_str).into())
            }
        },
        (false, None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_executables = vec![];
                #(
                    if libtest_with::which::which(#executables).is_err() {
                        missing_executables.push(#executables);
                    }
                )*
                match missing_executables.len() {
                    0 => {
                        #ident();
                        Ok(())
                    },
                    1 => Err(
                        format!("{}because executable {} not found",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables[0]
                    ).into()),
                    _ => Err(
                        format!("{}because following executables not found:\n{}\n",
                                libtest_with::RUNTIME_IGNORE_PREFIX, missing_executables.join(", ")
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
