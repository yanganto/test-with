use std::net::IpAddr;
use std::time::Duration;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

const PING_TIMEOUT: Duration = Duration::from_secs(4);

pub(crate) fn check_icmp_condition(attr_str: String) -> (bool, String) {
    let ips: Vec<&str> = attr_str.split(',').collect();
    let mut pinger = ping::Pinger::new();
    let mut missing_ips = vec![];
    for ip in ips.iter() {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            match pinger.ping(addr, PING_TIMEOUT) {
                Ok(_) => {}
                Err(ping::Error::Timeout) => missing_ips.push(ip.to_string()),
                Err(e) => return (false, format!("because fail to ping {ip}: {e}")),
            }
        } else {
            panic!("ip address malformat")
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let ips: Vec<&str> = attr_str.split(',').collect();
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
        ..
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(&format!("_check_{ident}"), proc_macro2::Span::call_site());

    let timeout_secs = PING_TIMEOUT.as_secs();
    let ping_ips = |pinger: proc_macro2::TokenStream, dot_await: proc_macro2::TokenStream| {
        quote::quote! {
            let mut pinger = #pinger;
            let mut missing_ips = vec![];
            #(
                match pinger.ping(
                    #ips.parse::<std::net::IpAddr>().expect("ip address is invalid"),
                    std::time::Duration::from_secs(#timeout_secs),
                )#dot_await {
                    Ok(_) => {},
                    Err(test_with::ping::Error::Timeout) => missing_ips.push(#ips),
                    Err(e) => return Ok(test_with::Completion::ignored_with(format!("because fail to ping {}: {e}", #ips))),
                }
            )*
        }
    };
    let sync_ping = ping_ips(
        quote::quote! { test_with::ping::Pinger::new() },
        quote::quote! {},
    );
    let async_ping = ping_ips(
        quote::quote! { test_with::ping::tokio::Pinger::new() },
        quote::quote! { .await },
    );

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                #async_ping
                match missing_ips.len() {
                    0 => {
                        #ident().await;
                        Ok(test_with::Completion::Completed)
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because {} not response", missing_ips[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because following ips not response: \n{}\n", missing_ips.join(", ")))),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                #async_ping
                match missing_ips.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(test_with::Completion::Completed)
                        }
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because {} not response", missing_ips[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because following ips not response: \n{}\n", missing_ips.join(", ")))),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                #sync_ping
                match missing_ips.len() {
                    0 => {
                        #ident();
                        Ok(test_with::Completion::Completed)
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because {} not response", missing_ips[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because following ips not response: \n{}\n", missing_ips.join(", ")))),
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
