use lazy_static::lazy_static;
use phf::phf_map;

mod build_paths;
use build_paths::{Env, LibWasmEdgePaths};

mod build_standalone;
use build_standalone::*;

use crate::build_paths::AsPath;

const WASMEDGE_RELEASE_VERSION: &str = "0.17.0";
const REMOTE_ARCHIVES: phf::Map<&'static str, (&'static str, &'static str)> = phf_map! {
    // The key is: {os}/{arch}[/{libc}][/static]
    //  * The libc abi is only added on linux.
    //  * "static" is added when the `static` feature is enabled.
    //
    // The value is a tuple containing the sha256sum of the archive, and the platform slug as it appears in the archive name:
    //  * The archive name is WasmEdge-{version}-{slug}.tar.gz

    "macos/aarch64"                => ("ae97ff792ac1bf7bcf703b20926b9bac168e5b6260930d13a156dc19e68a67c5", "darwin_arm64"),
    "macos/aarch64/static"         => ("57c22c699c1bc7b10c6da78e3b14b74d578a392c8019250e58d00da61857b203", "darwin_arm64_static"),
    "macos/x86_64"                 => ("5742f7d19bbdb983f4df57114b085f908bae06f705ea3e167f802a66bd9e0342", "darwin_x86_64"),
    "linux/aarch64/gnu"            => ("6d3aa5a43fd0998b11812e99b46e90a282f3caaacc9cfef14b67f3438b63e804", "manylinux_2_28_aarch64"),
    "linux/x86_64/gnu"             => ("5d8165559c553eacc9b87db1799c2204e056db8609bedbf61eb29f8a21a42993", "manylinux_2_28_x86_64"),
    "linux/aarch64/gnu/static"     => ("93c28c7caf84860ad9acb36c823a820d0abc062af432749c5eab4395e5158beb", "debian11_aarch64_static"),
    "linux/x86_64/gnu/static"      => ("7603ce19b745984c73057887219e5635cb08f857a1fc4625ba30347dbc6effe1", "debian11_x86_64_static"),
    "linux/aarch64/musl/static"    => ("415b8660fb11cb25f7c8a5d67a88a1ad9802eb832ca3dd65d363fc47b01faa35", "alpine3.23_aarch64_static"),
    "linux/x86_64/musl/static"     => ("4e285eb9e94f147e8d11b53209a3b7377265ab9dd784c8ac8be7682ea3531e44", "alpine3.23_x86_64_static"),
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

        // Tell cargo to tell rustc to link our `wasmedge` library statically.
        println!("cargo:rustc-link-lib=static=wasmedge");

        // fmt: macOS static archive merges fmt into libwasmedge.a and does not
        // ship libfmt.a separately. macOS developer hosts typically lack
        // libfmt.dylib on the linker's default search path (brew installs to
        // /opt/homebrew/lib which clang doesn't search by default), so the
        // dynamic fallback breaks clean macOS builds. Skip the explicit fmt
        // link on macOS — the symbols are already in libwasmedge.a.
        let fmt_static = std::path::Path::new(&lib_dir).join("libfmt.a");
        if fmt_static.exists() {
            debug!("found static libfmt at {fmt_static:?}");
            println!("cargo:rustc-link-lib=static=fmt");
        } else if !cfg!(target_os = "macos") {
            debug!("static libfmt not found, linking dynamically");
            println!("cargo:rustc-link-lib=dylib=fmt");
        } else {
            debug!(
                "static libfmt not found on macOS; skipping (assumed merged into libwasmedge.a)"
            );
        }

        // Platform-conditional system deps for the static-link path.
        // macOS: libSystem already provides rt/dl/pthread/m; libstdc++ is
        //        libc++. When the archive was built with
        //        WASMEDGE_LINK_LLVM_STATIC=ON (default for the darwin_arm64
        //        static workflow), LLVM's compression code pulls in zlib
        //        (libz), terminfo support pulls in ncurses, and archive
        //        parsing pulls in libxar — all available as /usr/lib system
        //        libraries on every macOS install.
        // Linux: needs the full set.
        let deps: &[&str] = if cfg!(target_os = "macos") {
            &["c++", "z", "ncurses", "xar"]
        } else {
            &["rt", "dl", "pthread", "m", "stdc++"]
        };
        for dep in deps {
            link_lib(dep);
        }

        // zstd: mirror the libfmt.a fallback pattern. macOS static archive
        // won't ship libzstd.a; fall back to skipping rather than dynamic-linking
        // a library the host may not have.
        let zstd_static = std::path::Path::new(&lib_dir).join("libzstd.a");
        if zstd_static.exists() {
            debug!("found static libzstd at {zstd_static:?}");
            println!("cargo:rustc-link-lib=static=zstd");
        } else if !cfg!(target_os = "macos") {
            debug!("static libzstd not found, linking dynamically");
            println!("cargo:rustc-link-lib=dylib=zstd");
        } else {
            debug!(
                "static libzstd not found on macOS; skipping (assumed merged into libwasmedge.a)"
            );
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
            .arg("--allowlist-item")
            .arg("WasmEdge.*")
            .arg("--no-layout-tests")
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
            .allowlist_item("WasmEdge.*")
            .layout_tests(false)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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
