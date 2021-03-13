# caller_modpath
[![crates.io](https://img.shields.io/crates/v/caller_modpath.svg)](https://crates.io/crates/caller_modpath)
[![docs.rs](https://docs.rs/caller_modpath/badge.svg)](https://docs.rs/caller_modpath)

This crates allows for getting the module path of the caller within a
[`#[proc_macro_attribute]`](https://doc.rust-lang.org/nightly/book/ch19-06-macros.html#procedural-macros-for-generating-code-from-attributes).

For more information, read [the docs](https://crates.io/crates/caller_modpath).

## Example
The simplest example is as follows:
```rust
#[caller_modpath::expose_caller_modpath]
#[proc_macro_attribute]
pub fn test(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    let modpath: String = proc_macro::Span::caller_modpath();
    // now do something with it. For example, just panic to have the compiler display the result:
    panic!(
        "module path of call site: {}",
        modpath
    );
}
```
