pub use caller_modpath_macros::*;

pub use proc_macro2::{Span, Ident};
pub use quote::quote;

use std::path::PathBuf;
use proc_macro2::TokenStream;
use uuid::Uuid;

pub static UUID_ENV_VAR_NAME: &'static str = "CARGO_INJECT_USEGROUP_SECOND_PASS_UUID";

pub fn gen_second_pass() -> TokenStream {
        let i = Ident::new(
            &format!(
                "USERGROUP_UUID_{}",
                std::env::var(UUID_ENV_VAR_NAME).unwrap()
            ),
            Span::call_site(),
        );
        TokenStream::from(quote! {
            static #i: &'static str = module_path!();
        })
}

pub fn gen_first_pass() -> String {

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

    let dep_p = PathBuf::from(std::env::current_dir().expect("Could not read env var OUT_DIR")).join(dep_dir)
        .join(format!("lib{}-*.so", env!("CARGO_CRATE_NAME")))
        .into_os_string();

    let dep_str = dep_p.to_string_lossy();

    let chosen_dir = glob::glob(&dep_str)
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
        .expect("Could not find suitable backend library paths")
        .into_os_string();

    let proc = std::process::Command::new("rustc")
	.current_dir(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .args(&[
            "-Z",
            "unstable-options",
            "--pretty=expanded",
            "--color=never",
            "--extern",
        ])
        .arg(format!("proc_crate={}", chosen_dir.to_string_lossy()))
        .arg(entry_p.into_os_string())
        .env("CARGO_INJECT_USEGROUP_SECOND_PASS_UUID", &uuid_string)
        .output()
        .expect("failed to execute a second pass of rustc");
    
    String::from_utf8_lossy(&proc.stdout).split(&uuid_string)
        .nth(1)
        .expect(&format!("Failed to find internal UUID; rustc metacall probably faliled. Stderr:\n{}", String::from_utf8_lossy(&proc.stderr)))
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
