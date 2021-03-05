pub use caller_modpath_macros::*;

pub use proc_macro2::{Span, Ident};
pub use quote::quote;

use std::path::PathBuf;
use proc_macro2::TokenStream;
use uuid::Uuid;

pub static UUID_ENV_VAR_NAME: &'static str = concat!("CARGO_INJECT_", env!("CARGO_PKG_NAME"), "_SECOND_PASS_UUID");

pub fn gen_second_pass() -> TokenStream {
        let i = Ident::new(
            &format!(
                "{}_UUID_{}",
		env!("CARGO_PKG_NAME"),
		std::env::var(UUID_ENV_VAR_NAME).unwrap()
            ),
            Span::call_site(),
        );
        TokenStream::from(quote! {
            static #i: &'static str = module_path!();
        })
}

pub fn gen_first_pass(client_proc_macro_crate_name: &str) -> String {
    let client_user_crate_name = std::env::var("CARGO_CRATE_NAME").expect("Could not read env var CARGO_CRATE_NAME");

    let entry_p = get_entrypoint();

    let uuid_string = Uuid::new_v4().to_string().replace("-", "_");

    let dep_dir = format!(
        "target/{}/deps/",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    let chosen_dir = find_lib_so(&client_proc_macro_crate_name);

    let liblink_path = format!("{}={}", client_proc_macro_crate_name,
				   chosen_dir);

    let rustc_args = vec!["-Z",
			  "unstable-options",
			  "--pretty=expanded",
			  "--color=never",
			  "--extern",
			  &liblink_path,
			  entry_p.to_str().unwrap(),
    ];

    let proc = std::process::Command::new("rustc")
	.current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
	.args(&rustc_args)
        .env(UUID_ENV_VAR_NAME, &uuid_string)
        .output()
        .expect("failed to execute a second pass of rustc");
    
    String::from_utf8_lossy(&proc.stdout).split(&uuid_string)
        .nth(1)
        .expect(&format!("Failed to find internal UUID; rustc metacall probably faliled. Called as `rustc {}`. Stderr:\n{}", rustc_args.join(" "), String::from_utf8_lossy(&proc.stderr)))
        .chars()
        .skip_while(|c| c != &'"')
        .skip(1)
        .take_while(|c| c != &'"')
        .collect()
}

fn get_entrypoint() -> PathBuf {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());

    if let Ok(bin_name) = std::env::var("CARGO_BIN_NAME") {
	// binary: need to parse targets in Cargo.toml to find the correct path

	let manifest = cargo_manifest::Manifest::from_path(manifest_dir.join("Cargo.toml")).expect("Could not parse Cargo.toml of caller");

	let rustc_entry = manifest.bin.unwrap().into_iter().find(|target| target.name.as_ref() == Some(&bin_name)).expect("Could not get binary target path from Cargo.toml. If you are manually specifying targets, make sure the path is included as well.").path.unwrap();

	manifest_dir.join(rustc_entry)
    } else {
	// just a library: can assume it's just src/lib.rs
	manifest_dir.join("src").join("lib.rs")
    }
}

fn find_lib_so(libname: &str) -> String {

    let dep_p = PathBuf::from(std::env::current_dir().expect("Could not get current dir from env")).join("target").join(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        })
        .join(format!("lib{}.so", libname))
        .into_os_string();

    let dep_str = dep_p.to_string_lossy();

    glob::glob(&dep_str)
        .expect("Failed to read library glob pattern")
        .into_iter()
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                std::fs::metadata(&e)
                    .and_then(|f| f.accessed())
                    .ok()
                    .map(|t| (e, t))
            })
        })
        .min()
        .map(|(f, _)| f)
        .expect(&format!("Could not find suitable backend library paths via glob {}", dep_str))
        .into_os_string().to_string_lossy().to_string()
}
