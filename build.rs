use std::env;
use std::fs;
use std::path::PathBuf;

use cmake::Config;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"));

    // Copy dectalk source into OUT_DIR
    let dectalk_source_path =
        PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("dectalk");
    let dectalk_path = out_dir.join("dectalk");

    dircpy::copy_dir(dectalk_source_path, &dectalk_path)
        .expect("Failed to copy DECTalk to OUT_DIR");

    let libdir_path: PathBuf;
    if env::var("DOCS_RS").is_ok() {
        // ---- Docs.rs Build ----
        // Because the native dependencies won't be available on docs.rs, just skip building them
        println!("cargo:warning=DOCS_RS detected: Skipping native build");

        libdir_path = out_dir.join("include");

        // Create the output path
        fs::create_dir_all(libdir_path.join("dtk")).expect("Failed to create include directory");

        // Copy headers since we need those
        fs::copy(
            dectalk_path.join("src/dapi/src/api/ttsapi.h"),
            libdir_path.join("dtk/ttsapi.h"),
        )
        .expect("Failed to copy ttsapi.h header");
        fs::copy(
            dectalk_path.join("src/dapi/src/osf/dtmmedefs.h"),
            libdir_path.join("dtk/dtmmedefs.h"),
        )
        .expect("Failed to copy dtmmedefs.h header");
    } else {
        // ---- Regular Build ----

        // Build the DECTalk binaries and stuff
        let dst = Config::new(&dectalk_path)
            .cxxflag("-DCMAKE_INSTALL_PREFIX=$OUT_DIR")
            // Don't build the samples
            .define("BUILD_SAMPLES", "OFF")
            .build();

        println!("cargo:rustc-link-search=native={}/lib", dst.display());
        println!("cargo:rustc-link-lib=dylib=dectalk");

        libdir_path = dst
            .join("include")
            .canonicalize()
            .expect("Can not canonicalize path");
    }

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
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
