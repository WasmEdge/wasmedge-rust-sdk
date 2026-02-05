//! Integration tests for WasmEdge Rust SDK
//!
//! This module tests loading and running WebAssembly modules with the SDK.

use std::collections::HashMap;
use std::path::PathBuf;
use wasmedge_sdk::error::CoreError;
use wasmedge_sdk::vm::SyncInst;
use wasmedge_sdk::AsInstance;
use wasmedge_sdk::{
    params, wat2wasm, CallingFrame, ImportObjectBuilder, Instance, Module, Store, Vm, WasmVal,
    WasmValue,
};

/// Get path to the compiled test-wasm module
fn get_test_wasm_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
        .join("test-wasm")
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("test_wasm.wasm")
}

/// Test running a simple add function
#[test]
fn test_add_function() {
    // Define a WebAssembly module with an "add" function
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $add (param $a i32) (param $b i32) (result i32)
                (i32.add (local.get $a) (local.get $b))
            )
            (export "add" (func $add))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    // Create a VM instance
    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    // Load and register the WebAssembly module
    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test case 1: 2 + 3 = 5
    let result = vm
        .run_func(None, "add", params!(2i32, 3i32))
        .expect("Failed to run add function");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].to_i32(), 5);

    // Test case 2: 100 + 200 = 300
    let result = vm
        .run_func(None, "add", params!(100i32, 200i32))
        .expect("Failed to run add function");
    assert_eq!(result[0].to_i32(), 300);

    // Test case 3: -10 + 25 = 15
    let result = vm
        .run_func(None, "add", params!(-10i32, 25i32))
        .expect("Failed to run add function");
    assert_eq!(result[0].to_i32(), 15);

    // Test case 4: 0 + 0 = 0
    let result = vm
        .run_func(None, "add", params!(0i32, 0i32))
        .expect("Failed to run add function");
    assert_eq!(result[0].to_i32(), 0);
}

/// Test running a fibonacci function
#[test]
fn test_fibonacci_function() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $fib (param $n i32) (result i32)
                (if (result i32)
                    (i32.lt_s (local.get $n) (i32.const 2))
                    (then (local.get $n))
                    (else
                        (i32.add
                            (call $fib (i32.sub (local.get $n) (i32.const 1)))
                            (call $fib (i32.sub (local.get $n) (i32.const 2)))
                        )
                    )
                )
            )
            (export "fib" (func $fib))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test fibonacci sequence: 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89
    let expected = vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89];
    for (n, expected_val) in expected.iter().enumerate() {
        let result = vm
            .run_func(None, "fib", params!(n as i32))
            .expect("Failed to run fib function");
        assert_eq!(
            result[0].to_i32(),
            *expected_val,
            "fib({}) should be {}",
            n,
            expected_val
        );
    }
}

/// Test running a factorial function
#[test]
fn test_factorial_function() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $factorial (param $n i32) (result i32)
                (if (result i32)
                    (i32.le_s (local.get $n) (i32.const 1))
                    (then (i32.const 1))
                    (else
                        (i32.mul
                            (local.get $n)
                            (call $factorial (i32.sub (local.get $n) (i32.const 1)))
                        )
                    )
                )
            )
            (export "factorial" (func $factorial))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test factorial: 0! = 1, 1! = 1, 5! = 120, 10! = 3628800
    let test_cases = vec![
        (0, 1),
        (1, 1),
        (2, 2),
        (3, 6),
        (4, 24),
        (5, 120),
        (10, 3628800),
    ];
    for (n, expected) in test_cases {
        let result = vm
            .run_func(None, "factorial", params!(n))
            .expect("Failed to run factorial function");
        assert_eq!(
            result[0].to_i32(),
            expected,
            "factorial({}) should be {}",
            n,
            expected
        );
    }
}

/// Test memory operations
#[test]
fn test_memory_operations() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (memory 1)
            (func $store (param $addr i32) (param $value i32)
                (i32.store (local.get $addr) (local.get $value))
            )
            (func $load (param $addr i32) (result i32)
                (i32.load (local.get $addr))
            )
            (export "store" (func $store))
            (export "load" (func $load))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Store value 42 at address 0
    vm.run_func(None, "store", params!(0i32, 42i32))
        .expect("Failed to run store function");

    // Load value from address 0
    let result = vm
        .run_func(None, "load", params!(0i32))
        .expect("Failed to run load function");
    assert_eq!(result[0].to_i32(), 42);

    // Store another value at a different address
    vm.run_func(None, "store", params!(100i32, 12345i32))
        .expect("Failed to run store function");

    let result = vm
        .run_func(None, "load", params!(100i32))
        .expect("Failed to run load function");
    assert_eq!(result[0].to_i32(), 12345);
}

