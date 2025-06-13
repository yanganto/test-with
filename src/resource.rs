use proc_macro_error2::abort_call_site;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_mem_condition(mem_size_str: String) -> (bool, String) {
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

pub(crate) fn check_swap_condition(swap_size_str: String) -> (bool, String) {
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

pub(crate) fn check_cpu_core_condition(core_limitation_str: String) -> (bool, String) {
    (
        match core_limitation_str.parse::<usize>() {
            Ok(c) => num_cpus::get() >= c,
            Err(_) => abort_call_site!("core limitation is incorrect"),
        },
        format!("because the cpu core less than {}", core_limitation_str),
    )
}

pub(crate) fn check_phy_core_condition(core_limitation_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.total_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_free_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_available_mem(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.available_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
                );
                let mem_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.available_memory()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system memory size can not get"),
                };
                let mem_size_limitation = libtest_with::byte_unit::Byte::parse_str(#mem_limitation_str, true).expect("mem limitation should correct");
                if  mem_size >= mem_size_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the memory less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #mem_limitation_str).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                let sys = libtest_with::sysinfo::System::new_with_specifics(
                    libtest_with::sysinfo::RefreshKind::nothing().with_memory(libtest_with::sysinfo::MemoryRefreshKind::nothing().with_ram()),
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

#[cfg(feature = "runtime")]
pub(crate) fn runtime_free_swap(attr: TokenStream, stream: TokenStream) -> TokenStream {
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
                let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_swap()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system swap size can not get"),
                };
                let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
                if swap_size >= swap_size_limitation {
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
                let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_swap()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system swap size can not get"),
                };
                let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
                if swap_size >= swap_size_limitation {
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
                let swap_size = match libtest_with::byte_unit::Byte::parse_str(format!("{} B", sys.free_swap()), false) {
                    Ok(b) => b,
                    Err(_) => panic!("system swap size can not get"),
                };
                let swap_size_limitation = libtest_with::byte_unit::Byte::parse_str(#swap_limitation_str, true).expect("swap limitation should correct");
                if swap_size >= swap_size_limitation {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_cpu_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get() >= #core_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get() >= #core_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get() >= #core_limitation {
                    #ident();
                    Ok(())
                } else {
                    Err(format!("{}because the cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_phy_cpu_core(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get_physical() >= #core_limitation {
                    #ident().await;
                    Ok(())
                } else {
                    Err(format!("{}because the physical cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
                }
            }
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get_physical() >= #core_limitation {
                    if let Err(e) = #ident().await {
                        Err(format!("{e:?}").into())
                    } else {
                        Ok(())
                    }
                } else {
                    Err(format!("{}because the physical cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
                }
            }
        },
        (None, _) => quote::quote! {
            fn #check_ident() -> Result<(), libtest_with::Failed> {
                if libtest_with::num_cpus::get_physical() >= #core_limitation {
                    #ident();
                    Ok(())
                } else {
                    Err(format!("{}because the physical cpu core less than {}",
                            libtest_with::RUNTIME_IGNORE_PREFIX, #core_limitation).into())
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
