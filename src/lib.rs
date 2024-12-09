//! `test_with` provides [macro@env], [macro@file], [macro@path], [macro@http], [macro@https],
//! [macro@icmp], [macro@tcp], [macro@root], [macro@group], [macro@user], [macro@mem], [macro@swap],
//! [macro@cpu_core], [macro@phy_core], [macro@executable], [macro@timezone] macros to help you run
//! test case only with the condition is fulfilled.  If the `#[test]` is absent for the test case,
//! `#[test_with]` will add it to the test case automatically.
//!
//! This crate help you easier make integrating test case and has a good cargo summary on CI server,
//! and will not affect on your binary output when you dependent it as dev-dependency as following.
//! ```toml
//! [dev-dependencies]
//! test-with = "*"
//! ```
//! All features will be opt-in default feature, so this crate will be easier to use, if you using
//! a CI server with really limitation resource and want this crate as slim as possible, you can
//! select the feature you want as following.
//! ```toml
//! [dev-dependencies]
//! test-with = { version = "*", default-features = false, features = ["net"] }
//! ```
//!
//! The solution to have a real runtime condition check, we need to put the test as normal function
//! as an example, then use `cargo run --example`
//! The `test-with` need be included as normal dependency with `runtime` feature.
//! And also include the `libtest-with` with corresponding features in `Cargo.toml`
//! [macro@runner] and [macro@module] are for the basic skeleton of the test runner.
//! [macro@runtime_env], [macro@runtime_no_env], [macro@runtime_file], [macro@runtime_path],
//! [macro@runtime_http], [macro@runtime_https], [macro@runtime_icmp], [macro@runtime_tcp],
//! [macro@runtime_root], [macro@runtime_group], [macro@runtime_user], [macro@runtime_mem],
//! [macro@runtime_free_mem], [macro@runtime_available_mem], [macro@runtime_swap],
//! [macro@runtime_free_swap], [macro@runtime_available_swap], [macro@runtime_cpu_core],
//! [macro@runtime_phy_core], [macro@runtime_executable], [macro@runtime_timezone]
//! and [macro@runtime_ignore_if] are used to transform a normal function to a testcase.
//!
//! ```toml
//! [dependencies]
//! test-with = { version = "*", default-features = false, features = ["runtime"] }
//! libtest-with = { version = "0.7.0-3", features = ["net", "resource", "user", "executable", "timezone"] }
//! ```
//!
//! ```rust
//! // write as example in examples/*rs
//! test_with::runner!(env);
//! #[test_with::module]
//! mod env {
//! #[test_with::runtime_env(PWD)]
//! fn test_works() {
//!     assert!(true);
//!     }
//! }
//! ```

use std::{fs::metadata, path::Path};

#[cfg(feature = "icmp")]
use std::net::IpAddr;
use std::net::TcpStream;

use proc_macro::TokenStream;
use proc_macro_error2::abort_call_site;
use proc_macro_error2::proc_macro_error;
use syn::{parse_macro_input, ItemFn, ItemMod};

#[cfg(feature = "runtime")]
use syn::{Item, ItemStruct, ItemType};

#[cfg(feature = "executable")]
use which::which;

use crate::utils::{fn_macro, is_module, lock_macro, mod_macro, sanitize_env_vars_attr};

mod utils;

/// Run test case when the environment variable is set.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // PWD environment variable exists
///     #[test_with::env(PWD)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // NOTHING environment variable does not exist
///     #[test_with::env(NOTHING)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
///
///     // NOT_SAYING environment variable does not exist
///     #[test_with::env(PWD, NOT_SAYING)]
///     #[test]
///     fn test_ignored_too() {
///         panic!("should be ignored")
///     }
/// }
/// ```
/// or run all test cases for test module when the environment variable is set.
/// ```
/// #[test_with::env(PWD)]
/// #[cfg(test)]
/// mod tests {
///
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_env_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_env_condition,
        )
    }
}

fn check_env_condition(attr_str: String) -> (bool, String) {
    let var_names = sanitize_env_vars_attr(&attr_str);

    // Check if the environment variables are set
    let mut missing_vars = vec![];
    for name in var_names {
        if std::env::var(name).is_err() {
            missing_vars.push(name.to_string());
        }
    }

    // Generate ignore message
    let ignore_msg = if missing_vars.is_empty() {
        String::new()
    } else if missing_vars.len() == 1 {
        format!("because variable {} not found", missing_vars[0])
    } else {
        format!(
            "because following variables not found:\n{}\n",
            missing_vars.join(", ")
        )
    };

    (missing_vars.is_empty(), ignore_msg)
}

/// Run test case when the example running and the environment variable is set.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(env);
/// #[test_with::module]
/// mod env {
/// #[test_with::runtime_env(PWD)]
/// fn test_works() {
///     assert!(true);
///     }
/// }
///```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_env(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let var_names: Vec<&str> = attr_str.split(',').collect();
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let mut missing_vars = vec![];
            #(
                if std::env::var(#var_names).is_err() {
                    missing_vars.push(#var_names);
                }
            )*
            match missing_vars.len() {
                0 => {
                    #ident();
                    Ok(())
                },
                1 => Err(
                    format!("{}because variable {} not found",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_vars[0]
                ).into()),
                _ => Err(
                    format!("{}because following variables not found:\n{}\n",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_vars.join(", ")
                ).into()),
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Ignore test case when the environment variable is set.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // The test will be ignored in GITHUB_ACTION
///     #[test_with::no_env(GITHUB_ACTIONS)]
///     #[test]
///     fn test_ignore_in_github_action() {
///         assert!(false);
///     }
/// }
#[proc_macro_attribute]
#[proc_macro_error]
pub fn no_env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_no_env_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_no_env_condition,
        )
    }
}

