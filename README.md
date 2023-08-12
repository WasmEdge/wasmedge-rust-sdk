# WasmEdge Rust SDK

WasmEdge Rust SDK provides idiomatic [Rust](https://www.rust-lang.org/) language bindings for [WasmEdge](https://wasmedge.org/)

**Notice:** This project is still under active development and not guaranteed to have a stable API.

- [WasmEdge website](https://wasmedge.org/)
- [WasmEdge Docs](https://wasmedge.org/docs/)
- [WasmEdge GitHub Page](https://github.com/WasmEdge/WasmEdge)
- [WasmEdge Rust SDK GitHub Page](https://github.com/WasmEdge/wasmedge-rust-sdk)
- [WasmEdge Rust SDK Examples](https://github.com/second-state/wasmedge-rustsdk-examples)

## Get Started

This crate depends on the WasmEdge C API. In linux/macOS the crate can download the API at build time by enabling the `standalone` feature. Otherwise the API needs to be installed in your system first. Please refer to [Install and uninstall WasmEdge](https://wasmedge.org/docs/start/install) to install the WasmEdge library. The versioning table below shows the version of the WasmEdge library required by each version of the `wasmedge-sdk` crate.

  | wasmedge-sdk  | WasmEdge lib  | wasmedge-sys  | wasmedge-types| wasmedge-macro| async-wasi|
  | :-----------: | :-----------: | :-----------: | :-----------: | :-----------: | :-------: |
  | 0.11.2        | 0.13.3        | 0.16.2        | 0.4.3         | 0.6.1         | 0.1.0     |
  | 0.11.0        | 0.13.3        | 0.16.0        | 0.4.3         | 0.6.0         | 0.0.3     |
  | 0.10.1        | 0.13.3        | 0.15.1        | 0.4.2         | 0.5.0         | 0.0.2     |
  | 0.10.0        | 0.13.2        | 0.15.0        | 0.4.2         | 0.5.0         | 0.0.2     |
  | 0.9.0         | 0.13.1        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
  | 0.9.0         | 0.13.0        | 0.14.0        | 0.4.2         | 0.4.0         | 0.0.1     |
  | 0.8.1         | 0.12.1        | 0.13.1        | 0.4.1         | 0.3.0         | -         |
  | 0.8.0         | 0.12.0        | 0.13.0        | 0.4.1         | 0.3.0         | -         |
  | 0.7.1         | 0.11.2        | 0.12.2        | 0.3.1         | 0.3.0         | -         |
  | 0.7.0         | 0.11.2        | 0.12          | 0.3.1         | 0.3.0         | -         |
  | 0.6.0         | 0.11.2        | 0.11          | 0.3.0         | 0.2.0         | -         |
  | 0.5.0         | 0.11.1        | 0.10          | 0.3.0         | 0.1.0         | -         |
  | 0.4.0         | 0.11.0        | 0.9           | 0.2.1         | -             | -         |
  | 0.3.0         | 0.10.1        | 0.8           | 0.2           | -             | -         |
  | 0.1.0         | 0.10.0        | 0.7           | 0.1           | -             | -         |

WasmEdge Rust SDK will automatically search for the WasmEdge library in your system. Alternatively you can set the `WASMEDGE_DIR` environment variable to the path of the WasmEdge library (or the `WASMEDGE_INCLUDE_DIR` and `WASMEDGE_LIB_DIR` variables for more fine-grained control). If you want to use a local `cmake` build of WasmEdge you can set the `WASMEDGE_BUILD_DIR` instead.

WasmEdge Rust SDK will search for the WasmEdge library in the following paths in order:

- `$WASMEDGE_[INCLUDE|LIB]_DIR`
- `$WASMEDGE_DIR`
- `$WASMEDGE_BUILD_DIR`
- `$HOME/.wasmedge`
- `/usr/local`
- `$HOME/.local`

When the `standalone` feature is enabled the correct library will be downloaded during build time and the previous locations are ignored. You can specify a proxy for the download process using the `WASMEDGE_STANDALONE_PROXY`, `WASMEDGE_STANDALONE_PROXY_USER` and `WASMEDGE_STANDALONE_PROXY_PASS` environment variables. You can set the `WASMEDGE_STANDALONE_ARCHIVE` environment variable to use a local archive instead of downloading one.

The following architectures are supported for automatic downloads:

  | os    | libc    | architecture        | linking type    |
  | :---: | :-----: | :-----------------: | :-------------: |
  | macos | -       | `x86_64`, `aarch64` | dynamic         |
  | linux | `glibc` | `x86_64`, `aarch64` | static, dynamic |
  | linux | `musl`  | `x86_64`, `aarch64` | static          |

This crate uses `rust-bindgen` during the build process. If you would like to use an external `rust-bindgen` you can set the `WASMEDGE_RUST_BINDGEN_PATH` environment variable to the `bindgen` executable path. This is particularly useful in systems like Alpine Linux (see [rust-lang/rust-bindgen#2360](https://github.com/rust-lang/rust-bindgen/issues/2360#issuecomment-1595869379), [rust-lang/rust-bindgen#2333](https://github.com/rust-lang/rust-bindgen/issues/2333)).

**Notice:** The minimum supported Rust version is 1.68.

## Examples

The [Examples of WasmEdge RustSDK](https://github.com/second-state/wasmedge-rustsdk-examples) repo contains a number of examples that demonstrate how to use the WasmEdge Rust SDK.

## Contributing

Please read the [contribution guidelines](https://github.com/WasmEdge/wasmedge-rust-sdk/blob/main/CONTRIBUTING.md) on how to contribute code.

## License

This project is licensed under the terms of the [Apache 2.0 license](https://github.com/tensorflow/rust/blob/HEAD/LICENSE).
