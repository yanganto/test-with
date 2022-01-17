#[cfg(feature = "ign-msg")]
use proc_macro2::{Ident, Span};
#[cfg(feature = "ign-msg")]
use regex::Regex;
use syn::Attribute;
#[cfg(feature = "ign-msg")]
use syn::Signature;

pub(crate) fn has_test_attr(attrs: &Vec<Attribute>) -> bool {
    for attr in attrs.iter() {
        if let Some(seg) = attr.path.segments.last() {
            if "test" == seg.ident.to_string() {
                return true;
            }
        }
    }
    false
}

#[cfg(feature = "ign-msg")]
pub(crate) fn rewrite_fn_name_with_msg(sig: &mut Signature, msg: &String) {
    let re = unsafe { Regex::new(r"[^\w]").unwrap_unchecked() };
    let new_fn_name = Ident::new(
        &format!("{}__{}", sig.ident.to_string(), re.replace_all(msg, "_")),
        Span::call_site(),
    );
    sig.ident = new_fn_name;
}