fn check_no_env_condition(attr_str: String) -> (bool, String) {
    let var_names = sanitize_env_vars_attr(&attr_str);

    // Check if the environment variables are set
    let mut found_vars = vec![];
    for name in var_names {
        if std::env::var(name).is_ok() {
            found_vars.push(name.to_string());
        }
    }

    // Generate ignore message
    let ignore_msg = if found_vars.is_empty() {
        String::new()
    } else if found_vars.len() == 1 {
        format!("because variable {} was found", found_vars[0])
    } else {
        format!(
            "because following variables were found:\n{}\n",
            found_vars.join(", ")
        )
    };

    (found_vars.is_empty(), ignore_msg)
}

/// Ignore test case when the example running and the environment variable is set.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(env);
/// #[test_with::module]
/// mod env {
/// #[test_with::runtime_no_env(NOT_EXIST)]
/// fn test_works() {
///     assert!(true);
///     }
/// }
///```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_no_env(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_no_env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let var_names: Vec<&str> = attr_str.split(',').collect();
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let mut should_no_exist_vars = vec![];
            #(
                if std::env::var(#var_names).is_ok() {
                    should_no_exist_vars.push(#var_names);
                }
            )*
            match should_no_exist_vars.len() {
                0 => {
                    #ident();
                    Ok(())
                },
                1 => Err(
                    format!("{}because variable {} found",
                            libtest_with::RUNTIME_IGNORE_PREFIX, should_no_exist_vars[0]
                ).into()),
                _ => Err(
                    format!("{}because following variables found:\n{}\n",
                            libtest_with::RUNTIME_IGNORE_PREFIX, should_no_exist_vars.join(", ")
                ).into()),
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when the file exist.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // hostname exists
///     #[test_with::file(/etc/hostname)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // nothing file does not exist
///     #[test_with::file(/etc/nothing)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
///
///     // hostname and hosts exist
///     #[test_with::file(/etc/hostname, /etc/hosts)]
///     #[test]
///     fn test_works_too() {
///         assert!(true);
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn file(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_file_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_file_condition,
        )
    }
}

fn check_file_condition(attr_str: String) -> (bool, String) {
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

/// Run test case when the example running and the file exist.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(file);
/// #[test_with::module]
/// mod file {
///     #[test_with::runtime_file(/etc/hostname)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
///```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_file(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_file(attr: TokenStream, stream: TokenStream) -> TokenStream {
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
    quote::quote! {
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

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when the path(file or folder) exist.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // etc exists
///     #[test_with::path(/etc)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // nothing does not exist
///     #[test_with::path(/nothing)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
///
///     // etc and tmp exist
///     #[test_with::path(/etc, /tmp)]
///     #[test]
///     fn test_works_too() {
///         assert!(true);
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn path(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_path_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_path_condition,
        )
    }
}

fn check_path_condition(attr_str: String) -> (bool, String) {
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

/// Run test case when the example running and the path(file or folder) exist.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(path);
/// #[test_with::module]
/// mod path {
///     #[test_with::runtime_path(/etc)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
///```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_path(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_path(attr: TokenStream, stream: TokenStream) -> TokenStream {
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
    quote::quote! {
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

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when the http service exist.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // http service exists
///     #[test_with::http(httpbin.org)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // There is no not.exist.com
///     #[test_with::http(not.exist.com)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "http")]
pub fn http(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_http_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_http_condition,
        )
    }
}

#[cfg(feature = "http")]
fn check_http_condition(attr_str: String) -> (bool, String) {
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

/// Run test case when the example running and the http service exist.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(http);
/// #[test_with::module]
/// mod http {
///     #[test_with::runtime_http(httpbin.org)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_http(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "http"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_http(attr: TokenStream, stream: TokenStream) -> TokenStream {
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
    quote::quote! {
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

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when the https service exist.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // https server exists
///     #[test_with::https(www.rust-lang.org)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // There is no not.exist.com
///     #[test_with::https(not.exist.com)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "http")]
pub fn https(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_https_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_https_condition,
        )
    }
}

#[cfg(feature = "http")]
fn check_https_condition(attr_str: String) -> (bool, String) {
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

/// Run test case when the example running and the http service exist.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(http);
/// #[test_with::module]
/// mod http {
///     #[test_with::runtime_https(httpbin.org)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_https(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "http"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_https(attr: TokenStream, stream: TokenStream) -> TokenStream {
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
    quote::quote! {
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

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when the server online.
/// Please make sure the role of test case runner have capability to open socket
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // localhost is online
///     #[test_with::icmp(127.0.0.1)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // 193.194.195.196 is offline
///     #[test_with::icmp(193.194.195.196)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "icmp")]
pub fn icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_icmp_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_icmp_condition,
        )
    }
}

#[cfg(feature = "icmp")]
fn check_icmp_condition(attr_str: String) -> (bool, String) {
    let ips: Vec<&str> = attr_str.split(',').collect();
    let mut missing_ips = vec![];
    for ip in ips.iter() {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            if ping::ping(addr, None, None, None, None, None).is_err() {
                missing_ips.push(ip.to_string());
            }
        } else {
            abort_call_site!("ip address malformat")
        }
    }
    let ignore_msg = if missing_ips.len() == 1 {
        format!("because ip {} not response", missing_ips[0])
    } else {
        format!(
            "because following ip not response: \n{}\n",
            missing_ips.join(", ")
        )
    };
    (missing_ips.is_empty(), ignore_msg)
}

