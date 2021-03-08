#![feature(proc_macro_span)]

// yeah
extern crate proc_macro;

pub use caller_modpath_macros::*;

pub use once_cell::sync::OnceCell;
pub use quote::quote;

use std::path::PathBuf;
use std::sync::RwLock;
use uuid::Uuid;

// use when we call rustc on ourself (this lib gets wild)
pub static UUID_ENV_VAR_NAME: &str =
    concat!("CARGO_INJECT_", env!("CARGO_PKG_NAME"), "_SECOND_PASS_UUID");

// so Span is a really special type
// It is very dumb and implements no useful traits (Eq, Hash, Send, Sync, etc)
// A lot of this stuff is crazy because of that
// If this was better I'd stick it in a lazy_static HashMap and call it a day but sometype needs attention
thread_local! {
    static MODCACHE: RwLock<Vec<(proc_macro2::Span, ResolveStatus)>> = RwLock::new(vec![]);
}

enum ResolveStatus {
    Unresolved(&'static str), // crate name
    Resolved(String),         // module path name
}

// This trait is the main interface for this crate
pub trait CallerModpath {
    fn caller_modpath() -> String;
}

// Get the caller modpath with lazy calculation
impl CallerModpath for proc_macro::Span {
    fn caller_modpath() -> String {
        let call_site = proc_macro2::Span::call_site().unwrap();
        // First, try to find any mention of it (it's initialized by the macro)
        MODCACHE.with(move |m| {
	    // overwritten and used only when required
	    let mut need_to_write_index = None;
	    let mut newly_resolved = None;
	    { // this weird scope is so the mutex can be reused mutably later
		let locked = m.read().unwrap();
		for i in 0..locked.len() {
                    if locked[i].0.unwrap().eq(&call_site) {
			match locked[i].1 {
			    ResolveStatus::Resolved(ref modpath) => {
				// If we have calculated everything already, just return it
				return modpath.clone();
			    },
			    ResolveStatus::Unresolved(cratename) => {
				// Otherwise, calculate and continue
				let modpath = resolve_modpath(cratename);
				need_to_write_index = Some(i);
				newly_resolved = Some(modpath.to_owned());
			    },
			};
                    }
		};
	    }
	    // If we found no mention, the user forgot to set up
	    if need_to_write_index.is_none() {
		panic!("Attempt to call Span::caller_modpath() without first putting #[expose_caller_modpath] on the parent #[proc_macro_attribute]!");
	    }
	    // Otherise, do the calculation and cache+return the result
	    let mut write_lock = m.write().unwrap();
	    let modpath = newly_resolved.unwrap();
	    write_lock[need_to_write_index.unwrap()].1 = ResolveStatus::Resolved(modpath.clone());
	    modpath
        })
    }
}

// I just want this available for both types
impl CallerModpath for proc_macro2::Span {
    fn caller_modpath() -> String {
        proc_macro::Span::caller_modpath()
    }
}

pub fn gen_second_pass() -> proc_macro::TokenStream {
    let i = proc_macro2::Ident::new(
        &format!(
            "{}_UUID_{}",
            env!("CARGO_PKG_NAME"),
            std::env::var(UUID_ENV_VAR_NAME).unwrap()
        ),
        proc_macro2::Span::call_site(),
    );
    (quote! {
        static #i: &'static str = module_path!();
    })
    .into()
}

pub fn gen_first_pass(client_proc_macro_crate_name: &'static str) {
    // Make sure we aren't logging the call site twice
    let call_site = proc_macro2::Span::call_site().unwrap();
    let already_calculated = MODCACHE.with(|m| {
        let locked = m.read().unwrap();
        for i in 0..locked.len() {
            if locked[i].0.unwrap().eq(&call_site) {
                return true;
            }
        }
        false
    });
    if already_calculated {
        return;
    }
    // Then just push an empty to be resolved when we actually ask for it
    MODCACHE.with(move |m| {
        m.write().unwrap().push((
            proc_macro2::Span::call_site(),
            ResolveStatus::Unresolved(client_proc_macro_crate_name),
        ))
    });
}

fn resolve_modpath(client_proc_macro_crate_name: &str) -> String {
    let entry_p = get_entrypoint();

    let uuid_string = Uuid::new_v4().to_string().replace("-", "_");

    let chosen_dir = find_lib_so(&client_proc_macro_crate_name);

    let liblink_path = format!("{}={}", client_proc_macro_crate_name, chosen_dir);

    let rustc_args = vec![
        "-Z",
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
        .unwrap_or_else(|| panic!("Failed to find internal UUID; rustc metacall probably faliled. Called as `rustc {}`. Stderr:\n{}", rustc_args.join(" "), String::from_utf8_lossy(&proc.stderr)))
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

        let manifest = cargo_manifest::Manifest::from_path(manifest_dir.join("Cargo.toml"))
            .expect("Could not parse Cargo.toml of caller");

        let rustc_entry = manifest.bin.unwrap().into_iter().find(|target| target.name.as_ref() == Some(&bin_name)).expect("Could not get binary target path from Cargo.toml. If you are manually specifying targets, make sure the path is included as well.").path.unwrap();

        manifest_dir.join(rustc_entry)
    } else {
        // just a library: can assume it's just src/lib.rs
        manifest_dir.join("src").join("lib.rs")
    }
}

fn find_lib_so(libname: &str) -> String {
    let target_path = std::env::current_dir()
        .expect("Could not get current dir from env")
        .join("target")
        .join(if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        });

    // need to look in two places:
    // target/{}/deps/ for crate dependencies
    let dep_p = target_path
        .join("deps")
        .join(format!("lib{}-*.so", libname))
        .into_os_string();

    let dep_str = dep_p.to_string_lossy();

    // and target/{}/ for workspace targets
    let t_p = target_path.join(format!("lib{}.so", libname));

    let mut file_candidates: Vec<_> = glob::glob(&dep_str)
        .expect("Failed to read library glob pattern")
        .into_iter()
        .filter_map(|entry| entry.ok())
        .collect();

    file_candidates.push(t_p);

    let fstr = file_candidates
        .iter()
        .map(|p| p.to_string_lossy())
        .collect::<Vec<_>>()
        .join(" ");

    file_candidates
        .into_iter()
        .filter_map(|entry| {
            std::fs::metadata(&entry)
                .and_then(|f| f.accessed())
                .ok()
                .map(|t| (entry, t))
        })
        .max()
        .map(|(f, _)| f)
        .unwrap_or_else(|| {
            panic!(
                "Could not find suitable backend library paths from file list {}",
                fstr
            )
        })
        .into_os_string()
        .to_string_lossy()
        .to_string()
}
