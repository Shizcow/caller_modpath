use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn make_modpath_available(_attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
