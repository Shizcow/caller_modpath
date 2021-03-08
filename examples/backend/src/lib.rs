#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_def_site)]

use caller_modpath::CallerModpath;
use proc_macro::TokenStream;

//#[caller_modpath::make_modpath_available]
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    if std::env::var(caller_modpath::UUID_ENV_VAR_NAME).is_ok() {
        return caller_modpath::gen_second_pass().into();
    }

    caller_modpath::gen_first_pass(env!("CARGO_CRATE_NAME"));

    panic!(
        "module path of call site: {}",
        proc_macro::Span::caller_modpath()
    );
}
