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
  | 0.16.1        | 0.16.1        | 0.20.0        | 0.6.0         | 0.6.1         | 0.2.1     |
  | 0.14.1        | 0.14.1        | 0.19.4        | 0.6.0         | 0.6.1         | 0.2.1     |
  | 0.14.0        | 0.14.0        | 0.19.0        | 0.6.0         | 0.6.1         | 0.2.0     |
  | 0.13.5-newapi | 0.13.5        | 0.18.0        | 0.5.0         | 0.6.1         | 0.2.0     |
  | 0.13.2        | 0.13.5        | 0.17.5        | 0.4.4         | 0.6.1         | 0.1.0     |
  | 0.13.1        | 0.13.5        | 0.17.4        | 0.4.4         | 0.6.1         | 0.1.0     |
  | 0.13.0        | 0.13.5        | 0.17.3        | 0.4.4         | 0.6.1         | 0.1.0     |
  | 0.12.2        | 0.13.4        | 0.17.2        | 0.4.4         | 0.6.1         | 0.1.0     |
  | 0.12.1        | 0.13.4        | 0.17.1        | 0.4.4         | 0.6.1         | 0.1.0     |
  | 0.12.0        | 0.13.4        | 0.17.0        | 0.4.4         | 0.6.1         | 0.1.0     |
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

**Notice:** The minimum supported Rust version is 1.71.

## Quick Start

Here's a simple example showing how to use the WasmEdge Rust SDK to run a WebAssembly module in your Rust application.

### Add Dependencies

Add the following to your `Cargo.toml`:

```toml
[dependencies]
wasmedge-sdk = "0.16.1"
```

Or with the `standalone` feature to automatically download the WasmEdge library:

```toml
[dependencies]
wasmedge-sdk = { version = "0.16.1", features = ["standalone"] }
```

### Run a WebAssembly Function

```rust
use std::collections::HashMap;
use wasmedge_sdk::{params, Module, Store, Vm, WasmVal, wat2wasm};
use wasmedge_sdk::vm::SyncInst;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define a simple WebAssembly module with an "add" function
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $add (param $a i32) (param $b i32) (result i32)
                (i32.add (local.get $a) (local.get $b))
            )
            (export "add" (func $add))
        )
        "#,
    )?;

    // Create a VM instance
    let mut vm = Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())?);

    // Load and register the WebAssembly module
    let module = Module::from_bytes(None, wasm_bytes)?;
    vm.register_module(None, module)?;

    // Call the "add" function with arguments 2 and 3
    let result = vm.run_func(None, "add", params!(2i32, 3i32))?;

    // Get the result
    println!("2 + 3 = {}", result[0].to_i32()); // Output: 2 + 3 = 5

    Ok(())
}
```

### Load a WASM File

```rust
use std::collections::HashMap;
use wasmedge_sdk::{params, Module, Store, Vm};
use wasmedge_sdk::vm::SyncInst;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a VM instance
    let mut vm = Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())?);

    // Load a WebAssembly module from a file
    let module = Module::from_file(None, "path/to/your/module.wasm")?;
    vm.register_module(Some("my_module"), module)?;

    // Call an exported function
    let result = vm.run_func(Some("my_module"), "your_function", params!())?;

    Ok(())
}
```