/// Run test case when the example running and the server online.
/// Please make sure the role of test case runner have capability to open socket
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(icmp);
/// #[test_with::module]
/// mod icmp {
///     // 193.194.195.196 is offline
///     #[test_with::runtime_icmp(193.194.195.196)]
///     fn test_ignored_with_non_existing_host() {
///         panic!("should be ignored with non existing host")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_icmp(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "icmp"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let ips: Vec<&str> = attr_str.split(',').collect();
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {

            let mut missing_ips = vec![];
            #(
                if libtest_with::ping::ping(#ips.parse().expect("ip address is invalid"), None, None, None, None, None).is_err() {
                    missing_ips.push(#ips);
                }
            )*
            match missing_ips.len() {
                0 => {
                    #ident();
                    Ok(())
                }
                ,
                1 => Err(
                    format!("{}because {} not response",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_ips[0]
                ).into()),
                _ => Err(
                    format!("{}because following ips not response: \n{}\n",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_ips.join(", ")
                ).into()),
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when socket connected
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Google DNS is online
///     #[test_with::tcp(8.8.8.8:53)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // 193.194.195.196 is offline
///     #[test_with::tcp(193.194.195.196)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn tcp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_tcp_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_tcp_condition,
        )
    }
}

fn check_tcp_condition(attr_str: String) -> (bool, String) {
    let sockets: Vec<&str> = attr_str.split(',').collect();
    let mut missing_sockets = vec![];
    for socket in sockets.iter() {
        if TcpStream::connect(socket).is_err() {
            missing_sockets.push(socket.to_string());
        }
    }
    let ignore_msg = if missing_sockets.len() == 1 {
        format!("because fail to connect socket {}", missing_sockets[0])
    } else {
        format!(
            "because follow sockets can not connect\n{}\n",
            missing_sockets.join(", ")
        )
    };
    (missing_sockets.is_empty(), ignore_msg)
}

/// Run test case when the example running and socket connected
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(tcp);
/// #[test_with::module]
/// mod tcp {
///     // Google DNS is online
///     #[test_with::runtime_tcp(8.8.8.8:53)]
///     fn test_works_with_DNS_server() {
///         assert!(true);
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_tcp(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_tcp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let sockets: Vec<&str> = attr_str.split(',').collect();
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {

            let mut missing_sockets = vec![];
            #(
                if std::net::TcpStream::connect(#sockets).is_err() {
                    missing_sockets.push(#sockets);
                }
            )*
            match missing_sockets.len() {
                0 => {
                    #ident();
                    Ok(())
                },
                1 => Err(
                    format!("{}because {} not response",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_sockets[0]
                ).into()),
                _ => Err(
                    format!("{}because following sockets not response: \n{}\n",
                            libtest_with::RUNTIME_IGNORE_PREFIX, missing_sockets.join(", ")
                ).into()),
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when runner is root
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with root account
///     #[test_with::root()]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(all(feature = "user", not(target_os = "windows")))]
pub fn root(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_root_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_root_condition,
        )
    }
}

#[cfg(all(feature = "user", not(target_os = "windows")))]
fn check_root_condition(_attr_str: String) -> (bool, String) {
    let current_user_id = uzers::get_current_uid();
    (
        current_user_id == 0,
        "because this case should run with root".into(),
    )
}

/// Run test case when runner is root
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(user);
/// #[test_with::module]
/// mod user {
///     // Google DNS is online
///     #[test_with::runtime_root()]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_root(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "user", not(target_os = "windows")))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_root(_attr: TokenStream, stream: TokenStream) -> TokenStream {
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            if 0 == libtest_with::users::get_current_uid() {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because this case should run with root", libtest_with::RUNTIME_IGNORE_PREFIX).into())
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when runner in group
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with group avengers
///     #[test_with::group(avengers)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(all(feature = "user", not(target_os = "windows")))]
pub fn group(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_group_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_group_condition,
        )
    }
}

#[cfg(feature = "user")]
#[cfg(all(feature = "user", not(target_os = "windows")))]
fn check_group_condition(group_name: String) -> (bool, String) {
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

/// Run test case when runner in group
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(user);
/// #[test_with::module]
/// mod user {
///     // Only works with group avengers
///     #[test_with::runtime_group(avengers)]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_group(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "user", not(target_os = "windows")))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_group(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    quote::quote! {

        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let current_user_id = libtest_with::users::get_current_uid();
            let in_group = match libtest_with::users::get_user_by_uid(current_user_id) {
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
                Ok(())
            } else {
                Err(format!("{}because this case should run user in group {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #group_name).into())
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when runner is specific user
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with user
///     #[test_with::user(spider)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(all(feature = "user", not(target_os = "windows")))]
pub fn user(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_user_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_user_condition,
        )
    }
}

#[cfg(feature = "user")]
#[cfg(all(feature = "user", not(target_os = "windows")))]
fn check_user_condition(user_name: String) -> (bool, String) {
    let is_user = match uzers::get_current_username() {
        Some(uname) => uname.to_string_lossy() == user_name,
        None => false,
    };
    (
        is_user,
        format!("because this case should run with user {}", user_name),
    )
}

/// Run test case when runner is specific user
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(user);
/// #[test_with::module]
/// mod user {
///     // Only works with user
///     #[test_with::runtime_user(spider)]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_user(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "user", not(target_os = "windows")))]
#[proc_macro_attribute]
#[proc_macro_error]
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

    quote::quote! {

        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let is_user = match libtest_with::users::get_current_username() {
                Some(uname) => uname.to_string_lossy() == #user_name,
                None => false,
            };

            if is_user {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because this case should run with user {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #user_name).into())
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

/// Run test case when memory size enough
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with enough memory size
///     #[test_with::mem(100GB)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "resource")]
pub fn mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_mem_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_mem_condition,
        )
    }
}

#[cfg(feature = "resource")]
fn check_mem_condition(mem_size_str: String) -> (bool, String) {
    let sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_swap()),
    );
    let mem_size = match byte_unit::Byte::parse_str(format!("{} B", sys.total_memory()), false) {
        Ok(b) => b,
        Err(_) => abort_call_site!("memory size description is not correct"),
    };
    let mem_size_limitation = match byte_unit::Byte::parse_str(&mem_size_str, true) {
        Ok(b) => b,
        Err(_) => abort_call_site!("system memory size can not get"),
    };
    (
        mem_size >= mem_size_limitation,
        format!("because the memory less than {}", mem_size_str),
    )
}

