//! `test_with` provides [macro@env], [macro@file], [macro@path], [macro@http], [macro@https],
//! [macro@icmp], [macro@tcp], [macro@root], [macro@group], [macro@user], [macro@mem], [macro@swap],
//! [macro@cpu_core], [macro@phy_core], [macro@executable] macros to help you run test case only
//! with the condition is fulfilled.  If the `#[test]` is absent for the test case, `#[test_with]`
//! will add it to the test case automatically.
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

use std::{fs::metadata, path::Path};

#[cfg(feature = "icmp")]
use std::net::IpAddr;
use std::net::TcpStream;

use proc_macro::TokenStream;
#[cfg(any(feature = "resource", feature = "icmp"))]
use proc_macro_error::abort_call_site;
use syn::{parse_macro_input, ItemFn, ItemMod};
#[cfg(feature = "resource")]
use sysinfo::SystemExt;
#[cfg(feature = "executable")]
use which::which;

use crate::utils::{fn_macro, is_module, mod_macro};

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
    let var_names: Vec<&str> = attr_str.split(',').collect();
    let mut missing_vars = vec![];
    for var in var_names.iter() {
        if std::env::var(var).is_err() {
            missing_vars.push(var.to_string());
        }
    }
    let ignore_msg = if missing_vars.len() == 1 {
        format!("because variable {} not found", missing_vars[0])
    } else {
        format!(
            "because following variables not found:\n{}\n",
            missing_vars.join(", ")
        )
    };
    (missing_vars.is_empty(), ignore_msg)
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
    let var_names: Vec<&str> = attr_str.split(',').collect();
    for var in var_names.iter() {
        if std::env::var(var).is_ok() {
            return (
                false,
                format!("because the environment with variable {var:} will ignore"),
            );
        }
    }
    (true, String::new())
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
    let current_user_id = users::get_current_uid();
    (
        current_user_id == 0,
        "because this case should run with root".into(),
    )
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
    let current_user_id = users::get_current_uid();

    let in_group = match users::get_user_by_uid(current_user_id) {
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
    let is_user = match users::get_current_username() {
        Some(uname) => uname.to_string_lossy() == user_name,
        None => false,
    };
    (
        is_user,
        format!("because this case should run with user {}", user_name),
    )
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
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let mem_size = match byte_unit::Byte::from_str(format!("{} B", sys.total_memory())) {
        Ok(b) => b,
        Err(_) => abort_call_site!("memory size description is not correct"),
    };
    let mem_size_limitation = match byte_unit::Byte::from_str(&mem_size_str) {
        Ok(b) => b,
        Err(_) => abort_call_site!("system memory size can not get"),
    };
    (
        mem_size >= mem_size_limitation,
        format!("because the memory less than {}", mem_size_str),
    )
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
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    let swap_size = match byte_unit::Byte::from_str(format!("{} B", sys.total_swap())) {
        Ok(b) => b,
        Err(_) => abort_call_site!("Swap size description is not correct"),
    };
    let swap_size_limitation = match byte_unit::Byte::from_str(&swap_size_str) {
        Ok(b) => b,
        Err(_) => abort_call_site!("Can not get system swap size"),
    };
    (
        swap_size >= swap_size_limitation,
        format!("because the swap less than {}", swap_size_str),
    )
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