/// Test global variables
#[test]
fn test_global_variables() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (global $counter (mut i32) (i32.const 0))
            (func $get_counter (result i32)
                (global.get $counter)
            )
            (func $increment
                (global.set $counter
                    (i32.add (global.get $counter) (i32.const 1))
                )
            )
            (func $add_to_counter (param $value i32)
                (global.set $counter
                    (i32.add (global.get $counter) (local.get $value))
                )
            )
            (export "get_counter" (func $get_counter))
            (export "increment" (func $increment))
            (export "add_to_counter" (func $add_to_counter))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Initial counter value should be 0
    let result = vm
        .run_func(None, "get_counter", params!())
        .expect("Failed to run get_counter");
    assert_eq!(result[0].to_i32(), 0);

    // Increment counter
    vm.run_func(None, "increment", params!())
        .expect("Failed to run increment");
    let result = vm
        .run_func(None, "get_counter", params!())
        .expect("Failed to run get_counter");
    assert_eq!(result[0].to_i32(), 1);

    // Increment again
    vm.run_func(None, "increment", params!())
        .expect("Failed to run increment");
    let result = vm
        .run_func(None, "get_counter", params!())
        .expect("Failed to run get_counter");
    assert_eq!(result[0].to_i32(), 2);

    // Add 10 to counter
    vm.run_func(None, "add_to_counter", params!(10i32))
        .expect("Failed to run add_to_counter");
    let result = vm
        .run_func(None, "get_counter", params!())
        .expect("Failed to run get_counter");
    assert_eq!(result[0].to_i32(), 12);
}

/// Test named module registration
#[test]
fn test_named_module() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $multiply (param $a i32) (param $b i32) (result i32)
                (i32.mul (local.get $a) (local.get $b))
            )
            (export "multiply" (func $multiply))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(Some("math"), module)
        .expect("Failed to register named module");

    // Call function from named module
    let result = vm
        .run_func(Some("math"), "multiply", params!(7i32, 6i32))
        .expect("Failed to run multiply function");
    assert_eq!(result[0].to_i32(), 42);

    let result = vm
        .run_func(Some("math"), "multiply", params!(123i32, 456i32))
        .expect("Failed to run multiply function");
    assert_eq!(result[0].to_i32(), 56088);
}

/// Test multiple return values
#[test]
fn test_multiple_return_values() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $divmod (param $a i32) (param $b i32) (result i32 i32)
                (i32.div_s (local.get $a) (local.get $b))
                (i32.rem_s (local.get $a) (local.get $b))
            )
            (export "divmod" (func $divmod))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // 17 / 5 = 3 remainder 2
    let result = vm
        .run_func(None, "divmod", params!(17i32, 5i32))
        .expect("Failed to run divmod function");
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].to_i32(), 3); // quotient
    assert_eq!(result[1].to_i32(), 2); // remainder

    // 100 / 7 = 14 remainder 2
    let result = vm
        .run_func(None, "divmod", params!(100i32, 7i32))
        .expect("Failed to run divmod function");
    assert_eq!(result[0].to_i32(), 14);
    assert_eq!(result[1].to_i32(), 2);
}

/// Test i64 operations
#[test]
fn test_i64_operations() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $add64 (param $a i64) (param $b i64) (result i64)
                (i64.add (local.get $a) (local.get $b))
            )
            (func $mul64 (param $a i64) (param $b i64) (result i64)
                (i64.mul (local.get $a) (local.get $b))
            )
            (export "add64" (func $add64))
            (export "mul64" (func $mul64))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test large number addition
    let result = vm
        .run_func(
            None,
            "add64",
            params!(1_000_000_000_000i64, 2_000_000_000_000i64),
        )
        .expect("Failed to run add64 function");
    assert_eq!(result[0].to_i64(), 3_000_000_000_000i64);

    // Test large number multiplication
    let result = vm
        .run_func(None, "mul64", params!(1_000_000i64, 1_000_000i64))
        .expect("Failed to run mul64 function");
    assert_eq!(result[0].to_i64(), 1_000_000_000_000i64);
}

