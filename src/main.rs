#![allow(dead_code)]

mod a {
    //#[push_to_tag("mytag")]
    static A: i32 = 0;
    //#[push_to_tag("mytag")]
    static B: i32 = 1;
    pub mod b {
        #[proc_crate::push_to_tag("mytag")]
        static C: i32 = 3;
    }
}
static D: i32 = 4;

fn main() {
    //use_tag!("mytag");
    //println!("{} {} {} {}", A, B, C, D);
}

// rustc -Z unstable-options --pretty=expanded src/main.rs
