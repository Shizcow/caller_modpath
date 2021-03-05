#[caller_modpath::push_to_tag]
#[proc_macro_attribute]
pub fn test(attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