/// Test f32 floating point operations
#[test]
fn test_f32_operations() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $add_f32 (param $a f32) (param $b f32) (result f32)
                (f32.add (local.get $a) (local.get $b))
            )
            (func $mul_f32 (param $a f32) (param $b f32) (result f32)
                (f32.mul (local.get $a) (local.get $b))
            )
            (export "add_f32" (func $add_f32))
            (export "mul_f32" (func $mul_f32))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test f32 addition: 1.5 + 2.5 = 4.0
    let result = vm
        .run_func(None, "add_f32", params!(1.5f32, 2.5f32))
        .expect("Failed to run add_f32 function");
    assert!((result[0].to_f32() - 4.0f32).abs() < 0.0001);

    // Test f32 multiplication: 3.0 * 4.0 = 12.0
    let result = vm
        .run_func(None, "mul_f32", params!(3.0f32, 4.0f32))
        .expect("Failed to run mul_f32 function");
    assert!((result[0].to_f32() - 12.0f32).abs() < 0.0001);
}

/// Test f64 floating point operations
#[test]
fn test_f64_operations() {
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (func $add_f64 (param $a f64) (param $b f64) (result f64)
                (f64.add (local.get $a) (local.get $b))
            )
            (func $sqrt_f64 (param $a f64) (result f64)
                (f64.sqrt (local.get $a))
            )
            (export "add_f64" (func $add_f64))
            (export "sqrt_f64" (func $sqrt_f64))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    let mut vm = Vm::new(
        Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
            .expect("Failed to create store"),
    );

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test f64 addition: pi + e approximately
    let result = vm
        .run_func(
            None,
            "add_f64",
            params!(std::f64::consts::PI, std::f64::consts::E),
        )
        .expect("Failed to run add_f64 function");
    let expected = std::f64::consts::PI + std::f64::consts::E;
    assert!((result[0].to_f64() - expected).abs() < 0.0000001);

    // Test f64 sqrt: sqrt(2.0) approximately 1.41421356
    let result = vm
        .run_func(None, "sqrt_f64", params!(2.0f64))
        .expect("Failed to run sqrt_f64 function");
    assert!((result[0].to_f64() - std::f64::consts::SQRT_2).abs() < 0.0000001);
}

// ============================================================================
// Tests for compiled Rust WASM module (test-wasm)
// ============================================================================

/// Macro to set up a VM with the test-wasm module loaded
macro_rules! setup_test_wasm_vm {
    ($vm:ident) => {
        let wasm_path = get_test_wasm_path();
        if !wasm_path.exists() {
            eprintln!(
                "WASM module not found at {:?}. Run: cd test-wasm && cargo build --release --target wasm32-unknown-unknown",
                wasm_path
            );
            return;
        }

        let mut $vm = Vm::new(
            Store::new(None, HashMap::<String, &mut dyn SyncInst>::new())
                .expect("Failed to create store"),
        );

        let module = Module::from_file(None, &wasm_path).expect("Failed to load test-wasm module");
        $vm.register_module(Some("test"), module)
            .expect("Failed to register test-wasm module");
    };
}

/// Test basic arithmetic operations from compiled Rust WASM
#[test]
fn test_rust_wasm_arithmetic() {
    setup_test_wasm_vm!(vm);

    // Test add
    let result = vm
        .run_func(Some("test"), "add", params!(10i32, 20i32))
        .expect("Failed to run add");
    assert_eq!(result[0].to_i32(), 30);

    // Test subtract
    let result = vm
        .run_func(Some("test"), "subtract", params!(50i32, 30i32))
        .expect("Failed to run subtract");
    assert_eq!(result[0].to_i32(), 20);

    // Test multiply
    let result = vm
        .run_func(Some("test"), "multiply", params!(7i32, 8i32))
        .expect("Failed to run multiply");
    assert_eq!(result[0].to_i32(), 56);

    // Test divide
    let result = vm
        .run_func(Some("test"), "divide", params!(100i32, 5i32))
        .expect("Failed to run divide");
    assert_eq!(result[0].to_i32(), 20);

    // Test divide by zero returns 0
    let result = vm
        .run_func(Some("test"), "divide", params!(100i32, 0i32))
        .expect("Failed to run divide");
    assert_eq!(result[0].to_i32(), 0);

    // Test modulo
    let result = vm
        .run_func(Some("test"), "modulo", params!(17i32, 5i32))
        .expect("Failed to run modulo");
    assert_eq!(result[0].to_i32(), 2);
}

