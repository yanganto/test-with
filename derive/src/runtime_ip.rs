use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemFn, ReturnType};

const DEFAULT_IP_CHECK_SITE: &str = "https://ip.me";

/// Parse the optional `ip_check_site=<url>` override out of the attribute parts, falling back to
/// the default public ip check site.
fn parse_ip_check_site<'a>(parts: &[&'a str]) -> &'a str {
    parts
        .iter()
        .find_map(|p| p.strip_prefix("ip_check_site="))
        // the url has to be a string literal (e.g. "https://ip.me") because a bare `//` would be
        // lexed as a comment, so strip the surrounding quotes here
        .map(|s| s.trim_matches('"'))
        .unwrap_or(DEFAULT_IP_CHECK_SITE)
}

/// Build an expression that fetches the public ip from the check site.  The expression evaluates
/// to the parsed `std::net::IpAddr`, or early returns an ignored `Completion` from the enclosing
/// check function when the ip can not be got or parsed.  Bind it at the call site, e.g.
/// `let public_ip: std::net::IpAddr = #fetch;`.
fn fetch_public_ip(site: &str, is_async: bool) -> proc_macro2::TokenStream {
    if is_async {
        quote::quote! {
            match test_with::reqwest::get(#site).await {
                Ok(resp) => match resp.text().await {
                    Ok(body) => match body.trim().parse() {
                        Ok(ip) => ip,
                        Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not parse public ip from {}", #site))),
                    },
                    Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not get public ip from {}", #site))),
                },
                Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not get public ip from {}", #site))),
            }
        }
    } else {
        quote::quote! {
            match test_with::reqwest::blocking::get(#site) {
                Ok(resp) => match resp.text() {
                    Ok(body) => match body.trim().parse() {
                        Ok(ip) => ip,
                        Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not parse public ip from {}", #site))),
                    },
                    Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not get public ip from {}", #site))),
                },
                Err(_) => return Ok(test_with::Completion::ignored_with(format!("because can not get public ip from {}", #site))),
            }
        }
    }
}

/// Run test case when the example running and the public ip can be got from the check site.
/// The default check site is `https://ip.me`, and can be overridden with `ip_check_site=<url>`.
pub(crate) fn runtime_public_ip(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let parts: Vec<&str> = if attr_str.is_empty() {
        vec![]
    } else {
        attr_str.split(',').collect()
    };
    let site = parse_ip_check_site(&parts);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(&format!("_check_{ident}"), proc_macro2::Span::call_site());

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let _public_ip: std::net::IpAddr = #fetch;
                    #ident().await;
                    Ok(test_with::Completion::Completed)
                }
            }
        }
        (Some(_), ReturnType::Type(_, _)) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let _public_ip: std::net::IpAddr = #fetch;
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(test_with::Completion::Completed)
                    }
                }
            }
        }
        (None, _) => {
            let fetch = fetch_public_ip(site, false);
            quote::quote! {
                fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let _public_ip: std::net::IpAddr = #fetch;
                    #ident();
                    Ok(test_with::Completion::Completed)
                }
            }
        }
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

/// Run test case when the example running and the public ip is in the CIDR.
/// The default check site is `https://ip.me`, and can be overridden with `ip_check_site=<url>`.
pub(crate) fn runtime_public_ip_in(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let parts: Vec<&str> = attr_str.split(',').collect();
    let cidr = parts.first().copied().unwrap_or_default();
    if cidr.is_empty() || cidr.starts_with("ip_check_site=") {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "runtime_public_ip_in needs a CIDR, e.g. #[test_with::runtime_public_ip_in(1.2.3.0/24)]",
        )
        .to_compile_error()
        .into();
    }
    let site = parse_ip_check_site(&parts[1..]);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(&format!("_check_{ident}"), proc_macro2::Span::call_site());

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let net: test_with::ipnet::IpNet = #cidr.parse().expect("CIDR is invalid");
                    if net.contains(&public_ip) {
                        #ident().await;
                        Ok(test_with::Completion::Completed)
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} not in {}", public_ip, #cidr)))
                    }
                }
            }
        }
        (Some(_), ReturnType::Type(_, _)) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let net: test_with::ipnet::IpNet = #cidr.parse().expect("CIDR is invalid");
                    if net.contains(&public_ip) {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(test_with::Completion::Completed)
                        }
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} not in {}", public_ip, #cidr)))
                    }
                }
            }
        }
        (None, _) => {
            let fetch = fetch_public_ip(site, false);
            quote::quote! {
                fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let net: test_with::ipnet::IpNet = #cidr.parse().expect("CIDR is invalid");
                    if net.contains(&public_ip) {
                        #ident();
                        Ok(test_with::Completion::Completed)
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} not in {}", public_ip, #cidr)))
                    }
                }
            }
        }
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

