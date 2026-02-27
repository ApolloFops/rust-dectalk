use std::env;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    // Builds the project in the directory located in `libfoo`, installing it
    // into $OUT_DIR
    let dst = Config::new("dectalk")
        .cxxflag("-DCMAKE_INSTALL_PREFIX=$OUT_DIR")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=dylib=dectalk");

    let libdir_path = dst
        .join("include")
        .canonicalize()
        .expect("Can not canonicalize path");
    let libdir_str = libdir_path.to_str().expect("Path is not a valid string");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        // Add include directory
        .clang_arg(format!("-I{libdir_str}"))
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
