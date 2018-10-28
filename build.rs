extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    // In cross compiling, recent bindgen fails if proper `--sysroot` is not specified.
    // To workaround this, try to figure out sysroot path from the compiler itself.
    let mut compiler = cc::Build::new().get_compiler().to_command();
    let sysroot = compiler.arg("-print-sysroot")
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().to_string())
        .ok();

    let mut bindgen = bindgen::Builder::default()
        .header("wrapper.h")
        .trust_clang_mangling(false);

    if let Some(path) = sysroot {
        println!("cargo:warning=Using library at {:?}", path);
        bindgen = bindgen.clang_arg(format!("--sysroot={}", path));
    }

    let bindings = bindgen.generate()
        .expect("Unable to generate bindings");

    println!("cargo:rustc-link-lib=yuv");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