For more examples, see the [WasmEdge Rust SDK Examples](https://github.com/second-state/wasmedge-rustsdk-examples) repository.

## Build and Run a Rust WASM Module

This section shows how to write a Rust library, compile it to WebAssembly, and run it in a Rust host application using the WasmEdge SDK.

### Step 1: Create the WASM Module

Create a new Rust library project for your WASM module:

```bash
cargo new --lib my-wasm-lib
cd my-wasm-lib
```

Update `Cargo.toml` to compile as a C-compatible dynamic library:

```toml
[package]
name = "my-wasm-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"   # Optimize for size
lto = true        # Enable link-time optimization
```

### Step 2: Write Exported Functions

In `src/lib.rs`, write functions with `#[no_mangle]` and `extern "C"` to export them to WebAssembly:

```rust
/// Add two numbers
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Calculate factorial
#[no_mangle]
pub extern "C" fn factorial(n: i32) -> i64 {
    if n <= 1 {
        1
    } else {
        n as i64 * factorial(n - 1)
    }
}

/// Check if a number is prime
#[no_mangle]
pub extern "C" fn is_prime(n: i32) -> i32 {
    if n <= 1 {
        return 0;
    }
    if n <= 3 {
        return 1;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return 0;
    }
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return 0;
        }
        i += 6;
    }
    1
}
```

### Step 3: Compile to WebAssembly

Install the WebAssembly target and build:

```bash
# Install the wasm32-unknown-unknown target (one-time setup)
rustup target add wasm32-unknown-unknown

# Build the WASM module
cargo build --release --target wasm32-unknown-unknown
```

The compiled WASM file will be at `target/wasm32-unknown-unknown/release/my_wasm_lib.wasm`.

### Step 4: Create the Host Application

Create a new Rust project for the host application:

```bash
cargo new my-host-app
cd my-host-app
```

Add the WasmEdge SDK to `Cargo.toml`:

```toml
[dependencies]
wasmedge-sdk = { version = "0.16.1", features = ["standalone"] }
```

### Step 5: Load and Run the WASM Module

In `src/main.rs`, load the WASM module and call its functions:

```rust
use std::collections::HashMap;
use wasmedge_sdk::{params, Module, Store, Vm, WasmVal};
use wasmedge_sdk::vm::SyncInst;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a VM instance
    let mut vm = Vm::new(Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())?);

    // Load the WASM module from file
    let module = Module::from_file(None, "../my-wasm-lib/target/wasm32-unknown-unknown/release/my_wasm_lib.wasm")?;
    vm.register_module(Some("math"), module)?;

    // Call the add function
    let result = vm.run_func(Some("math"), "add", params!(10i32, 20i32))?;
    println!("10 + 20 = {}", result[0].to_i32());

    // Call the factorial function
    let result = vm.run_func(Some("math"), "factorial", params!(10i32))?;
    println!("10! = {}", result[0].to_i64());

    // Call the is_prime function
    for n in [2, 17, 18, 97, 100] {
        let result = vm.run_func(Some("math"), "is_prime", params!(n))?;
        let is_prime = result[0].to_i32() == 1;
        println!("{} is prime: {}", n, is_prime);
    }

    Ok(())
}
```

### Step 6: Run the Host Application

```bash
cargo run
```

Expected output:
```
10 + 20 = 30
10! = 3628800
2 is prime: true
17 is prime: true
18 is prime: false
97 is prime: true
100 is prime: false
```

### Supported Types

WebAssembly supports the following primitive types that can be used as function parameters and return values:

| Rust Type | WASM Type | Description |
|-----------|-----------|-------------|
| `i32`     | `i32`     | 32-bit integer |
| `i64`     | `i64`     | 64-bit integer |
| `f32`     | `f32`     | 32-bit float |
| `f64`     | `f64`     | 64-bit float |

For more complex data types (strings, arrays, structs), you'll need to use WebAssembly linear memory. See the [WasmEdge Rust SDK Examples](https://github.com/second-state/wasmedge-rustsdk-examples) for advanced usage patterns.

## WASM Calling Host Functions

WebAssembly modules can import and call functions provided by the host application. This enables powerful interactions where WASM can use native functionality, access system resources, or maintain state in the host.

### Define Host Functions

Host functions have a specific signature:

```rust
use wasmedge_sdk::{
    error::CoreError, CallingFrame, Instance, WasmValue,
};

fn my_host_function(
    data: &mut MyHostData,        // Mutable reference to host data
    _inst: &mut Instance,         // The calling instance
    _frame: &mut CallingFrame,    // The calling frame
    args: Vec<WasmValue>,         // Arguments from WASM
) -> Result<Vec<WasmValue>, CoreError> {
    // Access arguments
    let arg1 = args[0].to_i32();

    // Do something with host data
    data.counter += arg1;

    // Return results
    Ok(vec![WasmValue::from_i32(data.counter)])
}
```

### Create Import Object with Host Functions

```rust
use std::collections::HashMap;
use wasmedge_sdk::{
    error::CoreError, params, vm::SyncInst, AsInstance, CallingFrame,
    ImportObjectBuilder, Instance, Module, Store, Vm, WasmValue,
};

// Define host data structure
#[derive(Default)]
struct HostData {
    counter: i32,
}

// Define host functions
fn host_increment(
    data: &mut HostData,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    data.counter += args[0].to_i32();
    Ok(vec![])
}

fn host_get_counter(
    data: &mut HostData,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(data.counter)])
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create host data
    let host_data = HostData::default();

    // Build import object with host functions
    let mut import_builder = ImportObjectBuilder::new("env", host_data)?;
    import_builder.with_func::<i32, ()>("increment", host_increment)?;
    import_builder.with_func::<(), i32>("get_counter", host_get_counter)?;
    let mut import_object = import_builder.build();

    // Create instances map
    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(import_object.name().unwrap().to_string(), &mut import_object);

    // Create VM with instances
    let mut vm = Vm::new(Store::new(None, instances)?);

    // Load WASM module that imports these functions
    let module = Module::from_file(None, "module_with_imports.wasm")?;
    vm.register_module(None, module)?;

    // Run WASM function that uses host functions
    let result = vm.run_func(None, "do_something", params!())?;

    Ok(())
}
```

### WASM Module with Imports (WAT)

Here's an example WASM module that imports and uses host functions:

```wat
(module
    ;; Import host functions from "env" module
    (import "env" "increment" (func $increment (param i32)))
    (import "env" "get_counter" (func $get_counter (result i32)))

    ;; WASM function that uses host functions
    (func $add_and_get (param $value i32) (result i32)
        ;; Call host increment function
        (call $increment (local.get $value))
        ;; Call host get_counter and return result
        (call $get_counter)
    )

    (export "add_and_get" (func $add_and_get))
)
```

### Use Cases for Host Functions

- **Logging and debugging**: WASM calls host to print messages
- **File I/O**: WASM requests host to read/write files
- **Network access**: WASM makes HTTP requests through host
- **State management**: Host maintains state across WASM calls
- **Native computations**: Offload heavy computations to native code
- **System integration**: Access databases, APIs, or hardware

## Upgrade to 0.14.0

If you are upgrading from 0.13.2 to 0.14.0, refer to [docs/Upgrade_to_0.14.0.md](docs/Upgrade_to_0.14.0.md).

## API Reference

- [API Reference](https://wasmedge.github.io/wasmedge-rust-sdk/wasmedge_sdk/index.html)
- [Async API Reference](https://second-state.github.io/wasmedge-async-rust-sdk/wasmedge_sdk/index.html)

## Examples

The [Examples of WasmEdge RustSDK](https://github.com/second-state/wasmedge-rustsdk-examples) repo contains a number of examples that demonstrate how to use the WasmEdge Rust SDK.

## Contributing

Please read the [contribution guidelines](https://github.com/WasmEdge/wasmedge-rust-sdk/blob/main/CONTRIBUTING.md) on how to contribute code.

## License

This project is licensed under the terms of the [Apache 2.0 license](https://github.com/tensorflow/rust/blob/HEAD/LICENSE).
