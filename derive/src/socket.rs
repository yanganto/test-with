use std::net::TcpStream;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_tcp_condition(attr_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_tcp(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_sockets = vec![];
                #(
                    if std::net::TcpStream::connect(#sockets).is_err() {
                        missing_sockets.push(#sockets);
                    }
                )*
                match missing_sockets.len() {
                    0 => {
                        #ident().await;
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
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_sockets = vec![];
                #(
                    if std::net::TcpStream::connect(#sockets).is_err() {
                        missing_sockets.push(#sockets);
                    }
                )*
                match missing_sockets.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
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
        },
        (None, _) => quote::quote! {
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}
