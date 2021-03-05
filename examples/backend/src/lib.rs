#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_def_site)]

use proc_macro::TokenStream;

//#[caller_modpath::make_modpath_available]
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    if std::env::var(caller_modpath::UUID_ENV_VAR_NAME).is_ok() {
        return caller_modpath::gen_second_pass().into();
    }
    let c_mod = caller_modpath::gen_first_pass(env!("CARGO_CRATE_NAME"));

    panic!("module path of call site: {}", c_mod);

    input
}
