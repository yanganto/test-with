use regex::Regex;

use proc_macro::TokenStream;
#[cfg(feature = "ign-msg")]
use proc_macro2::Span;
use proc_macro2::TokenTree;
use proc_macro_error2::abort_call_site;
use quote::quote;
#[cfg(feature = "ign-msg")]
use syn::Signature;
use syn::{Attribute, Meta};
use syn::{Block, Ident, Item, ItemFn, ItemMod};

// check for `#[test]`, `#[tokio::test]`, `#[async_std::test]`
pub(crate) fn has_test_attr(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Some(seg) = attr.path().segments.last() {
            if seg.ident == "test" {
                return true;
            }
        }
    }
    false
}

// check
// first for `#[test]`, `#[tokio::test]`, `#[async_std::test]`
// second for `#[test_with::*]`
// third for runtime mod `#[test_with::runtime_*]`
#[cfg(feature = "runtime")]
pub(crate) fn test_with_attrs(attrs: &[Attribute]) -> (bool, bool, bool) {
    let mut has_test = false;
    let mut has_test_with = false;
    let mut has_rt_test_with = false;
    for attr in attrs.iter() {
        if let Some(seg) = attr.path().segments.last() {
            if seg.ident == "test" {
                has_test = true;
            }
        }
        if let (Some(first_seg), Some(last_seg)) =
            (attr.path().segments.first(), attr.path().segments.last())
        {
            if first_seg.ident == "test_with" {
                has_test_with = true;
            }
            if last_seg.ident.to_string().starts_with("runtime_") {
                has_rt_test_with = true;
            }
        }
    }
    (has_test, has_test_with, has_rt_test_with)
}

// check the attribute order for `#[serial]`
pub(crate) fn check_before_attrs(attrs: &[Attribute]) {
    for attr in attrs.iter() {
        if let Some(seg) = attr.path().segments.last() {
            if seg.ident == "serial" {
                abort_call_site!("`#[test_with::*]` should place after `#[serial]`");
            }
        }
    }
}

// check for `#[cfg(test)]`
pub(crate) fn has_test_cfg(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Meta::List(metalist) = &attr.meta {
            for token in metalist.tokens.clone().into_iter() {
                if let TokenTree::Group(group) = token {
                    for tt in group.stream().into_iter() {
                        if "test" == tt.to_string() {
                            return true;
                        }
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
    check_before_attrs(&attrs);
    let has_test = has_test_attr(&attrs);

    if all_var_exist && has_test {
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
    }
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

        if all_var_exist && has_test {
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
        }
    } else {
        abort_call_site!("should use on mod with context")
    }
}

pub(crate) fn lock_macro(attr: TokenStream, input: ItemFn) -> TokenStream {
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input;
    let Block { stmts, .. } = *block;
    let attr_str = attr.to_string().replace(' ', "");
    let lock_attrs: Vec<&str> = attr_str.split(',').collect();
    let (lock_name, wait_time) = match (lock_attrs.first(), lock_attrs.get(1)) {
        (Some(name), None) => (name, 60),
        (Some(name), Some(sec)) => {
            if let Ok(wait_time) = sec.parse::<usize>() {
                (name, wait_time)
            } else {
                abort_call_site!("`The second parameter of #[test_with::lock]` should be a number for waiting time");
            }
        }
        (None, _) => {
            abort_call_site!("`#[test_with::lock]` need a name for the file lock");
        }
    };
    let lock_file = std::env::temp_dir().join(lock_name).display().to_string();

    check_before_attrs(&attrs);

    if has_test_attr(&attrs) {
        quote! {
            #(#attrs)*
            #vis #sig {
                let mut _test_with_lock = None;
                for i in 0..#wait_time {
                    if let Ok(file) = std::fs::File::create_new(#lock_file) {
                        _test_with_lock = Some(file);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                if _test_with_lock.is_none() {
                    panic!("Fail to acquire the lock for the testcase")
                }
                #(#stmts)*
                if std::fs::remove_file(#lock_file).is_err() {
                    panic!("Fail to unlock the testcase")
                }
            }
        }
        .into()
    } else {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig {
                let mut _test_with_lock = None;
                for i in 0..#wait_time {
                    if let Ok(file) = std::fs::File::create_new(#lock_file) {
                        _test_with_lock = Some(file);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                if _test_with_lock.is_none() {
                    panic!("Fail to acquire the lock for the testcase")
                }
                #(#stmts)*
                if std::fs::remove_file(#lock_file).is_err() {
                    panic!("Fail to unlock the testcase")
                }
            }
        }
        .into()
    }
}

/// Sanitize the attribute string to remove any leading or trailing whitespace
/// and split the string into an iterator of individual environment variable names.
pub fn sanitize_env_vars_attr(attr_str: &str) -> impl Iterator<Item = &str> {
    attr_str.split(',').map(str::trim)
}

#[cfg(test)]
mod tests {
    use super::sanitize_env_vars_attr;

    #[test]
    fn sanitize_single_env_var() {
        //* Given
        let env_var = "FOO";

        let attr_str = env_var.to_string();

        //* When
        let result = sanitize_env_vars_attr(&attr_str).collect::<Vec<_>>();

        //* Then
        assert_eq!(result, vec!["FOO"]);
    }

    #[test]
    fn sanitize_multiple_env_vars() {
        //* Given
        let env_var1 = "FOO";
        let env_var2 = "BAR";
        let env_var3 = "BAZ";

        let attr_str = format!("\t{},\n\t{},\n\t{}", env_var1, env_var2, env_var3);

        //* When
        let result = sanitize_env_vars_attr(&attr_str).collect::<Vec<_>>();

        //* Then
        assert_eq!(result, vec!["FOO", "BAR", "BAZ"]);
    }

    #[test]
    fn sanitize_env_vars_with_whitespace() {
        //* Given
        let env_var = "FOO BAR";

        let attr_str = env_var.to_string();

        //* When
        let result = sanitize_env_vars_attr(&attr_str).collect::<Vec<_>>();

        //* Then
        assert_eq!(result, vec!["FOO BAR"]);
    }
}
