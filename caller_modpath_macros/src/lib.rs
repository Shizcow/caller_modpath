//! Macros for [`caller_modpath`](https://docs.rs/caller_modpath).

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse::Nothing, Block, ItemFn};
/// Makes [`Span::caller_modpath()`](https://docs.rs/caller_modpath/trait.CallerModpath.html#tymethod.caller_modpath)
/// available to the applied macro, and all functions which it calls.
///
/// ## Note:
/// This can only be applied to [`#[proc_macro_attribute]`](https://doc.rust-lang.org/nightly/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes) functions.
///
/// [`#[expose_caller_modpath]`](macro@expose_caller_modpath) should go above
/// [`#[proc_macro_attribute]`](https://doc.rust-lang.org/nightly/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes)
///
/// For an example, see the [top level documentation](https://docs.rs/caller_modpath).
// prepend the setup code to the beginning of the input proc_macro
#[proc_macro_attribute]
pub fn expose_caller_modpath(attr: TokenStream, input: TokenStream) -> TokenStream {
    syn::parse_macro_input!(attr as Nothing); // I take no args

    // for error reporting
    let proc_err = syn::Error::new(
        proc_macro2::Span::call_site(),
        "#[expose_caller_modpath] can only be used on #[proc_macro_attribute] functions",
    )
    .to_compile_error()
    .into();

    // make sure the format matches
    match syn::parse::<ItemFn>(input) {
        Err(_) => proc_err,
        Ok(input) => {
            // make sure there's #[proc_macro_attribute]
            // This is strictly required due to the rustc meta-call returning early
            if !input
                .attrs
                .clone()
                .into_iter()
                .any(|attr| attr.path.is_ident("proc_macro_attribute"))
            {
                return proc_err;
            }

            // This will be placed at the beginning of the function
            let mut inject = syn::parse2::<Block>(quote! {{
                if std::env::var(caller_modpath::UUID_ENV_VAR_NAME).is_ok() {
                    return caller_modpath::gen_second_pass();
                }

                caller_modpath::gen_first_pass(env!("CARGO_CRATE_NAME"));
            }})
            .unwrap();

            // wrap everything back up and return
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
