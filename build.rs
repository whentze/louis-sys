extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=louis");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .unwrap_or_else(|_| {
            bindgen::Builder::default()
                .header("fallback.h")
                .generate()
                .expect("Error generating Bindings")
        });

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
