extern crate bindgen;
extern crate cc;
#[cfg(feature="bundled")]
extern crate cmake;

use std::env;
#[cfg(feature="bundled")]
use std::fs;
#[cfg(feature="bundled")]
use std::io;
use std::path::PathBuf;
#[cfg(feature="bundled")]
use std::process::Command;

#[cfg(feature="bundled")]
fn output_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

#[cfg(feature="bundled")]
fn search_dir() -> PathBuf {
    let mut absolute = env::current_dir().unwrap();
    absolute.push(&output_dir());
    absolute
}

#[cfg(feature="bundled")]
fn source_dir() -> PathBuf {
    output_dir().join("libyuv")
}

#[cfg(feature="bundled")]
fn fetch() -> io::Result<()> {
    let status = Command::new("git")
        .current_dir(&output_dir())
        .arg("clone")
        .arg("https://chromium.googlesource.com/libyuv/libyuv")
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
    }
}

fn main() {
    #[cfg(feature="bundled")]
    let include_paths: Vec<PathBuf> = {
        let statik = cfg!(feature = "static-link");

        println!(
            "cargo:rustc-link-search=native={}",
            search_dir().join("lib").to_string_lossy()
        );

        let kind = if statik { "static" } else { "dylib" };

        println!("cargo:rustc-link-lib={}=yuv", kind);

        if fs::metadata(&search_dir().join("libyuv")).is_err() {
            fs::create_dir_all(&output_dir())
                .ok()
                .expect("failed to create build directory");
            fetch().unwrap();
        }

        if (statik && fs::metadata(&search_dir().join("lib").join("libyuv.a")).is_err())
            || (!statik && fs::metadata(&search_dir().join("lib").join("libyuv.so")).is_err())
        {
            cmake::Config::new(source_dir()).build();
        }

        vec![search_dir().join("include")]
    };

    #[cfg(not(feature="bundled"))]
    let include_paths: Vec<PathBuf> = {
        println!("cargo:rustc-link-lib=yuv");

        Vec::new()
    };

    // In cross compiling, recent bindgen fails if proper `--sysroot` is not specified.
    // To workaround this, try to figure out sysroot path from the compiler itself.
    let mut compiler = cc::Build::new().get_compiler().to_command();
    let sysroot = compiler
        .arg("-print-sysroot")
        .output()
        .map(|o| String::from_utf8(o.stdout).unwrap().trim().to_string())
        .ok();

    let mut bindgen = bindgen::Builder::default()
        .header("wrapper.h")
        .trust_clang_mangling(false)
        .blacklist_type("max_align_t") // Until https://github.com/rust-lang-nursery/rust-bindgen/issues/550 gets fixed
        ;

    if let Some(path) = sysroot {
        bindgen = bindgen.clang_arg(format!("--sysroot={}", path));
    }

    for dir in include_paths {
        bindgen = bindgen
            .clang_arg("-I")
            .clang_arg(dir.to_string_lossy().into_owned());
    }

    let bindings = bindgen.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