/// Test 64-bit integer operations from compiled Rust WASM
#[test]
fn test_rust_wasm_i64_operations() {
    setup_test_wasm_vm!(vm);

    // Test add_i64
    let result = vm
        .run_func(
            Some("test"),
            "add_i64",
            params!(1_000_000_000_000i64, 2_000_000_000_000i64),
        )
        .expect("Failed to run add_i64");
    assert_eq!(result[0].to_i64(), 3_000_000_000_000i64);

    // Test multiply_i64
    let result = vm
        .run_func(
            Some("test"),
            "multiply_i64",
            params!(1_000_000i64, 1_000i64),
        )
        .expect("Failed to run multiply_i64");
    assert_eq!(result[0].to_i64(), 1_000_000_000i64);
}

/// Test floating point operations from compiled Rust WASM
#[test]
fn test_rust_wasm_float_operations() {
    setup_test_wasm_vm!(vm);

    // Test add_f32
    let result = vm
        .run_func(Some("test"), "add_f32", params!(1.5f32, 2.5f32))
        .expect("Failed to run add_f32");
    assert!((result[0].to_f32() - 4.0f32).abs() < 0.0001);

    // Test multiply_f32
    let result = vm
        .run_func(Some("test"), "multiply_f32", params!(3.0f32, 4.0f32))
        .expect("Failed to run multiply_f32");
    assert!((result[0].to_f32() - 12.0f32).abs() < 0.0001);

    // Test add_f64
    let result = vm
        .run_func(Some("test"), "add_f64", params!(1.5f64, 2.5f64))
        .expect("Failed to run add_f64");
    assert!((result[0].to_f64() - 4.0f64).abs() < 0.0000001);

    // Test multiply_f64
    let result = vm
        .run_func(Some("test"), "multiply_f64", params!(3.0f64, 4.0f64))
        .expect("Failed to run multiply_f64");
    assert!((result[0].to_f64() - 12.0f64).abs() < 0.0000001);

    // Test sqrt_f64
    let result = vm
        .run_func(Some("test"), "sqrt_f64", params!(16.0f64))
        .expect("Failed to run sqrt_f64");
    assert!((result[0].to_f64() - 4.0f64).abs() < 0.0000001);

    // Test pow_f64: 2^10 = 1024
    let result = vm
        .run_func(Some("test"), "pow_f64", params!(2.0f64, 10.0f64))
        .expect("Failed to run pow_f64");
    assert!((result[0].to_f64() - 1024.0f64).abs() < 0.0000001);
}

/// Test recursive functions from compiled Rust WASM
#[test]
fn test_rust_wasm_recursive_functions() {
    setup_test_wasm_vm!(vm);

    // Test factorial
    let test_cases = vec![(0, 1i64), (1, 1), (5, 120), (10, 3628800)];
    for (n, expected) in test_cases {
        let result = vm
            .run_func(Some("test"), "factorial", params!(n))
            .expect("Failed to run factorial");
        assert_eq!(
            result[0].to_i64(),
            expected,
            "factorial({}) should be {}",
            n,
            expected
        );
    }

    // Test fibonacci
    let fib_cases = vec![(0, 0i64), (1, 1), (2, 1), (5, 5), (10, 55)];
    for (n, expected) in fib_cases {
        let result = vm
            .run_func(Some("test"), "fibonacci", params!(n))
            .expect("Failed to run fibonacci");
        assert_eq!(
            result[0].to_i64(),
            expected,
            "fibonacci({}) should be {}",
            n,
            expected
        );
    }

    // Test fibonacci_iter (iterative version)
    for (n, expected) in vec![(0, 0i64), (1, 1), (10, 55), (20, 6765)] {
        let result = vm
            .run_func(Some("test"), "fibonacci_iter", params!(n))
            .expect("Failed to run fibonacci_iter");
        assert_eq!(
            result[0].to_i64(),
            expected,
            "fibonacci_iter({}) should be {}",
            n,
            expected
        );
    }
}

/// Test bitwise operations from compiled Rust WASM
#[test]
fn test_rust_wasm_bitwise_operations() {
    setup_test_wasm_vm!(vm);

    // Test bitwise_and: 0b1100 & 0b1010 = 0b1000 = 8
    let result = vm
        .run_func(Some("test"), "bitwise_and", params!(12i32, 10i32))
        .expect("Failed to run bitwise_and");
    assert_eq!(result[0].to_i32(), 8);

    // Test bitwise_or: 0b1100 | 0b1010 = 0b1110 = 14
    let result = vm
        .run_func(Some("test"), "bitwise_or", params!(12i32, 10i32))
        .expect("Failed to run bitwise_or");
    assert_eq!(result[0].to_i32(), 14);

    // Test bitwise_xor: 0b1100 ^ 0b1010 = 0b0110 = 6
    let result = vm
        .run_func(Some("test"), "bitwise_xor", params!(12i32, 10i32))
        .expect("Failed to run bitwise_xor");
    assert_eq!(result[0].to_i32(), 6);

    // Test bitwise_not: !0 = -1 (all bits set)
    let result = vm
        .run_func(Some("test"), "bitwise_not", params!(0i32))
        .expect("Failed to run bitwise_not");
    assert_eq!(result[0].to_i32(), -1);

    // Test shift_left: 1 << 4 = 16
    let result = vm
        .run_func(Some("test"), "shift_left", params!(1i32, 4i32))
        .expect("Failed to run shift_left");
    assert_eq!(result[0].to_i32(), 16);

    // Test shift_right: 16 >> 2 = 4
    let result = vm
        .run_func(Some("test"), "shift_right", params!(16i32, 2i32))
        .expect("Failed to run shift_right");
    assert_eq!(result[0].to_i32(), 4);
}

