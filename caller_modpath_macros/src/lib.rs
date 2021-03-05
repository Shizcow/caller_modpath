#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_def_site)]
use proc_macro::TokenStream;
//use std::io::Write;
use proc_macro::Span;
use std::path::PathBuf;
use uuid::Uuid;

// write path to file
#[proc_macro_attribute]
pub fn make_modpath_available(attr: TokenStream, input: TokenStream) -> TokenStream {
    input
}
