//
// Copyright (C) 2020 Jonas Zaddach.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 2 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston,
// MA 02110-1301, USA.

use std::path::{PathBuf, Path};
use std::env;

fn main() {
    println!("cargo:rustc-link-lib=dylib={}", "clamav");
    if cfg!(windows) {
        let clamav_build = env::var("CLAMAV_BUILD").expect("CLAMAV_BUILD environment variable must be set and point to ClamAV's build directory");
        let profile = env::var("PROFILE").unwrap();

        let library_path = match profile.as_str() {
            "debug" => Path::new(&clamav_build).join("libclamav/Debug"),
            "release" => Path::new(&clamav_build).join("libclamav/Release"),
            _ => panic!("Unexpected build profile"),
        };

        println!("cargo:rustc-link-search=native={}", library_path.to_str().unwrap());
    }
    else {
        pkg_config::Config::new()
            .atleast_version("0.102")
            .probe("libclamav")
            .unwrap();
    }

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        //Whitelist wanted functions
        .whitelist_function("cl_init")
        .whitelist_function("cl_initialize_crypto")
        .whitelist_function("cl_cleanup_crypto")
        .whitelist_function("cl_strerror")
        .whitelist_function("cl_engine_new")
        .whitelist_function("cl_engine_free")
        .whitelist_function("cl_engine_compile")
        .whitelist_function("cl_scandesc")
        .whitelist_function("cl_scanmap_callback")
        .whitelist_function("cl_fmap_open_memory")
        .whitelist_function("cl_fmap_close")
        .whitelist_function("cl_retflevel")
        .whitelist_function("cl_retver")
        .whitelist_function("cl_load")
        .whitelist_function("cl_scanfile")
        .whitelist_function("cl_retdbdir")
        .rustified_enum("cl_error_t")
        //Whitelist wanted constants
        .whitelist_var("CL_SCAN_.*")
        .whitelist_var("CL_INIT_DEFAULT")
        .whitelist_var("CL_DB_.*")

        .header("wrapper.h")

        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks));

    if cfg!(windows) {
        let clamav_source = PathBuf::from(env::var("CLAMAV_SOURCE").expect("CLAMAV_SOURCE environment variable must be set and point to ClamAV's source directory"));
        let clamav_build = PathBuf::from(env::var("CLAMAV_BUILD").expect("CLAMAV_BUILD environment variable must be set and point to ClamAV's build directory"));
        let openssl_include = PathBuf::from(env::var("OPENSSL_INCLUDE").expect("OPENSSL_INCLUDE environment variable must be set and point to openssl's include directory"));
        bindings = bindings
            .clang_arg("-I") .clang_arg(clamav_source.join("libclamav").to_str().unwrap())
            .clang_arg("-I") .clang_arg(clamav_build.to_str().unwrap())
            .clang_arg("-I") .clang_arg(openssl_include.to_str().unwrap())
    }

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    bindings
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings")
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