/// Run test case when the example running and memory size enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough memory size
///     #[test_with::runtime_mem(100GB)]
///     fn test_ignored_mem_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_mem(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let mem_limitation_str = attr.to_string().replace(' ', "");
    if byte_unit::Byte::parse_str(&mem_limitation_str, true).is_err() {
        abort_call_site!("memory size description is not correct")
    }

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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let sys = libtest_with::sysinfo::System::new_with_specifics(
                libtest_with::sysinfo::RefreshKind::new().with_memory(libtest_with::sysinfo::MemoryRefreshKind::new().with_ram()),
            );
            let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_memory()), false) {
                Ok(b) => b,
                Err(_) => panic!("system memory size can not get"),
            };
            let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
            if  mem_size >= mem_size_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the memory less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when the example running and free memory size enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough free memory size
///     #[test_with::runtime_free_mem(100GB)]
///     fn test_ignored_free_mem_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_free_mem(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_free_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let mem_limitation_str = attr.to_string().replace(' ', "");
    if byte_unit::Byte::parse_str(&mem_limitation_str, true).is_err() {
        abort_call_site!("memory size description is not correct")
    }

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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let sys = libtest_with::sysinfo::System::new_with_specifics(
                libtest_with::sysinfo::RefreshKind::new().with_memory(libtest_with::sysinfo::MemoryRefreshKind::new().with_ram()),
            );
            let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_memory()), false) {
                Ok(b) => b,
                Err(_) => panic!("system memory size can not get"),
            };
            let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
            if  mem_size >= mem_size_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the memory less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when the example running and available memory size enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough available memory size
///     #[test_with::runtime_available_mem(100GB)]
///     fn test_ignored_available_mem_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_available_mem(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_available_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let mem_limitation_str = attr.to_string().replace(' ', "");
    if byte_unit::Byte::parse_str(&mem_limitation_str, true).is_err() {
        abort_call_site!("memory size description is not correct")
    }

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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let sys = libtest_with::sysinfo::System::new_with_specifics(
                libtest_with::sysinfo::RefreshKind::new().with_memory(libtest_with::sysinfo::MemoryRefreshKind::new().with_ram()),
            );
            let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.available_memory()), false) {
                Ok(b) => b,
                Err(_) => panic!("system memory size can not get"),
            };
            let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
            if  mem_size >= mem_size_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the memory less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when swap size enough
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with enough swap size
///     #[test_with::swap(100GB)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "resource")]
pub fn swap(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_swap_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_swap_condition,
        )
    }
}

#[cfg(feature = "resource")]
fn check_swap_condition(swap_size_str: String) -> (bool, String) {
    let sys = sysinfo::System::new_with_specifics(
        sysinfo::RefreshKind::nothing()
            .with_memory(sysinfo::MemoryRefreshKind::nothing().with_swap()),
    );
    let swap_size = match byte_unit::Byte::parse_str(format!("{} B", sys.total_swap()), false) {
        Ok(b) => b,
        Err(_) => abort_call_site!("Swap size description is not correct"),
    };
    let swap_size_limitation = match byte_unit::Byte::parse_str(&swap_size_str, true) {
        Ok(b) => b,
        Err(_) => abort_call_site!("Can not get system swap size"),
    };
    (
        swap_size >= swap_size_limitation,
        format!("because the swap less than {}", swap_size_str),
    )
}

/// Run test case when the example running and swap enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough swap size
///     #[test_with::runtime_swap(100GB)]
///     fn test_ignored_swap_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_swap(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_swap(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let swap_limitation_str = attr.to_string().replace(' ', "");
    if byte_unit::Byte::parse_str(&swap_limitation_str, true).is_err() {
        abort_call_site!("swap size description is not correct")
    }

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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let sys = libtest_with::sysinfo::System::new_with_specifics(
                libtest_with::sysinfo::RefreshKind::new().with_memory(libtest_with::sysinfo::MemoryRefreshKind::new().with_swap()),
            );
            let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_swap()), false) {
                Ok(b) => b,
                Err(_) => panic!("system swap size can not get"),
            };
            let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
            if  swap_size >= swap_size_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the swap less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #swap_limitation_str).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when the example running and free swap enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough free swap size
