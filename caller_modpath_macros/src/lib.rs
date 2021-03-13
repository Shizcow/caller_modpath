use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Nothing, Block, ItemFn};

#[proc_macro_attribute]
pub fn expose_caller_modpath(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args
    let proc_err = syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[expose_caller_modpath] can only be used on #[proc_macro_attribute] functions",
    )
    .to_compile_error()
    .into();
    match syn::parse::<ItemFn>(input) {
        Err(_) => proc_err,
        Ok(input) => {
            if !input
                .attrs
                .clone()
                .into_iter()
                .any(|attr| attr.path.is_ident("proc_macro_attribute"))
            {
                return proc_err;
            }

            let mut inject = syn::parse2::<Block>(quote! {{
                if std::env::var(caller_modpath::UUID_ENV_VAR_NAME).is_ok() {
                    return caller_modpath::gen_second_pass();
                }

                caller_modpath::gen_first_pass(env!("CARGO_CRATE_NAME"));
            }})
            .unwrap();

            let attrs = input.attrs;
            let vis = input.vis;
            let sig = input.sig;
            let mut block = input.block;
            inject.stmts.append(&mut block.stmts);
            (quote! {
                    #(#attrs)*
                    #vis #sig
            #inject
                })
            .into()
        }
    }
}
