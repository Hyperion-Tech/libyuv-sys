#[cfg(feature="build")]
extern crate cmake;

use std::env;
use std::path::PathBuf;

fn output_dir() -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").unwrap())
}

#[cfg(feature="build")]
fn build_dir() -> PathBuf {
    let mut absolute = env::current_dir().unwrap();
    absolute.push(&output_dir());
    absolute
}

#[cfg(feature="bundled")]
fn source_dir() -> PathBuf {
    env::current_dir().unwrap().join("libyuv")
}

#[cfg(feature="fetched")]
fn source_dir() -> PathBuf {
    output_dir().join("libyuv")
}

#[cfg(feature="fetched")]
fn fetch() -> std::io::Result<std::process::ExitStatus> {
    std::process::Command::new("git")
        .current_dir(&output_dir())
        .arg("clone")
        .arg("https://chromium.googlesource.com/libyuv/libyuv")
        .status()
}

fn main() {
    #[cfg(feature="build")]
    let include_paths: Vec<PathBuf> = {
        use std::fs;

        let statik = cfg!(feature = "static-link");

        println!(
            "cargo:rustc-link-search=native={}",
            build_dir().join("lib").to_string_lossy()
        );

        let kind = if statik { "static" } else { "dylib" };

        println!("cargo:rustc-link-lib={}=yuv", kind);

        #[cfg(feature="fetched")]
        {
            if fs::metadata(&build_dir().join("libyuv")).is_err() {
                fs::create_dir_all(&output_dir())
                    .ok()
                    .expect("failed to create build directory");

                fetch().expect("Unable to fetch libyuv");
            }
        }

        if (statik && fs::metadata(&build_dir().join("lib").join("libyuv.a")).is_err())
            || (!statik && fs::metadata(&build_dir().join("lib").join("libyuv.so")).is_err())
        {
            cmake::Config::new(source_dir()).build();
        }

        vec![build_dir().join("include")]
    };
    #[cfg(not(feature="build"))]
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

    let out_path = output_dir();
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
