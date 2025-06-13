use proc_macro_error2::abort_call_site;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_timezone(attr_str: &str) -> (bool, Vec<&str>) {
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
            "CET" | "WAT" | "WEST" | "BST" => Ok(60),
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

pub(crate) fn check_tz_condition(attr_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_timezone(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
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
                        #ident().await;
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
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
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
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
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
        },
        (None, _) => quote::quote! {
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}
