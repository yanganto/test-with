use crate::utils::sanitize_env_vars_attr;

#[cfg(feature = "runtime")]
use proc_macro::TokenStream;
#[cfg(feature = "runtime")]
use syn::{parse_macro_input, ItemFn, ReturnType};

pub(crate) fn check_env_condition(attr_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_env(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_vars = vec![];
                #(
                    if std::env::var(#var_names).is_err() {
                        missing_vars.push(#var_names);
                    }
                )*
                match missing_vars.len() {
                    0 => {
                        let _ = #ident().await;
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
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut missing_vars = vec![];
                #(
                    if std::env::var(#var_names).is_err() {
                        missing_vars.push(#var_names);
                    }
                )*
                match missing_vars.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
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
        },
        (None, _) => quote::quote! {
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

pub(crate) fn check_no_env_condition(attr_str: String) -> (bool, String) {
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

#[cfg(feature = "runtime")]
pub(crate) fn runtime_no_env(attr: TokenStream, stream: TokenStream) -> TokenStream {
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

    let check_fn = match (&sig.asyncness, &sig.output) {
        (Some(_), ReturnType::Default) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut should_no_exist_vars = vec![];
                #(
                    if std::env::var(#var_names).is_ok() {
                        should_no_exist_vars.push(#var_names);
                    }
                )*
                match should_no_exist_vars.len() {
                    0 => {
                        #ident().await;
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
        },
        (Some(_), ReturnType::Type(_, _)) => quote::quote! {
            async fn #check_ident() -> Result<(), libtest_with::Failed> {
                let mut should_no_exist_vars = vec![];
                #(
                    if std::env::var(#var_names).is_ok() {
                        should_no_exist_vars.push(#var_names);
                    }
                )*
                match should_no_exist_vars.len() {
                    0 => {
                        if let Err(e) = #ident().await {
                            Err(format!("{e:?}").into())
                        } else {
                            Ok(())
                        }
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
        },
        (None, _) => quote::quote! {
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
        },
    };

    quote::quote! {
            #check_fn
            #(#attrs)*
            #vis #sig #block
    }
    .into()
}

#[cfg(test)]
mod tests {
    use crate::env::{check_env_condition, check_no_env_condition};

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