///     #[test_with::runtime_free_swap(100GB)]
///     fn test_ignored_free_swap_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_free_swap(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_free_swap(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let swap_limitation_str = attr.to_string().replace(' ', "");
    if byte_unit::Byte::parse_str(&swap_limitation_str, true).is_err() {
        abort_call_site!("swap size description is not correct")
    }

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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            let sys = libtest_with::sysinfo::System::new_with_specifics(
                libtest_with::sysinfo::RefreshKind::new().with_memory(libtest_with::sysinfo::MemoryRefreshKind::new().with_swap()),
            );
            let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_swap()), false) {
                Ok(b) => b,
                Err(_) => panic!("system swap size can not get"),
            };
            let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
            if  swap_size >= swap_size_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the swap less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #swap_limitation_str).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when cpu core enough
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with enough cpu core
///     #[test_with::cpu_core(32)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "resource")]
pub fn cpu_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_cpu_core_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_cpu_core_condition,
        )
    }
}

#[cfg(feature = "resource")]
fn check_cpu_core_condition(core_limitation_str: String) -> (bool, String) {
    (
        match core_limitation_str.parse::<usize>() {
            Ok(c) => num_cpus::get() >= c,
            Err(_) => abort_call_site!("core limitation is incorrect"),
        },
        format!("because the cpu core less than {}", core_limitation_str),
    )
}

/// Run test case when cpu core enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough cpu core
///     #[test_with::runtime_cpu_core(32)]
///     fn test_ignored_core_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_cpu_core(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_cpu_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let core_limitation = match attr_str.parse::<usize>() {
        Ok(c) => c,
        Err(_) => abort_call_site!("core limitation is incorrect"),
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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            if libtest_with::num_cpus::get() >= #core_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the cpu core less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when physical cpu core enough
///
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // Only works with enough cpu core
///     #[test_with::phy_core(32)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "resource")]
pub fn phy_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_cpu_core_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_phy_core_condition,
        )
    }
}

#[cfg(feature = "resource")]
fn check_phy_core_condition(core_limitation_str: String) -> (bool, String) {
    (
        match core_limitation_str.parse::<usize>() {
            Ok(c) => num_cpus::get_physical() >= c,
            Err(_) => abort_call_site!("physical core limitation is incorrect"),
        },
        format!(
            "because the physical cpu core less than {}",
            core_limitation_str
        ),
    )
}

/// Run test case when physical core enough
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(resource);
/// #[test_with::module]
/// mod resource {
///     // Only works with enough physical cpu core
///     #[test_with::runtime_phy_cpu_core(32)]
///     fn test_ignored_phy_core_not_enough() {
///         panic!("should be ignored")
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_phy_cpu_core(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "resource"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_phy_cpu_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let core_limitation = match attr_str.parse::<usize>() {
        Ok(c) => c,
        Err(_) => abort_call_site!("physical core limitation is incorrect"),
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

    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            if libtest_with::num_cpus::get_physical() >= #core_limitation {
                #ident();
                Ok(())
            } else {
                Err(format!("{}because the physical cpu core less than {}",
                        libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
            }
        }

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Run test case when the executables exist.
/// ```
/// #[cfg(test)]
/// mod tests {
///     // `pwd` executable command exists
///     #[test_with::executable(pwd)]
///     #[test]
///     fn test_executable() {
///         assert!(true);
///     }
///
///     // `/bin/sh` executable exists
///     #[test_with::executable(/bin/sh)]
///     #[test]
///     fn test_executable_with_path() {
///         assert!(true);
///     }
///
///     // `non` does not exist
///     #[test_with::executable(non)]
///     #[test]
///     fn test_non_existing_executable() {
///         panic!("should be ignored")
///     }
///
///     // `pwd` and `ls` exist
///     #[test_with::executable(pwd, ls)]
///     #[test]
///     fn test_executables_too() {
///         assert!(true);
///     }
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
#[cfg(feature = "executable")]
pub fn executable(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_executable_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_executable_condition,
        )
    }
}

