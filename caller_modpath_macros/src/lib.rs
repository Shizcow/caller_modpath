use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn expose_caller_modpath(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