/// Test comparison operations from compiled Rust WASM
#[test]
fn test_rust_wasm_comparison_operations() {
    setup_test_wasm_vm!(vm);

    // Test greater_than
    let result = vm
        .run_func(Some("test"), "greater_than", params!(10i32, 5i32))
        .expect("Failed to run greater_than");
    assert_eq!(result[0].to_i32(), 1);

    let result = vm
        .run_func(Some("test"), "greater_than", params!(5i32, 10i32))
        .expect("Failed to run greater_than");
    assert_eq!(result[0].to_i32(), 0);

    // Test less_than
    let result = vm
        .run_func(Some("test"), "less_than", params!(5i32, 10i32))
        .expect("Failed to run less_than");
    assert_eq!(result[0].to_i32(), 1);

    // Test equals
    let result = vm
        .run_func(Some("test"), "equals", params!(42i32, 42i32))
        .expect("Failed to run equals");
    assert_eq!(result[0].to_i32(), 1);

    let result = vm
        .run_func(Some("test"), "equals", params!(42i32, 43i32))
        .expect("Failed to run equals");
    assert_eq!(result[0].to_i32(), 0);

    // Test max
    let result = vm
        .run_func(Some("test"), "max", params!(10i32, 20i32))
        .expect("Failed to run max");
    assert_eq!(result[0].to_i32(), 20);

    // Test min
    let result = vm
        .run_func(Some("test"), "min", params!(10i32, 20i32))
        .expect("Failed to run min");
    assert_eq!(result[0].to_i32(), 10);

    // Test clamp
    let result = vm
        .run_func(Some("test"), "clamp", params!(15i32, 10i32, 20i32))
        .expect("Failed to run clamp");
    assert_eq!(result[0].to_i32(), 15);

    let result = vm
        .run_func(Some("test"), "clamp", params!(5i32, 10i32, 20i32))
        .expect("Failed to run clamp");
    assert_eq!(result[0].to_i32(), 10);

    let result = vm
        .run_func(Some("test"), "clamp", params!(25i32, 10i32, 20i32))
        .expect("Failed to run clamp");
    assert_eq!(result[0].to_i32(), 20);
}

/// Test mathematical functions from compiled Rust WASM
#[test]
fn test_rust_wasm_math_functions() {
    setup_test_wasm_vm!(vm);

    // Test abs
    let result = vm
        .run_func(Some("test"), "abs", params!(-42i32))
        .expect("Failed to run abs");
    assert_eq!(result[0].to_i32(), 42);

    let result = vm
        .run_func(Some("test"), "abs", params!(42i32))
        .expect("Failed to run abs");
    assert_eq!(result[0].to_i32(), 42);

    // Test abs_f64
    let result = vm
        .run_func(Some("test"), "abs_f64", params!(-3.14f64))
        .expect("Failed to run abs_f64");
    assert!((result[0].to_f64() - 3.14f64).abs() < 0.0000001);

    // Test is_prime
    let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
    for p in primes {
        let result = vm
            .run_func(Some("test"), "is_prime", params!(p))
            .expect("Failed to run is_prime");
        assert_eq!(result[0].to_i32(), 1, "{} should be prime", p);
    }

    let non_primes = vec![0, 1, 4, 6, 8, 9, 10, 12, 15, 100];
    for np in non_primes {
        let result = vm
            .run_func(Some("test"), "is_prime", params!(np))
            .expect("Failed to run is_prime");
        assert_eq!(result[0].to_i32(), 0, "{} should not be prime", np);
    }

    // Test gcd
    let result = vm
        .run_func(Some("test"), "gcd", params!(48i32, 18i32))
        .expect("Failed to run gcd");
    assert_eq!(result[0].to_i32(), 6);

    let result = vm
        .run_func(Some("test"), "gcd", params!(100i32, 35i32))
        .expect("Failed to run gcd");
    assert_eq!(result[0].to_i32(), 5);

    // Test lcm
    let result = vm
        .run_func(Some("test"), "lcm", params!(4i32, 6i32))
        .expect("Failed to run lcm");
    assert_eq!(result[0].to_i32(), 12);

    // Test sum_of_digits
    let result = vm
        .run_func(Some("test"), "sum_of_digits", params!(12345i32))
        .expect("Failed to run sum_of_digits");
    assert_eq!(result[0].to_i32(), 15);

    // Test count_digits
    let result = vm
        .run_func(Some("test"), "count_digits", params!(12345i32))
        .expect("Failed to run count_digits");
    assert_eq!(result[0].to_i32(), 5);

    // Test reverse_number
    let result = vm
        .run_func(Some("test"), "reverse_number", params!(12345i32))
        .expect("Failed to run reverse_number");
    assert_eq!(result[0].to_i32(), 54321);

    // Test is_palindrome
    let result = vm
        .run_func(Some("test"), "is_palindrome", params!(12321i32))
        .expect("Failed to run is_palindrome");
    assert_eq!(result[0].to_i32(), 1);

    let result = vm
        .run_func(Some("test"), "is_palindrome", params!(12345i32))
        .expect("Failed to run is_palindrome");
    assert_eq!(result[0].to_i32(), 0);
}

