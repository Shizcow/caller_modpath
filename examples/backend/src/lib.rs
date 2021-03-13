#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_def_site)]

use caller_modpath::CallerModpath;
use proc_macro::TokenStream;

#[caller_modpath::expose_caller_modpath]
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    panic!(
        "module path of call site: {}",
        proc_macro::Span::caller_modpath()
    );
}
