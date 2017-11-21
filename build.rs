extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let libyuv_dir = env::var("LIBYUV_DIR").expect("LIBYUV_DIR should be defined");

    println!("cargo:rustc-link-lib=yuv");
    println!("cargo:rustc-link-search=native={}/lib", libyuv_dir);

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}/include", libyuv_dir))
        .header("wrapper.h")
        .trust_clang_mangling(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