/// Test summation functions from compiled Rust WASM
#[test]
fn test_rust_wasm_summation_functions() {
    setup_test_wasm_vm!(vm);

    // Test sum_natural: 1+2+3+...+10 = 55
    let result = vm
        .run_func(Some("test"), "sum_natural", params!(10i32))
        .expect("Failed to run sum_natural");
    assert_eq!(result[0].to_i64(), 55);

    // Test sum_squares: 1^2+2^2+...+10^2 = 385
    let result = vm
        .run_func(Some("test"), "sum_squares", params!(10i32))
        .expect("Failed to run sum_squares");
    assert_eq!(result[0].to_i64(), 385);

    // Test sum_cubes: 1^3+2^3+...+10^3 = 3025
    let result = vm
        .run_func(Some("test"), "sum_cubes", params!(10i32))
        .expect("Failed to run sum_cubes");
    assert_eq!(result[0].to_i64(), 3025);

    // Test triangular_number
    let result = vm
        .run_func(Some("test"), "triangular_number", params!(10i32))
        .expect("Failed to run triangular_number");
    assert_eq!(result[0].to_i64(), 55);

    // Test power_of_two: 2^10 = 1024
    let result = vm
        .run_func(Some("test"), "power_of_two", params!(10i32))
        .expect("Failed to run power_of_two");
    assert_eq!(result[0].to_i64(), 1024);
}

/// Test type conversion functions from compiled Rust WASM
#[test]
fn test_rust_wasm_type_conversions() {
    setup_test_wasm_vm!(vm);

    // Test i32_to_i64
    let result = vm
        .run_func(Some("test"), "i32_to_i64", params!(42i32))
        .expect("Failed to run i32_to_i64");
    assert_eq!(result[0].to_i64(), 42i64);

    // Test i64_to_i32
    let result = vm
        .run_func(Some("test"), "i64_to_i32", params!(42i64))
        .expect("Failed to run i64_to_i32");
    assert_eq!(result[0].to_i32(), 42i32);

    // Test f32_to_i32
    let result = vm
        .run_func(Some("test"), "f32_to_i32", params!(3.7f32))
        .expect("Failed to run f32_to_i32");
    assert_eq!(result[0].to_i32(), 3);

    // Test i32_to_f32
    let result = vm
        .run_func(Some("test"), "i32_to_f32", params!(42i32))
        .expect("Failed to run i32_to_f32");
    assert!((result[0].to_f32() - 42.0f32).abs() < 0.0001);

    // Test f64_to_i64
    let result = vm
        .run_func(Some("test"), "f64_to_i64", params!(3.7f64))
        .expect("Failed to run f64_to_i64");
    assert_eq!(result[0].to_i64(), 3i64);

    // Test i64_to_f64
    let result = vm
        .run_func(Some("test"), "i64_to_f64", params!(42i64))
        .expect("Failed to run i64_to_f64");
    assert!((result[0].to_f64() - 42.0f64).abs() < 0.0000001);
}

