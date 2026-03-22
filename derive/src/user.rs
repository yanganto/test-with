#[cfg(target_os = "windows")]
use proc_macro_error2::abort_call_site;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

#[cfg(not(target_os = "windows"))]
pub(crate) fn check_root_condition(_attr_str: String) -> (bool, String) {
    let current_user_id = uzers::get_current_uid();
    (
        current_user_id == 0,
        "because this case should run with root".into(),
    )
}
#[cfg(target_os = "windows")]
pub(crate) fn check_root_condition(_attr_str: String) -> (bool, String) {
    abort_call_site!("windows do not support root user condition")
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn check_group_condition(group_name: String) -> (bool, String) {
    let current_user_id = uzers::get_current_uid();

    let in_group = match uzers::get_user_by_uid(current_user_id) {
        Some(user) => {
            let mut in_group = false;
            for group in user.groups().expect("user not found") {
                if in_group {
                    break;
                }
                in_group |= group.name().to_string_lossy() == group_name;
            }
            in_group
        }
        None => false,
    };
    (
        in_group,
        format!("because this case should run user in group {}", group_name),
    )
}

#[cfg(target_os = "windows")]
pub(crate) fn check_group_condition(group_name: String) -> (bool, String) {
    abort_call_site!("windows do not support user group condition")
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn check_user_condition(user_name: String) -> (bool, String) {
    let is_user = match uzers::get_current_username() {
        Some(uname) => uname.to_string_lossy() == user_name,
        None => false,
    };
    (
        is_user,
        format!("because this case should run with user {}", user_name),
    )
}
#[cfg(target_os = "windows")]
pub(crate) fn check_user_condition(user_name: String) -> (bool, String) {
    abort_call_site!("windows do not support user condition")
}

#[cfg(all(feature = "runtime", not(target_os = "windows")))]
pub(crate) fn runtime_root(_attr: TokenStream, stream: TokenStream) -> TokenStream {
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
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                if 0 == test_with::uzers::get_current_uid() {
                    #ident().await;
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with("because this case should run with root".to_string()))
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                if 0 == test_with::uzers::get_current_uid() {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(test_with::Completion::Completed)
                    }
                } else {
                    Ok(test_with::Completion::ignored_with("because this case should run with root".to_string()))
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                if 0 == test_with::uzers::get_current_uid() {
                    #ident();
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with("because this case should run with root".to_string()))
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

#[cfg(all(feature = "runtime", target_os = "windows"))]
pub(crate) fn runtime_root(_attr: TokenStream, stream: TokenStream) -> TokenStream {
    abort_call_site!("windows do not support root user condition")
}

#[cfg(all(feature = "runtime", not(target_os = "windows")))]
pub(crate) fn runtime_group(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let group_name = attr.to_string().replace(' ', "");
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
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let current_user_id = test_with::uzers::get_current_uid();
                let in_group = match test_with::uzers::get_user_by_uid(current_user_id) {
                    Some(user) => {
                        let mut in_group = false;
                        for group in user.groups().expect("user not found") {
                            if in_group {
                                break;
                            }
                            in_group |= group.name().to_string_lossy() == #group_name;
                        }
                        in_group
                    }
                    None => false,
                };
                if in_group {
                    #ident().await;
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run user in group {}", #group_name)))
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let current_user_id = test_with::uzers::get_current_uid();
                let in_group = match test_with::uzers::get_user_by_uid(current_user_id) {
                    Some(user) => {
                        let mut in_group = false;
                        for group in user.groups().expect("user not found") {
                            if in_group {
                                break;
                            }
                            in_group |= group.name().to_string_lossy() == #group_name;
                        }
                        in_group
                    }
                    None => false,
                };
                if in_group {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(test_with::Completion::Completed)
                    }
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run user in group {}", #group_name)))
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let current_user_id = test_with::uzers::get_current_uid();
                let in_group = match test_with::uzers::get_user_by_uid(current_user_id) {
                    Some(user) => {
                        let mut in_group = false;
                        for group in user.groups().expect("user not found") {
                            if in_group {
                                break;
                            }
                            in_group |= group.name().to_string_lossy() == #group_name;
                        }
                        in_group
                    }
                    None => false,
                };
                if in_group {
                    #ident();
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run user in group {}", #group_name)))
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

#[cfg(all(feature = "runtime", target_os = "windows"))]
pub(crate) fn runtime_group(attr: TokenStream, stream: TokenStream) -> TokenStream {
    abort_call_site!("windows do not support user group condition")
}

#[cfg(all(feature = "runtime", not(target_os = "windows")))]
pub fn runtime_user(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let user_name = attr.to_string().replace(' ', "");
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
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let is_user = match test_with::uzers::get_current_username() {
                    Some(uname) => uname.to_string_lossy() == #user_name,
                    None => false,
                };
                if is_user {
                    #ident().await;
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run with user {}", #user_name)))
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let is_user = match test_with::uzers::get_current_username() {
                    Some(uname) => uname.to_string_lossy() == #user_name,
                    None => false,
                };
                if is_user {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(test_with::Completion::Completed)
                    }
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run with user {}", #user_name)))
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let is_user = match test_with::uzers::get_current_username() {
                    Some(uname) => uname.to_string_lossy() == #user_name,
                    None => false,
                };
                if is_user {
                    #ident();
                    Ok(test_with::Completion::Completed)
                } else {
                    Ok(test_with::Completion::ignored_with(format!("because this case should run with user {}", #user_name)))
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

#[cfg(all(feature = "runtime", target_os = "windows"))]
pub(crate) fn runtime_user(attr: TokenStream, stream: TokenStream) -> TokenStream {
    abort_call_site!("windows do not support user condition")
}
