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

    trait ModPath {
        fn modpath() -> &'static String {
            static CELL: caller_modpath::OnceCell<String> = caller_modpath::OnceCell::new();
            CELL.get_or_init(|| caller_modpath::gen_first_pass(env!("CARGO_CRATE_NAME")))
        }
    }

    impl ModPath for proc_macro::Span {}

    panic!("module path of call site: {}", proc_macro::Span::modpath());

    input
}
