//! `test_with` provides [macro@env], [macro@file], [macro@path], [macro@http], [macro@https],
//! [macro@icmp], [macro@tcp] macros to help you run test case only with the condition is
//! fulfilled.  If the `#[test]` is absent for the test case, `#[test_with]` will add it to the
//! test case automatically.
use std::{
    fs::metadata,
    net::{IpAddr, Ipv4Addr, TcpStream},
    path::Path,
};

use crate::utils::has_test_attr;
#[cfg(feature = "ign-msg")]
use crate::utils::rewrite_fn_name_with_msg;
use proc_macro::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use regex::Regex;
use syn::{parse_macro_input, ItemFn};

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
#[proc_macro_attribute]
pub fn env(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let var_names: Vec<&str> = attr_str.split(',').collect();
    let mut all_var_exist = true;
    let mut ignore_msg = "because following variable not found:".to_string();
    for var in var_names.iter() {
        if std::env::var(var).is_err() {
            all_var_exist = false;
            ignore_msg.push(' ');
            ignore_msg.push_str(var);
        }
    }
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
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
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
pub fn file(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let files: Vec<&str> = attr_str.split(',').collect();
    let mut all_file_exist = true;
    let mut ignore_msg = "because following file not found:".to_string();
    for file in files.iter() {
        if !Path::new(file).is_file() {
            all_file_exist = false;
            ignore_msg.push('\n');
            ignore_msg.push_str(file);
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_file_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_file_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
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
pub fn path(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let paths: Vec<&str> = attr_str.split(',').collect();
    let mut all_path_exist = true;
    let mut ignore_msg = "because following path not found:".to_string();
    for path in paths.iter() {
        if metadata(path).is_err() {
            all_path_exist = false;
            ignore_msg.push('\n');
            ignore_msg.push_str(path);
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_path_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_path_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
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
pub fn http(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let links: Vec<&str> = attr_str.split(',').collect();
    let mut all_link_exist = true;
    let mut ignore_msg = "because following link not found:".to_string();
    let client = reqwest::blocking::Client::new();
    for link in links.iter() {
        if client.head(&format!("http://{}", link)).send().is_err() {
            all_link_exist = false;
            ignore_msg.push('\n');
            ignore_msg.push_str(link);
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_link_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_link_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
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
pub fn https(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let links: Vec<&str> = attr_str.split(',').collect();
    let mut all_link_exist = true;
    let mut ignore_msg = "because following link not found:".to_string();
    let client = reqwest::blocking::Client::new();
    for link in links.iter() {
        if client.head(&format!("https://{}", link)).send().is_err() {
            all_link_exist = false;
            ignore_msg.push('\n');
            ignore_msg.push_str(link);
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_link_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_link_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
}

fn parse_ipv4_addre(cap: regex::Captures) -> Result<IpAddr, std::num::ParseIntError> {
    Ok(IpAddr::V4(Ipv4Addr::new(
        cap[1].parse::<u8>()?,
        cap[2].parse::<u8>()?,
        cap[3].parse::<u8>()?,
        cap[4].parse::<u8>()?,
    )))
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
pub fn icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let ipv4s: Vec<&str> = attr_str.split(',').collect();
    let ipv4_re = unsafe { Regex::new(r"^(\d+)\.(\d+)\.(\d+)\.(\d+)$").unwrap_unchecked() };
    let mut all_ipv4_exist = true;
    let mut ignore_msg = "because following ipv4 not found:".to_string();
    for ipv4 in ipv4s.iter() {
        if let Some(cap) = ipv4_re.captures(ipv4) {
            if let Ok(addr_v4) = parse_ipv4_addre(cap) {
                if ping::ping(addr_v4, None, None, None, None, None).is_err() {
                    all_ipv4_exist = false;
                    ignore_msg.push('\n');
                    ignore_msg.push_str(ipv4);
                }
            } else {
                abort_call_site!("ip v4 address malformat, digit not u8")
            }
        } else {
            abort_call_site!("ip v4 address malformat")
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_ipv4_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_ipv4_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
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
pub fn tcp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as ItemFn);
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
    let attr_str = attr.to_string().replace(" ", "");
    let sockets: Vec<&str> = attr_str.split(',').collect();
    let mut all_socket_exist = true;
    let mut ignore_msg = "because following socket not found:".to_string();
    for socket in sockets.iter() {
        if TcpStream::connect(socket).is_err() {
            all_socket_exist = false;
            ignore_msg.push('\n');
            ignore_msg.push_str(socket);
        }
    }
    let has_test = has_test_attr(&attrs);
    return if all_socket_exist && has_test {
        quote! {
            #(#attrs)*
            #vis #sig #block
        }
        .into()
    } else if all_socket_exist {
        quote! {
            #(#attrs)*
            #[test]
            #vis #sig #block
        }
        .into()
    } else if has_test {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    } else {
        #[cfg(feature = "ign-msg")]
        rewrite_fn_name_with_msg(&mut sig, &ignore_msg);
        quote! {
           #(#attrs)*
           #[test]
           #[ignore = #ignore_msg ]
           #vis #sig #block
        }
        .into()
    };
}