#[cfg(feature = "executable")]
fn check_executable_condition(attr_str: String) -> (bool, String) {
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

/// Run test case when the executable existing
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(exe);
/// #[test_with::module]
/// mod exe {
///     // `/bin/sh` executable exists
///     #[test_with::runtime_executable(/bin/sh)]
///     fn test_executable_with_path() {
///         assert!(true);
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_executable(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(all(feature = "runtime", feature = "executable"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_executable(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let executables: Vec<&str> = attr_str.split(',').collect();
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

    quote::quote! {
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

        #(#attrs)*
        #vis #sig #block

    }
    .into()
}

/// Provide a test runner and test on each module
///```rust
/// // example/run-test.rs
///
/// test_with::runner!(module1, module2);
/// #[test_with::module]
/// mod module1 {
///     #[test_with::runtime_env(PWD)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
///
/// #[test_with::module]
/// mod module2 {
///     #[test_with::runtime_env(PWD)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
///```
#[cfg(not(feature = "runtime"))]
#[proc_macro]
pub fn runner(_input: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro]
pub fn runner(input: TokenStream) -> TokenStream {
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

/// Help each function with `#[test_with::runtime_*]` in the module can register to run
/// Also you can set up a mock instance for all of the test in the module
///
/// ```rust
///  // example/run-test.rs
///
///  test_with::runner!(module1, module2);
///  #[test_with::module]
///  mod module1 {
///      #[test_with::runtime_env(PWD)]
///      fn test_works() {
///          assert!(true);
///      }
///  }
///
///  #[test_with::module]
///  mod module2 {
///      #[test_with::runtime_env(PWD)]
///      fn test_works() {
///          assert!(true);
///      }
///  }
/// ```
/// You can set up mock with a public struct named `TestEnv` inside the module, or a public type
/// named `TestEnv` inside the module.  And the type or struct should have a Default trait for
/// initialize the mock instance.
/// ```rust
/// use std::ops::Drop;
/// use std::process::{Child, Command};
///
/// test_with::runner!(net);
///
/// #[test_with::module]
/// mod net {
///     pub struct TestEnv {
///         p: Child,
///     }
///
///     impl Default for TestEnv {
///         fn default() -> TestEnv {
///             let p = Command::new("python")
///                 .args(["-m", "http.server"])
///                 .spawn()
///                 .expect("failed to execute child");
///             let mut count = 0;
///             while count < 3 {
///                 if libtest_with::reqwest::blocking::get("http://127.0.0.1:8000").is_ok() {
///                     break;
///                 }
///                 std::thread::sleep(std::time::Duration::from_secs(1));
///                 count += 1;
///             }
///             TestEnv { p }
///         }
///     }
///
///     impl Drop for TestEnv {
///         fn drop(&mut self) {
///             self.p.kill().expect("fail to kill python http.server");
///         }
///     }
///
///     #[test_with::runtime_http(127.0.0.1:8000)]
///     fn test_with_environment() {
///         assert!(true);
///     }
/// }
///
/// ```
/// or you can write mock struct in other place and just pass by type.
/// ```rust
/// use std::ops::Drop;
/// use std::process::{Child, Command};
///
/// test_with::runner!(net);
///
/// pub struct Moc {
///     p: Child,
/// }
///
/// impl Default for Moc {
///     fn default() -> Moc {
///         let p = Command::new("python")
///             .args(["-m", "http.server"])
///             .spawn()
///             .expect("failed to execute child");
///         let mut count = 0;
///         while count < 3 {
///             if libtest_with::reqwest::blocking::get("http://127.0.0.1:8000").is_ok() {
///                 break;
///             }
///             std::thread::sleep(std::time::Duration::from_secs(1));
///             count += 1;
///         }
///         Moc { p }
///     }
/// }
///
/// impl Drop for Moc {
///     fn drop(&mut self) {
///         self.p.kill().expect("fail to kill python http.server");
///     }
/// }
///
/// #[test_with::module]
/// mod net {
///     pub type TestEnv = super::Moc;
///
///     #[test_with::runtime_http(127.0.0.1:8000)]
///     fn test_with_environment() {
///         assert!(true);
///     }
/// }
/// ```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn module(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn module(_attr: TokenStream, stream: TokenStream) -> TokenStream {
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
            let test_names: Vec<String> = content
                .iter()
                .filter_map(|c| match c {
                    Item::Fn(ItemFn {
                        sig: syn::Signature { ident, .. },
                        attrs,
                        ..
                    }) => match crate::utils::test_with_attrs(&attrs) {
                        (true, true, _) => abort_call_site!(
                            "should not use #[test] for method in `#[test_with::module]`"
                        ),
                        (_, true, false) => abort_call_site!(
                            "use `#[test_with::runtime_*]` for method in `#[test_with::module]`"
                        ),
                        (false, true, true) => Some(ident.to_string()),
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
            let check_names: Vec<syn::Ident> = test_names
                .iter()
                .map(|c| {
                    syn::Ident::new(
                        &format!("_check_{}", c.to_string()),
                        proc_macro2::Span::call_site(),
                    )
                })
                .collect();
            if let Some(test_env_type) = test_env_type {
                quote::quote! {
                    #(#attrs)*
                    #vis #mod_token #ident {
                        use super::*;
                        pub fn _runtime_tests() -> (Option<#test_env_type>, Vec<libtest_with::Trial>) {
                            use libtest_with::Trial;
                            (
                                Some(#test_env_type::default()),
                                vec![
                                    #(Trial::test(#test_names, #check_names),)*
                                ]
                            )
                        }
                        #(#content)*
                    }
                }
                .into()
            } else {
                quote::quote! {
                    #(#attrs)*
                    #vis #mod_token #ident {
                        use super::*;
                        pub fn _runtime_tests() -> (Option<()>, Vec<libtest_with::Trial>) {
                            use libtest_with::Trial;
                            (
                                None,
                                vec![
                                    #(Trial::test(#test_names, #check_names),)*
                                ]
                            )
                        }
                        #(#content)*
                    }
                }
                .into()
            }
        }
    } else {
        abort_call_site!("should use on mod with context")
    }
}

/// Ignore test case when function return some reason
/// The function should be `fn() -> Option<String>`
/// ```
/// test_with::runner!(custom_mod);
///
/// fn something_happened() -> Option<String> {
///     Some("because something happened".to_string())
/// }
///
/// #[test_with::module]
/// mod custom_mod {
/// #[test_with::runtime_ignore_if(something_happened)]
/// fn test_ignored() {
///     assert!(false);
///     }
/// }
/// ```
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_ignore_if(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_ignore_if(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let ignore_function = syn::Ident::new(
        &attr.to_string().replace(' ', ""),
        proc_macro2::Span::call_site(),
    );
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {
            if let Some(msg) = #ignore_function() {
                Err(format!("{}{msg}", libtest_with::RUNTIME_IGNORE_PREFIX).into())
            } else {
                #ident();
                Ok(())
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::{check_env_condition, check_no_env_condition};

    mod env_macro {
        use super::*;

        #[test]
        fn single_env_var_should_be_not_set() {
            //* Given
            let env_var = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = env_var.to_string();

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(ignore_msg.contains(env_var));
        }

        #[test]
        fn multiple_env_vars_should_not_be_set() {
            //* Given
            let env_var1 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";
            let env_var2 = "ANOTHER_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}", env_var1, env_var2);

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(ignore_msg.contains(env_var1));
            assert!(ignore_msg.contains(env_var2));
        }

        #[test]
        fn single_env_var_should_be_set() {
            //* Given
            let env_var = "PATH";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = env_var.to_string();

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(!ignore_msg.contains(env_var));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars containing spaces and newlines.
        ///
        /// ```no_run
        /// #[test_with::env(
        ///   PATH,
        ///   HOME
        /// )]
        /// #[test]
        /// fn some_test() {}
        #[test]
        fn multiple_env_vars_should_be_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "HOME";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("\t{},\n\t{}\n", env_var1, env_var2);

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(!ignore_msg.contains(env_var1));
            assert!(!ignore_msg.contains(env_var2));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars and one of them is not set.
        #[test]
        fn multiple_env_vars_but_one_is_not_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "HOME";
            let env_var3 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}, {}", env_var1, env_var2, env_var3);

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(!ignore_msg.contains(env_var1));
            assert!(!ignore_msg.contains(env_var2));
            assert!(ignore_msg.contains(env_var3));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars and various of them are not set.
        #[test]
        fn multiple_env_vars_and_various_not_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";
            let env_var3 = "ANOTHER_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}, {}", env_var1, env_var2, env_var3);

            //* When
            let (is_ok, ignore_msg) = check_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the missing env var names
            assert!(!ignore_msg.contains(env_var1));
            assert!(ignore_msg.contains(env_var2));
            assert!(ignore_msg.contains(env_var3));
        }
    }

    mod no_env_macro {
        use super::*;

        #[test]
        fn single_env_var_not_set() {
            //* Given
            let env_var = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = env_var.to_string();

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(!ignore_msg.contains(env_var));
        }

        #[test]
        fn multiple_env_vars_not_set() {
            //* Given
            let env_var1 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";
            let env_var2 = "ANOTHER_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}", env_var1, env_var2);

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(!ignore_msg.contains(env_var1));
            assert!(!ignore_msg.contains(env_var2));
        }

        #[test]
        fn single_env_var_set() {
            //* Given
            let env_var = "PATH";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = env_var.to_string();

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(ignore_msg.contains(env_var));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars containing spaces and newlines.
        ///
        /// ```no_run
        /// #[test_with::no_env(
        ///   PATH,
        ///   HOME
        /// )]
        /// #[test]
        /// fn some_test() {}
        #[test]
        fn multiple_env_vars_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "HOME";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("\t{},\n\t{}\n", env_var1, env_var2);

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(ignore_msg.contains(env_var1));
            assert!(ignore_msg.contains(env_var2));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars and one of them is set.
        #[test]
        fn multiple_env_vars_but_one_is_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";
            let env_var3 = "ANOTHER_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}, {}", env_var1, env_var2, env_var3);

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(ignore_msg.contains(env_var1));
            assert!(!ignore_msg.contains(env_var2));
            assert!(!ignore_msg.contains(env_var3));
        }

        /// Test the `test_with::env(<attr_str>)` macro should parse the attribute string correctly
        /// when the attribute string contains multiple env vars and various of them are set.
        #[test]
        fn multiple_env_vars_and_various_are_set() {
            //* Given
            let env_var1 = "PATH";
            let env_var2 = "HOME";
            let env_var3 = "A_RIDICULOUS_ENV_VAR_NAME_THAT_SHOULD_NOT_BE_SET";

            // The `test_with::env(<attr_str>)` macro arguments
            let attr_str = format!("{}, {}, {}", env_var1, env_var2, env_var3);

            //* When
            let (is_ok, ignore_msg) = check_no_env_condition(attr_str);

            //* Then
            // Assert if the test should be ignored
            assert!(!is_ok);
            // Assert the ignore message should contain only the found env var names
            assert!(ignore_msg.contains(env_var1));
            assert!(ignore_msg.contains(env_var2));
            assert!(!ignore_msg.contains(env_var3));
        }
    }
}

/// Run test case one by one when the lock is acquired
/// It will automatically implement a file lock for the test case to prevent it run in the same
/// time. Also, you can pass the second parameter to specific the waiting seconds, default will be
/// 60 seconds.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // `LOCK` is file based lock to prevent test1 an test2 run at the same time
///     #[test_with::lock(LOCK)]
///     #[test]
///     fn test_1() {
///         assert!(true);
///     }
///
///     // `LOCK` is file based lock to prevent test1 an test2 run at the same time
///     #[test_with::lock(LOCK)]
///     #[test]
///     fn test_2() {
///         assert!(true);
///     }
///
///     // `ANOTHER_LOCK` is file based lock to prevent test3 an test4 run at the same time with 3 sec
///     // waiting time.
///     #[test_with::lock(ANOTHER_LOCK, 3)]
///     fn test_3() {
///         assert!(true);
///     }
///
///     // `ANOTHER_LOCK` is file based lock to prevent test3 an test4 run at the same time with 3 sec
///     // waiting time.
///     #[test_with::lock(ANOTHER_LOCK, 3)]
///     fn test_4() {
///         assert!(true);
///     }
///
/// }
#[proc_macro_attribute]
#[proc_macro_error]
pub fn lock(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        abort_call_site!("#[test_with::lock] only works with fn")
    } else {
        lock_macro(attr, parse_macro_input!(stream as ItemFn))
    }
}

