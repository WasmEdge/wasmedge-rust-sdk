//! Example: Run a WebAssembly module
//!
//! This example demonstrates loading and running a WebAssembly module using the WasmEdge SDK.
//! When built with the `bundled` feature, the WasmEdge library is statically linked,
//! creating a self-contained binary that doesn't require any runtime library paths.
//!
//! Usage:
//!   cargo build --release --features bundled --example run_wasm
//!   ./target/release/examples/run_wasm <path_to_wasm_file>
//!
//! Example:
//!   ./target/release/examples/run_wasm test-wasm/target/wasm32-unknown-unknown/release/test_wasm.wasm

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use wasmedge_sdk::vm::SyncInst;
use wasmedge_sdk::{params, Module, Store, Vm, WasmVal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <wasm_file> [function_name] [args...]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} module.wasm                    # List exported functions", args[0]);
        eprintln!("  {} module.wasm add 2 3            # Call 'add' with args 2 and 3", args[0]);
        eprintln!("  {} module.wasm factorial 10       # Call 'factorial' with arg 10", args[0]);
        std::process::exit(1);
    }

    let wasm_path = PathBuf::from(&args[1]);
    if !wasm_path.exists() {
        eprintln!("Error: WASM file not found: {}", wasm_path.display());
        std::process::exit(1);
    }

    println!("Loading WASM module: {}", wasm_path.display());

    // Create a VM instance
    let mut vm = Vm::new(Store::new(
        None,
        HashMap::<String, &mut dyn SyncInst>::new(),
    )?);

    // Load the WebAssembly module
    let module = Module::from_file(None, &wasm_path)?;

    // Get and display exported functions
    let exports: Vec<_> = module.exports().collect();
    println!("Exported functions:");
    for export in &exports {
        println!("  - {}", export.name());
    }

    // Register the module
    vm.register_module(Some("wasm"), module)?;

    // If a function name is provided, call it
    if args.len() >= 3 {
        let func_name = &args[2];
        println!("\nCalling function: {}", func_name);

        // Parse integer arguments (simple i32 parsing for demo)
        let func_args: Vec<i32> = args[3..]
            .iter()
            .filter_map(|s| s.parse::<i32>().ok())
            .collect();

        println!("Arguments: {:?}", func_args);

        // Build params based on argument count
        let result = match func_args.len() {
            0 => vm.run_func(Some("wasm"), func_name, params!())?,
            1 => vm.run_func(Some("wasm"), func_name, params!(func_args[0]))?,
            2 => vm.run_func(Some("wasm"), func_name, params!(func_args[0], func_args[1]))?,
            3 => vm.run_func(
                Some("wasm"),
                func_name,
                params!(func_args[0], func_args[1], func_args[2]),
            )?,
            _ => {
                eprintln!("Too many arguments (max 3 supported in this demo)");
                std::process::exit(1);
            }
        };

        // Display results
        if result.is_empty() {
            println!("Result: (no return value)");
        } else {
            print!("Result: ");
            for (i, val) in result.iter().enumerate() {
                if i > 0 {
                    print!(", ");
                }
                // Try to display as different types
                if let Some(v) = val.to_i32().checked_add(0) {
                    print!("{} (i32)", v);
                } else if let Some(v) = val.to_i64().checked_add(0) {
                    print!("{} (i64)", v);
                } else {
                    print!("{:?}", val);
                }
            }
            println!();
        }
    }

    println!("\nDone!");
    Ok(())
}
