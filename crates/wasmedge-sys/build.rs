use lazy_static::lazy_static;
use phf::phf_map;

mod build_paths;
use build_paths::{Env, LibWasmEdgePaths};

mod build_standalone;
use build_standalone::*;

use crate::build_paths::AsPath;

const WASMEDGE_RELEASE_VERSION: &str = "0.13.4";
const REMOTE_ARCHIVES: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    // The key is: {os}/{arch}[/{libc}][/static]
    //  * The libc abi is only added on linux.
    //  * "static" is added when the `static` feature is enabled.
    //
    // The value is a tuple containing the sha256sum of the archive, and the platform slug as it appears in the archive name:
    //  * The archive name is WasmEdge-{version}-{slug}.tar.gz

    "macos/aarch64"                => ("db03037e2678e197f9cd5f01392ee088df07e7726849626190f2d4ec1dc693c9", "darwin_arm64"),
    "macos/x86_64"                 => ("f913ba51c69603d0c3409bbbbef1cd2a525ffccb9643fe2aea313a47c70cbea7", "darwin_x86_64"),
    "linux/aarch64/gnu"            => ("82a6d5c686b34b15abab4f28f33099bcded11446563bb2bf0d8a22811304efa1", "manylinux2014_aarch64"),
    "linux/x86_64/gnu"             => ("e3634df41375856fbf374f65e91a925c15821f78a4c93852bce5df053d1f2b2b", "manylinux2014_x86_64"),
    "linux/aarch64/gnu/static"     => ("3fb21a5dace3dd46a3f28e0a0e4473c8e22526b083aba15d875afc28b1976cc9", "debian11_aarch64_static"),
    "linux/x86_64/gnu/static"      => ("488d47c43653c0a079c034aa73e525dd60873af15967d6c1d82386862ba31baa", "debian11_x86_64_static"),
    "linux/aarch64/musl/static"    => ("9a44c495ce8d9bcfb284dc2154d76a27165ad852d3a3a16a1578de1d2a765fb8", "alpine3.16_aarch64_static"),
    "linux/x86_64/musl/static"     => ("a0ccfd944e5df5f89008a7158046e10ee84fe74406f9c44394e81f4e1ed23b02", "alpine3.16_x86_64_static"),
};

lazy_static! {

static ref SEARCH_LOCATIONS: [Option<LibWasmEdgePaths>; 11] = [
    // search in the env variables: WASMEDGE_INCLUDE_DIR, WASMEDGE_LIB_DIR
    LibWasmEdgePaths::try_from("", Env("WASMEDGE_INCLUDE_DIR"), Env("WASMEDGE_LIB_DIR")),
    // search in the env variable: WASMEDGE_DIR
    LibWasmEdgePaths::try_from(Env("WASMEDGE_DIR"), "include", "lib64"),
    LibWasmEdgePaths::try_from(Env("WASMEDGE_DIR"), "include", "lib"),
    // search in the env variable: WASMEDGE_BUILD_DIR
    LibWasmEdgePaths::try_from(Env("WASMEDGE_BUILD_DIR"), "include/api", "lib64/api"),
    LibWasmEdgePaths::try_from(Env("WASMEDGE_BUILD_DIR"), "include/api", "lib/api"),
    // search in the official docker container
    LibWasmEdgePaths::try_from(Env("HOME"), ".wasmedge/include", ".wasmedge/lib64"),
    LibWasmEdgePaths::try_from(Env("HOME"), ".wasmedge/include", ".wasmedge/lib"),
    // search in /usr/local/
    LibWasmEdgePaths::try_from("/usr/local", "include", "lib64"),
    LibWasmEdgePaths::try_from("/usr/local", "include", "lib"),
    // search in xdg
    LibWasmEdgePaths::try_from(Env("HOME"), ".local/include", ".local/lib64"),
    LibWasmEdgePaths::try_from(Env("HOME"), ".local/include", ".local/lib"),
];

static ref OUT_DIR: std::path::PathBuf = Env("OUT_DIR").expect("failed to get OUT_DIR");
static ref STANDALONE_DIR: std::path::PathBuf = OUT_DIR.join("standalone");

}

fn find_libwasmedge<'a, L: IntoIterator<Item = &'a Option<LibWasmEdgePaths>>>(
    locations: L,
) -> Option<LibWasmEdgePaths> {
    locations
        .into_iter()
        .flatten()
        .find(|paths| paths.is_wasmedge_dir())
        .cloned()
}

