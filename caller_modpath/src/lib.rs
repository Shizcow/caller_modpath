#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
#![feature(proc_macro_def_site)]
use proc_macro::TokenStream;
//use std::io::Write;
use std::path::PathBuf;
use proc_macro::Span;
use uuid::Uuid;

// write path to file
#[proc_macro_attribute]
pub fn push_to_tag(attr: TokenStream, input: TokenStream) -> TokenStream {
    if std::env::var("CARGO_INJECT_USEGROUP_SECOND_PASS_UUID").is_ok() {
	let i = proc_macro2::Ident::new(&format!("USERGROUP_UUID_{}", std::env::var("CARGO_INJECT_USEGROUP_SECOND_PASS_UUID").unwrap()), proc_macro2::Span::call_site());
	return TokenStream::from(quote::quote!{
	    static #i: &'static str = module_path!();
	});
    } else {
	let p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Could not get CARGO_MANIFEST_DIR")).join("Cargo.toml");
	let manifest = cargo_manifest::Manifest::from_path(p).expect("Could not load source's Cargo.toml");

	let bin_name = std::env::var("CARGO_BIN_NAME").expect("Could not get CARGO_BIN_NAME");
	let rustc_entry = manifest.bin.unwrap().into_iter().find(|target| target.name.as_ref() == Some(&bin_name)).unwrap().path.unwrap();
	let entry_p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(rustc_entry);
	eprintln!("rustc_entry: {:?}", entry_p);

	let uuid_string = Uuid::new_v4().to_string().replace("-", "_");

	let dep_dir = format!("target/{}/deps/", if cfg!(debug_assertions) {
		"debug"
	    } else {
		"release"
	});

	let dep_p = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(dep_dir).join(format!("lib{}-*.so", env!("CARGO_CRATE_NAME"))).into_os_string();

	let dep_str = dep_p.to_string_lossy();

	let chosen_dir = glob::glob(&dep_str).expect("Failed to read glob pattern").into_iter().filter_map(|entry| {
	    entry.ok().and_then(|e| std::fs::metadata(&e).and_then(|f| f.accessed()).ok().map(|t| (e, t)))
	}).min().map(|(f, _)| f).unwrap().into_os_string();
	
	let proc = std::process::Command::new("rustc").args(&["-Z", "unstable-options", "--pretty=expanded", "--color=never", "--extern"])
	    .arg(format!("proc_crate={}", chosen_dir.to_string_lossy()))
	    .arg(entry_p.into_os_string())
	    .env("CARGO_INJECT_USEGROUP_SECOND_PASS_UUID", &uuid_string)
	    .current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
	    .output()
            .expect("failed to execute a second pass of rustc");
	let rustc_expand = String::from_utf8_lossy(&proc.stdout);
	panic!("module path of call site: {}", rustc_expand.split(&uuid_string).nth(1).unwrap().chars().skip_while(|c| c!=&'"').skip(1).take_while(|c| c!=&'"').collect::<String>());
    }
}

// use it somehow
#[proc_macro]
pub fn use_tag(_attr: TokenStream) -> TokenStream {
    todo!();
}
