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
//! libtest-with = { version = "0.8.1-11", features = ["net", "resource", "user", "executable", "timezone"] }
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

use proc_macro::TokenStream;
use proc_macro_error2::abort_call_site;
use proc_macro_error2::proc_macro_error;
use syn::{parse_macro_input, ItemFn, ItemMod};

#[cfg(feature = "runtime")]
use syn::ReturnType;

use crate::utils::{fn_macro, is_module, lock_macro, mod_macro};

mod env;
#[cfg(feature = "executable")]
mod executable;
mod file;
#[cfg(feature = "http")]
mod http;
#[cfg(feature = "icmp")]
mod icmp;
#[cfg(feature = "resource")]
mod resource;
#[cfg(feature = "runtime")]
mod runtime;
mod socket;
#[cfg(feature = "timezone")]
mod timezone;
#[cfg(feature = "user")]
mod user;
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
            env::check_env_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            env::check_env_condition,
        )
    }
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
    env::runtime_env(attr, stream)
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
            env::check_no_env_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            env::check_no_env_condition,
        )
    }
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
    env::runtime_no_env(attr, stream)
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
            file::check_file_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            file::check_file_condition,
        )
    }
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
    file::runtime_file(attr, stream)
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
            file::check_path_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            file::check_path_condition,
        )
    }
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
    file::runtime_path(attr, stream)
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
            http::check_http_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            http::check_http_condition,
        )
    }
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
    http::runtime_http(attr, stream)
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
            http::check_https_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            http::check_https_condition,
        )
    }
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
#[cfg(all(not(feature = "runtime"), feature = "http"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_https(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "http"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_https(attr: TokenStream, stream: TokenStream) -> TokenStream {
    http::runtime_https(attr, stream)
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
            icmp::check_icmp_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            icmp::check_icmp_condition,
        )
    }
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
#[cfg(all(not(feature = "runtime"), feature = "icmp"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_icmp(_attr: TokenStream, _stream: TokenStream) -> TokenStream {
    panic!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "icmp"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    icmp::runtime_icmp(attr, stream)
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
            socket::check_tcp_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            socket::check_tcp_condition,
        )
    }
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
    socket::runtime_tcp(attr, stream)
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
#[cfg(all(feature = "user"))]
pub fn root(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            user::check_root_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            user::check_root_condition,
        )
    }
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
#[cfg(all(feature = "runtime", feature = "user"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_root(attr: TokenStream, stream: TokenStream) -> TokenStream {
    user::runtime_root(attr, stream)
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
#[cfg(all(feature = "user"))]
pub fn group(attr: TokenStream, stream: TokenStream) -> TokenStream {
    if is_module(&stream) {
        mod_macro(
            attr,
            parse_macro_input!(stream as ItemMod),
            user::check_group_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            user::check_group_condition,
        )
    }
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
#[cfg(all(feature = "runtime", feature = "user"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_group(attr: TokenStream, stream: TokenStream) -> TokenStream {
    user::runtime_group(attr, stream)
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
            user::check_user_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            user::check_user_condition,
        )
    }
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
#[cfg(all(feature = "runtime", feature = "user"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_user(attr: TokenStream, stream: TokenStream) -> TokenStream {
    user::runtime_user(attr, stream)
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
            resource::check_mem_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            resource::check_mem_condition,
        )
    }
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
    resource::runtime_mem(attr, stream)
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
    resource::runtime_free_mem(attr, stream)
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
    resource::runtime_available_mem(attr, stream)
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
            resource::check_swap_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            resource::check_swap_condition,
        )
    }
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_swap()),
                );
                let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_swap()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system swap size can not get"),
                };
                let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
                if  swap_size >= swap_size_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the swap less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #swap_limitation_str).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_swap()),
                );
                let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_swap()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system swap size can not get"),
                };
                let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
                if  swap_size >= swap_size_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the swap less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #swap_limitation_str).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_swap()),
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
        },
    };

    quote::quote! {
            #check_fn
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
    resource::runtime_free_swap(attr, stream)
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
            resource::check_cpu_core_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            resource::check_cpu_core_condition,
        )
    }
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
    resource::runtime_cpu_core(attr, stream)
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
            resource::check_cpu_core_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            resource::check_phy_core_condition,
        )
    }
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
    resource::runtime_phy_cpu_core(attr, stream)
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
///
///     // `non` or `ls` exist
///     #[test_with::executable(non || ls)]
///     #[test]
///     fn test_one_of_executables_exist() {
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
            executable::check_executable_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            executable::check_executable_condition,
        )
    }
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
    executable::runtime_executable(attr, stream)
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if let Some(msg) = #ignore_function() {
                    Err(format!("{}{msg}", libtest_with::RUNTIME_IGNORE_PREFIX).into())
                } else {
                    #ident().await;
                    Ok(())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if let Some(msg) = #ignore_function() {
                    Err(format!("{}{msg}", libtest_with::RUNTIME_IGNORE_PREFIX).into())
                } else {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                if let Some(msg) = #ignore_function() {
                    Err(format!("{}{msg}", libtest_with::RUNTIME_IGNORE_PREFIX).into())
                } else {
                    #ident();
                    Ok(())
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
            timezone::check_tz_condition,
        )
    } else {
        fn_macro(
            attr,
            parse_macro_input!(stream as ItemFn),
            timezone::check_tz_condition,
        )
    }
}

/// Run test case when the example running within specific timezones.
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
    abort_call_site!("should be used with runtime feature")
}

#[cfg(all(feature = "runtime", feature = "timezone"))]
#[proc_macro_attribute]
#[proc_macro_error]
pub fn runtime_timezone(attr: TokenStream, stream: TokenStream) -> TokenStream {
    timezone::runtime_timezone(attr, stream)
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
    abort_call_site!("should be used with runtime feature")
}
#[cfg(feature = "runtime")]
#[proc_macro]
pub fn runner(input: TokenStream) -> TokenStream {
    runtime::runner(input)
}

#[cfg(not(feature = "runtime"))]
#[proc_macro]
pub fn tokio_runner(_input: TokenStream) -> TokenStream {
    abort_call_site!("should be used with `runtime` feature")
}
#[cfg(feature = "runtime")]
#[proc_macro]
pub fn tokio_runner(input: TokenStream) -> TokenStream {
    runtime::tokio_runner(input)
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
pub fn module(attr: TokenStream, stream: TokenStream) -> TokenStream {
    runtime::module(attr, stream)
}