/// Run test case when the timezone is expected.
/// ```
/// #[cfg(test)]
/// mod tests {
///
///     // 0 means UTC
///     #[test_with::timezone(0)]
///     #[test]
///     fn test_works() {
///         assert!(true);
///     }
///
///     // UTC is GMT+0
///     #[test_with::timezone(UTC)]
///     #[test]
///     fn test_works_too() {
///         assert!(true);
///     }
///
///     // +8 means GMT+8
///     #[test_with::timezone(+8)]
///     #[test]
///     fn test_ignored() {
///         panic!("should be ignored")
///     }
///
///     // HKT GMT+8
///     #[test_with::timezone(HKT)]
///     #[test]
///     fn test_ignored_too() {
///         panic!("should be ignored")
///     }
/// }
/// ```
#[cfg(feature = "timezone")]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn timezone(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            check_tz_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            check_tz_condition,
        )
    }
}

#[cfg(feature = "timezone")]
fn check_timezone(attr_str: &String) -> (bool, Vec<&str>) {
    let mut incorrect_tzs = vec![];
    let mut match_tz = false;
    let current_tz = chrono::Local::now().offset().local_minus_utc() / 60;

    for tz in attr_str.split(',') {
        let parsed_tz = match tz {
            "NZDT" => Ok(13 * 60),
            "NZST" => Ok(12 * 60),
            "AEDT" => Ok(11 * 60),
            "ACDT" => Ok(10 * 60 + 30),
            "AEST" => Ok(10 * 60),
            "ACST" => Ok(9 * 60 + 30),
            "KST" | "JST" => Ok(9 * 60),
            "HKT" | "WITA" | "AWST" => Ok(8 * 60),
            "PST" => abort_call_site!("PST can be GMT+8 or GMT-8, please use +8 or -8 instead"),
            "WIB" => Ok(7 * 60),
            "CST" => abort_call_site!("PST can be GMT+8 or GMT-6, please use +8 or -6 instead"),
            "5.5" | "+5.5" => Ok(5 * 60 + 30),
            "IST" => abort_call_site!(
                "IST can be GMT+5.5, GMT+2 or GMT+1, please use +5.5, 2 or 1 instead"
            ),
            "PKT" => Ok(5 * 60),
            "EAT" | "EEST" | "IDT" | "MSK" => Ok(3 * 60),
            "CAT" | "EET" | "CEST" | "SAST" => Ok(2 * 60),
            "CET" | "WAT" | "WEST" | "BST" => Ok(1 * 60),
            "UTC" | "GMT" | "WET" => Ok(0),
            "NDT" | "-2.5" => Ok(-2 * 60 - 30),
            "NST" | "-3.5" => Ok(-3 * 60 - 30),
            "ADT" => Ok(-3 * 60),
            "AST" | "EDT" => Ok(-4 * 60),
            "EST" | "CDT" => Ok(-5 * 60),
            "MDT" => Ok(-6 * 60),
            "MST" | "PDT" => Ok(-7 * 60),
            "AKDT" => Ok(-8 * 60),
            "HDT" | "AKST" => Ok(-9 * 60),
            "HST" => Ok(-10 * 60),
            _ => tz.parse::<i32>().map(|tz| tz * 60),
        };
        if let Ok(parsed_tz) = parsed_tz {
            match_tz |= current_tz == parsed_tz;
        } else {
            incorrect_tzs.push(tz);
        }
    }
    (match_tz, incorrect_tzs)
}

