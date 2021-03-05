fn main() {
    // just trigger a rebuild
    println!("cargo:rerun-if-changed=src/test2.rs");
}
