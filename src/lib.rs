use proc_macro::TokenStream;
use proc_macro_error::{abort_call_site, proc_macro_error};
use regex::Regex;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn env(attr: TokenStream, item: TokenStream) -> TokenStream {
    let context = item.to_string();
    let fn_re = Regex::new(r"^fn (?P<fn_name>[^\(\s]*)(?P<to_first_bracket>[^\{]*\{)").unwrap();

    if fn_re.captures(&context).is_some() {
        let var_name = attr.to_string().replace(" ", "");
        return if std::env::var(var_name).is_ok() {
            fn_re
                .replace(
                    &context,
                    r#"
                #[test]
                fn $fn_name () {
                    "#,
                )
                .to_string()
                .parse()
                .unwrap()
        } else {
            fn_re
                .replace(
                    &context,
                    r#"
                #[test]
                #[ignore = "var not found"]
                fn $fn_name () {
                    "#,
                )
                .to_string()
                .parse()
                .unwrap()
        };
    } else {
        abort_call_site!("parse mod or function for testing error")
    }
}