fn main() {
    // rerun if the other build sources change
    println!("cargo:rerun-if-changed=build_paths.rs");
    println!("cargo:rerun-if-changed=build_install.rs");

    // find the location of the libwasmedge
    let paths = if cfg!(feature = "standalone") {
        // use a standalone library from an extracted archive
        let standalone_dir = get_standalone_libwasmedge();
        debug!("using standalone extraction at {standalone_dir:?}");
        let locations = [
            LibWasmEdgePaths::try_from(&standalone_dir, "include", "lib64"),
            LibWasmEdgePaths::try_from(&standalone_dir, "include", "lib"),
        ];
        find_libwasmedge(&locations)
    } else {
        // find the library in the system
        debug!("searching for existing libwasmedge install");
        find_libwasmedge(&*SEARCH_LOCATIONS)
    };

    let paths = paths.expect("Failed to locate the required header and/or library file. Please reference the link: https://wasmedge.org/book/en/embed/rust.html");
    debug!("found libwasmedge at {paths:?}");

    let lib_dir = paths.lib_dir.to_string_lossy().to_string();

    if cfg!(feature = "static") {
        // Tell cargo to look for static libraries in the specified directory
        println!("cargo:rustc-link-search=native={lib_dir}");

        // Tell cargo to tell rustc to link our `wasmedge` library. Cargo will
        // automatically know it must look for a `libwasmedge.a` file.
        println!("cargo:rustc-link-lib=static=wasmedge");
        for dep in ["rt", "dl", "pthread", "m", "stdc++"] {
            link_lib(dep);
        }
    } else {
        println!("cargo:rustc-env=LD_LIBRARY_PATH={lib_dir}");
        println!("cargo:rustc-link-search={lib_dir}");
        println!("cargo:rustc-link-lib=dylib=wasmedge");
    }

    let inc_dir = paths.inc_dir.to_string_lossy().to_string();
    let header = paths.header().to_string_lossy().to_string();

    // Tell cargo to invalidate the built crate whenever the header changes.
    println!("cargo:rerun-if-changed={}", &header);

    let out_file = OUT_DIR.join("wasmedge.rs");

    debug!("generating bindgen header {out_file:?}");
    if let Some(bindgen_path) = Env("WASMEDGE_RUST_BINDGEN_PATH").as_path() {
        let success = std::process::Command::new(bindgen_path)
            .arg("--no-prepend-enum-name") // The API already prepends the name.
            .arg("--dynamic-link-require-all")
            .arg("--formatter=none")
            .arg("-o")
            .arg(out_file)
            .arg(header)
            .arg("--")
            .arg(format!("-I{inc_dir}"))
            .status()
            .expect("failed to run rust bindgen")
            .success();
        assert!(success, "failed to run rust bindgen");
    } else {
        bindgen::builder()
            .header(header)
            .clang_arg(format!("-I{inc_dir}"))
            .prepend_enum_name(false)
            .dynamic_link_require_all(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("failed to generate bindings")
            .write_to_file(out_file)
            .expect("failed to write bindings");
    }
}

fn link_lib(dep: &str) {
    // Sanitize dependency name for evn-vars, particularly `stdc++`.
    let dep_slug: String = dep.replace('+', "x").to_uppercase();

    let generic_link_type_var = Env!("WASMEDGE_DEPS_LINK_TYPE");
    let generic_lib_path_var = Env!("WASMEDGE_DEPS_LIB_PATH");
    let named_link_type_var = Env!("WASMEDGE_DEP_{dep_slug}_LINK_TYPE");
    let named_lib_path_var = Env!("WASMEDGE_DEP_{dep_slug}_LIB_PATH");

    let link_type = named_link_type_var
        .lossy()
        .or_else(|| generic_link_type_var.lossy())
        .unwrap_or("dylib".to_string());

    for path_var in [named_lib_path_var, generic_lib_path_var] {
        if let Some(path) = path_var.lossy() {
            println!("cargo:rustc-link-search={path}");
        }
    }

    println!("cargo:rustc-link-lib={link_type}={dep}");
}

#[macro_export]
macro_rules! debug {
    ($($args:expr),+) => {
        println!("cargo:warning=[wasmedge-sys] {}", format!($($args),+))
    };
}

#[macro_export]
macro_rules! Env {
    ($($args:expr),+) => { Env(format!($($args),+)) };
}