#[cfg(feature = "timezone")]
fn check_tz_condition(attr_str: String) -> (bool, String) {
    let (match_tz, incorrect_tzs) = check_timezone(&attr_str);

    // Generate ignore message
    if incorrect_tzs.len() == 1 {
        (
            false,
            format!("because timezone {} is incorrect", incorrect_tzs[0]),
        )
    } else if incorrect_tzs.len() > 1 {
        (
            false,
            format!(
                "because following timezones are incorrect:\n{}\n",
                incorrect_tzs.join(", ")
            ),
        )
    } else if match_tz {
        (true, String::new())
    } else {
        (
            false,
            format!(
                "because the test case not run in following timezone:\n{}\n",
                attr_str
            ),
        )
    }
}

/// Run test case when the example running within specific timzones.
///```rust
/// // write as example in examples/*rs
/// test_with::runner!(timezone);
/// #[test_with::module]
/// mod timezone {
///     // 0 means UTC timezone
///     #[test_with::runtime_timezone(0)]
///     fn test_works() {
///         assert!(true);
///     }
/// }
#[cfg(not(feature = "runtime"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_timezone(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "timezone"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_timezone(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string();
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
    quote::quote! {
        fn #check_ident() -> Result<(), libtest_with::Failed> {

            let mut incorrect_tzs = vec![];
            let mut match_tz = false;
            let current_tz = libtest_with::chrono::Local::now().offset().local_minus_utc() / 60;
            for tz in #attr_str.split(',') {
                if let Ok(parsed_tz) = tz.parse::<i32>() {
                    match_tz |= current_tz == parsed_tz;
                } else {
                    incorrect_tzs.push(tz);
                }
            }

            if match_tz && incorrect_tzs.is_empty() {
                    #ident();
                    Ok(())
            } else if incorrect_tzs.len() == 1 {
                Err(
                    format!("{}because timezone {} is incorrect",
                            libtest_with::RUNTIME_IGNORE_PREFIX, incorrect_tzs[0]
                ).into())
            } else if incorrect_tzs.len() > 1 {
                Err(
                    format!("{}because following timezones are incorrect:\n{:?}\n",
                            libtest_with::RUNTIME_IGNORE_PREFIX, incorrect_tzs
                ).into())
            } else {
                Err(
                    format!(
                    "{}because the test case not run in following timezone:\n{}\n",
                    libtest_with::RUNTIME_IGNORE_PREFIX,
                    #attr_str
                ).into())
            }
        }

        #(#attrs)*
        #vis #sig #block
    }
    .into()
}
