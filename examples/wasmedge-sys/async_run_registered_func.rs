//! This example demonstrates how to call registered functions asynchronously.
//!
//! To run this example, use the following command:
//!
//! ```bash
//! cd <wasmedge-root-dir>/bindings/rust/
//!
//! cargo run -p wasmedge-sys --features async --example async_run_registered_func
//! ```

#[cfg(all(feature = "async", target_os = "linux"))]
use wasmedge_sys::{
    r#async::fiber::AsyncState, Config, Executor, Loader, Store, Validator, WasmValue,
};
#[cfg(all(feature = "async", target_os = "linux"))]
use wasmedge_types::wat2wasm;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "async", target_os = "linux"))]
    {
        // create a Config context
        let mut config = Config::create()?;
        config.bulk_memory_operations(true);
        assert!(config.bulk_memory_operations_enabled());

        // create an executor
        let mut executor = Executor::create(Some(&config), None)?;

        // create a store
        let mut store = Store::create()?;

        // register a wasm module from a buffer
        let wasm_bytes = wat2wasm(
            br#"
        (module
            (export "fib" (func $fib))
            (func $fib (param $n i32) (result i32)
             (if
              (i32.lt_s
               (local.get $n)
               (i32.const 2)
              )
              (then
                (return (i32.const 1))
              )
             )
             (return
              (i32.add
               (call $fib
                (i32.sub
                 (local.get $n)
                 (i32.const 2)
                )
               )
               (call $fib
                (i32.sub
                 (local.get $n)
                 (i32.const 1)
                )
               )
              )
             )
            )
           )
    "#,
        )?;

        let module = Loader::create(Some(&config))?.from_bytes(&wasm_bytes)?;
        Validator::create(Some(&config))?.validate(&module)?;
        let fib = executor
            .register_named_module(&mut store, &module, "extern")?
            .get_func("fib")?;

        // async run function
        let async_state1 = AsyncState::new();
        let fut1 = executor.call_func_async(&async_state1, &fib, vec![WasmValue::from_i32(20)]);
        let async_state2 = AsyncState::new();
        let fut2 = executor.call_func_async(&async_state2, &fib, vec![WasmValue::from_i32(5)]);

        let returns = tokio::join!(fut1, fut2);

        let (ret1, ret2) = returns;
        let returns1 = ret1?;
        assert_eq!(returns1[0].to_i32(), 10946);
        let returns2 = ret2?;
        assert_eq!(returns2[0].to_i32(), 8);
    }

    Ok(())
}
