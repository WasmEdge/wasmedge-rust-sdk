# WasmEdge Rust SDK

WasmEdge Rust SDK provides idiomatic [Rust](https://www.rust-lang.org/) language bindings for [WasmEdge](https://wasmedge.org/)

**Notice:** This project is still under active development and not guaranteed to have a stable API.

- [Documentation](https://wasmedge.org/docs/)
- [WasmEdge website](https://wasmedge.org/)
- [WasmEdge GitHub Page](https://github.com/WasmEdge/WasmEdge)
- [WasmEdge Rust SDK GitHub Page](https://github.com/WasmEdge/wasmedge-rust-sdk)
- [WasmEdge Rust SDK Examples](https://github.com/second-state/wasmedge-rustsdk-examples)

## Get Started

Since this crate depends on the WasmEdge C API, it needs to be installed in your system first. Please refer to [WasmEdge Installation and Uninstallation](https://wasmedge.org/book/en/quick_start/install.html) to install the WasmEdge library. The versioning table below shows the version of the WasmEdge library required by each version of the `wasmedge-sdk` crate.

  | wasmedge-sdk  | WasmEdge lib  | wasmedge-sys  | wasmedge-types| wasmedge-macro| async-wasi|
  | :-----------: | :-----------: | :-----------: | :-----------: | :-----------: | :-------: |
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

WasmEdge Rust SDK can automatically search the following paths for the WasmEdge library:

- `$HOME/.wasmedge` (Linux/macOS)
- `/usr/local` (Linux/macOS)

If you have installed the WasmEdge library in a different path, you can set the `WASMEDGE_INCLUDE_DIR` and `WASMEDGE_LIB_DIR` environment variables to the path of the WasmEdge library.

**Notice:** The minimum supported Rust version is 1.68.

## Examples

The [Examples of WasmEdge RustSDK](https://github.com/second-state/wasmedge-rustsdk-examples) repo contains a number of examples that demonstrate how to use the WasmEdge Rust SDK.

## Contributing

Please read the [contribution guidelines](https://github.com/WasmEdge/wasmedge-rust-sdk/blob/main/CONTRIBUTING.md) on how to contribute code.

## License

This project is licensed under the terms of the [Apache 2.0 license](https://github.com/tensorflow/rust/blob/HEAD/LICENSE).