/// Test complex calculation functions from compiled Rust WASM
#[test]
fn test_rust_wasm_complex_calculations() {
    setup_test_wasm_vm!(vm);

    // Test compound_interest: 1000 * (1 + 0.05)^10 ≈ 1628.89
    let result = vm
        .run_func(
            Some("test"),
            "compound_interest",
            params!(1000.0f64, 0.05f64, 10i32),
        )
        .expect("Failed to run compound_interest");
    assert!((result[0].to_f64() - 1628.894627f64).abs() < 0.001);

    // Test distance_2d: distance from (0,0) to (3,4) = 5
    let result = vm
        .run_func(
            Some("test"),
            "distance_2d",
            params!(0.0f64, 0.0f64, 3.0f64, 4.0f64),
        )
        .expect("Failed to run distance_2d");
    assert!((result[0].to_f64() - 5.0f64).abs() < 0.0000001);

    // Test circle_area: π * r^2 for r=2 ≈ 12.566
    let result = vm
        .run_func(Some("test"), "circle_area", params!(2.0f64))
        .expect("Failed to run circle_area");
    let expected = std::f64::consts::PI * 4.0;
    assert!((result[0].to_f64() - expected).abs() < 0.0000001);

    // Test circle_circumference: 2πr for r=2 ≈ 12.566
    let result = vm
        .run_func(Some("test"), "circle_circumference", params!(2.0f64))
        .expect("Failed to run circle_circumference");
    let expected = 2.0 * std::f64::consts::PI * 2.0;
    assert!((result[0].to_f64() - expected).abs() < 0.0000001);

    // Test hypotenuse: sqrt(3^2 + 4^2) = 5
    let result = vm
        .run_func(Some("test"), "hypotenuse", params!(3.0f64, 4.0f64))
        .expect("Failed to run hypotenuse");
    assert!((result[0].to_f64() - 5.0f64).abs() < 0.0000001);

    // Test quadratic_discriminant: b^2 - 4ac for a=1, b=5, c=6 = 25-24 = 1
    let result = vm
        .run_func(
            Some("test"),
            "quadratic_discriminant",
            params!(1.0f64, 5.0f64, 6.0f64),
        )
        .expect("Failed to run quadratic_discriminant");
    assert!((result[0].to_f64() - 1.0f64).abs() < 0.0000001);
}

// ============================================================================
// Tests for WASM calling host functions
// ============================================================================

/// Host data for tracking state across host function calls
#[derive(Debug, Default)]
struct HostState {
    counter: i32,
    log_messages: Vec<String>,
}

/// Host function: get the current counter value
fn host_get_counter(
    data: &mut HostState,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    _args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(data.counter)])
}

/// Host function: increment the counter by a given amount
fn host_increment(
    data: &mut HostState,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let amount = args[0].to_i32();
    data.counter += amount;
    Ok(vec![])
}

/// Host function: multiply two numbers (provided by host)
fn host_multiply(
    _data: &mut HostState,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let a = args[0].to_i32();
    let b = args[1].to_i32();
    Ok(vec![WasmValue::from_i32(a * b)])
}

/// Host function: log a value (stores in host state)
fn host_log_value(
    data: &mut HostState,
    _inst: &mut Instance,
    _frame: &mut CallingFrame,
    args: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    let value = args[0].to_i32();
    data.log_messages.push(format!("logged: {}", value));
    Ok(vec![])
}

