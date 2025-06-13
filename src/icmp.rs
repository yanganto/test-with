use proc_macro_error2::abort_call_site;
use std::net::IpAddr;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_icmp_condition(attr_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_icmp(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_ips = vec![];
                #(
                    if libtest_with::ping::ping(#ips.parse().expect("ip address is invalid"), None, None, None, None, None).is_err() {
                        missing_ips.push(#ips);
                    }
                )*
                match missing_ips.len() {
                    0 => {
                        #ident().await;
                        Ok(())
                    },
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
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_ips = vec![];
                #(
                    if libtest_with::ping::ping(#ips.parse().expect("ip address is invalid"), None, None, None, None, None).is_err() {
                        missing_ips.push(#ips);
                    }
                )*
                match missing_ips.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
                    },
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
        },
        (None, _) => quote::quote! {
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}