/// Run test case when the example running and the public ip is exactly the expected ip.
/// The default check site is `https://ip.me`, and can be overridden with `ip_check_site=<url>`.
pub(crate) fn runtime_public_ip_is(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let parts: Vec<&str> = attr_str.split(',').collect();
    let expected = parts.first().copied().unwrap_or_default();
    if expected.is_empty() || expected.starts_with("ip_check_site=") {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "runtime_public_ip_is needs an ip, e.g. #[test_with::runtime_public_ip_is(1.2.3.4)]",
        )
        .to_compile_error()
        .into();
    }
    let site = parse_ip_check_site(&parts[1..]);
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(&format!("_check_{ident}"), proc_macro2::Span::call_site());

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let expected_ip: std::net::IpAddr = #expected.parse().expect("ip is invalid");
                    if public_ip == expected_ip {
                        #ident().await;
                        Ok(test_with::Completion::Completed)
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} is not {}", public_ip, #expected)))
                    }
                }
            }
        }
        (Some(_), ReturnType::Type(_, _)) => {
            let fetch = fetch_public_ip(site, true);
            quote::quote! {
                async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let expected_ip: std::net::IpAddr = #expected.parse().expect("ip is invalid");
                    if public_ip == expected_ip {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(test_with::Completion::Completed)
                        }
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} is not {}", public_ip, #expected)))
                    }
                }
            }
        }
        (None, _) => {
            let fetch = fetch_public_ip(site, false);
            quote::quote! {
                fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                    let public_ip: std::net::IpAddr = #fetch;
                    let expected_ip: std::net::IpAddr = #expected.parse().expect("ip is invalid");
                    if public_ip == expected_ip {
                        #ident();
                        Ok(test_with::Completion::Completed)
                    } else {
                        Ok(test_with::Completion::ignored_with(format!("because public ip {} is not {}", public_ip, #expected)))
                    }
                }
            }
        }
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

pub(crate) fn runtime_ip_in(attr: TokenStream, stream: TokenStream) -> TokenStream {
    let attr_str = attr.to_string().replace(' ', "");
    let cidrs: Vec<&str> = attr_str.split(',').collect();
    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = parse_macro_input!(stream as ItemFn);
    let syn::Signature { ident, .. } = sig.clone();
    let check_ident = syn::Ident::new(&format!("_check_{ident}"), proc_macro2::Span::call_site());

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let interface_ips: Vec<std::net::IpAddr> = match test_with::local_ip_address::list_afinet_netifas() {
                    Ok(ifas) => ifas.into_iter().map(|(_, ip)| ip).collect(),
                    Err(_) => panic!("can not get the ip of the interfaces"),
                };
                let mut missing_cidrs = vec![];
                #(
                    let net: test_with::ipnet::IpNet = #cidrs.parse().expect("CIDR is invalid");
                    if !interface_ips.iter().any(|ip| net.contains(ip)) {
                        missing_cidrs.push(#cidrs);
                    }
                )*
                match missing_cidrs.len() {
                    0 => {
                        #ident().await;
                        Ok(test_with::Completion::Completed)
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because no interface ip in {}", missing_cidrs[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because no interface ip in following cidr: \n{}\n", missing_cidrs.join(", ")))),
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let interface_ips: Vec<std::net::IpAddr> = match test_with::local_ip_address::list_afinet_netifas() {
                    Ok(ifas) => ifas.into_iter().map(|(_, ip)| ip).collect(),
                    Err(_) => panic!("can not get the ip of the interfaces"),
                };
                let mut missing_cidrs = vec![];
                #(
                    let net: test_with::ipnet::IpNet = #cidrs.parse().expect("CIDR is invalid");
                    if !interface_ips.iter().any(|ip| net.contains(ip)) {
                        missing_cidrs.push(#cidrs);
                    }
                )*
                match missing_cidrs.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(test_with::Completion::Completed)
                        }
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because no interface ip in {}", missing_cidrs[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because no interface ip in following cidr: \n{}\n", missing_cidrs.join(", ")))),
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<test_with::Completion, test_with::Failed> {
                let interface_ips: Vec<std::net::IpAddr> = match test_with::local_ip_address::list_afinet_netifas() {
                    Ok(ifas) => ifas.into_iter().map(|(_, ip)| ip).collect(),
                    Err(_) => panic!("can not get the ip of the interfaces"),
                };
                let mut missing_cidrs = vec![];
                #(
                    let net: test_with::ipnet::IpNet = #cidrs.parse().expect("CIDR is invalid");
                    if !interface_ips.iter().any(|ip| net.contains(ip)) {
                        missing_cidrs.push(#cidrs);
                    }
                )*
                match missing_cidrs.len() {
                    0 => {
                        #ident();
                        Ok(test_with::Completion::Completed)
                    },
                    1 => Ok(test_with::Completion::ignored_with(format!("because no interface ip in {}", missing_cidrs[0]))),
                    _ => Ok(test_with::Completion::ignored_with(format!("because no interface ip in following cidr: \n{}\n", missing_cidrs.join(", ")))),
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