/// Test WASM module calling host functions
#[test]
fn test_wasm_calls_host_function() {
    // Create a WASM module that imports and calls host functions
    let wasm_bytes = wat2wasm(
        br#"
        (module
            ;; Import host functions
            (import "host" "multiply" (func $host_multiply (param i32 i32) (result i32)))
            (import "host" "get_counter" (func $host_get_counter (result i32)))
            (import "host" "increment" (func $host_increment (param i32)))
            (import "host" "log_value" (func $host_log_value (param i32)))

            ;; WASM function that uses host multiply
            (func $calculate (param $a i32) (param $b i32) (result i32)
                ;; Call host multiply and return result
                (call $host_multiply (local.get $a) (local.get $b))
            )

            ;; WASM function that increments counter and returns new value
            (func $increment_and_get (param $amount i32) (result i32)
                (call $host_increment (local.get $amount))
                (call $host_get_counter)
            )

            ;; WASM function that does computation and logs intermediate results
            (func $compute_with_logging (param $n i32) (result i32)
                (local $result i32)
                (local $i i32)

                ;; Initialize result to 0
                (local.set $result (i32.const 0))
                (local.set $i (i32.const 1))

                ;; Sum 1 to n, logging each step
                (block $break
                    (loop $continue
                        ;; if i > n, break
                        (br_if $break (i32.gt_s (local.get $i) (local.get $n)))

                        ;; result += i
                        (local.set $result (i32.add (local.get $result) (local.get $i)))

                        ;; log the current result
                        (call $host_log_value (local.get $result))

                        ;; i++
                        (local.set $i (i32.add (local.get $i) (i32.const 1)))

                        ;; continue loop
                        (br $continue)
                    )
                )

                (local.get $result)
            )

            (export "calculate" (func $calculate))
            (export "increment_and_get" (func $increment_and_get))
            (export "compute_with_logging" (func $compute_with_logging))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    // Create host state
    let host_state = HostState::default();

    // Build import object with host functions
    let mut import_builder = ImportObjectBuilder::new("host", host_state).unwrap();
    import_builder
        .with_func::<(i32, i32), i32>("multiply", host_multiply)
        .unwrap();
    import_builder
        .with_func::<(), i32>("get_counter", host_get_counter)
        .unwrap();
    import_builder
        .with_func::<i32, ()>("increment", host_increment)
        .unwrap();
    import_builder
        .with_func::<i32, ()>("log_value", host_log_value)
        .unwrap();
    let mut import_object = import_builder.build();

    // Create instances map with the import object
    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(import_object.name().unwrap().to_string(), &mut import_object);

    // Create VM with the instances
    let mut vm = Vm::new(Store::new(None, instances).expect("Failed to create store"));

    // Load and register the WASM module
    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test 1: WASM calls host multiply function
    let result = vm
        .run_func(None, "calculate", params!(7i32, 8i32))
        .expect("Failed to run calculate");
    assert_eq!(result[0].to_i32(), 56, "7 * 8 should be 56");

    // Test 2: WASM calls host increment and get_counter
    let result = vm
        .run_func(None, "increment_and_get", params!(10i32))
        .expect("Failed to run increment_and_get");
    assert_eq!(result[0].to_i32(), 10, "Counter should be 10 after incrementing by 10");

    let result = vm
        .run_func(None, "increment_and_get", params!(5i32))
        .expect("Failed to run increment_and_get");
    assert_eq!(result[0].to_i32(), 15, "Counter should be 15 after incrementing by 5");

    // Test 3: WASM computes with logging (sum 1 to 5 = 15, with logging each step)
    let result = vm
        .run_func(None, "compute_with_logging", params!(5i32))
        .expect("Failed to run compute_with_logging");
    assert_eq!(result[0].to_i32(), 15, "Sum of 1 to 5 should be 15");
}

/// Test simple host function callback
#[test]
fn test_simple_host_callback() {
    // Simple WASM that imports a "double" function from host
    let wasm_bytes = wat2wasm(
        br#"
        (module
            (import "env" "double" (func $double (param i32) (result i32)))

            (func $quadruple (param $x i32) (result i32)
                ;; Call double twice: double(double(x))
                (call $double (call $double (local.get $x)))
            )

            (export "quadruple" (func $quadruple))
        )
        "#,
    )
    .expect("Failed to convert WAT to WASM");

    // Host function that doubles a number
    fn host_double(
        _data: &mut (),
        _inst: &mut Instance,
        _frame: &mut CallingFrame,
        args: Vec<WasmValue>,
    ) -> Result<Vec<WasmValue>, CoreError> {
        let x = args[0].to_i32();
        Ok(vec![WasmValue::from_i32(x * 2)])
    }

    // Build import object
    let mut import_builder = ImportObjectBuilder::new("env", ()).unwrap();
    import_builder
        .with_func::<i32, i32>("double", host_double)
        .unwrap();
    let mut import_object = import_builder.build();

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(import_object.name().unwrap().to_string(), &mut import_object);

    let mut vm = Vm::new(Store::new(None, instances).expect("Failed to create store"));

    let module = Module::from_bytes(None, wasm_bytes).expect("Failed to load module");
    vm.register_module(None, module)
        .expect("Failed to register module");

    // Test: quadruple(5) = double(double(5)) = double(10) = 20
    let result = vm
        .run_func(None, "quadruple", params!(5i32))
        .expect("Failed to run quadruple");
    assert_eq!(result[0].to_i32(), 20, "quadruple(5) should be 20");

    // Test: quadruple(3) = double(double(3)) = double(6) = 12
    let result = vm
        .run_func(None, "quadruple", params!(3i32))
        .expect("Failed to run quadruple");
    assert_eq!(result[0].to_i32(), 12, "quadruple(3) should be 12");
}
