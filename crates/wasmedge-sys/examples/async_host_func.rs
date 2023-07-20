//! This example demonstrates how to create host functions and call them asynchronously.
//!
//! To run this example, use the following command:
//!
//! ```bash
//! cargo run -p wasmedge-sys --features async --example async_host_func
//! ```

#[cfg(feature = "async")]
use wasmedge_sys::{r#async::AsyncState, CallingFrame, Executor, FuncType, Function, WasmValue};
#[cfg(feature = "async")]
use wasmedge_types::error::HostFuncError;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "async")]
    {
        #[derive(Debug)]
        struct Data<T, S> {
            _x: i32,
            _y: String,
            _v: Vec<T>,
            _s: Vec<S>,
        }
        let data: Data<i32, &str> = Data {
            _x: 12,
            _y: "hello".to_string(),
            _v: vec![1, 2, 3],
            _s: vec!["macos", "linux", "windows"],
        };

        // create a FuncType
        let func_ty = FuncType::create(vec![], vec![])?;

        // define an async closure
        let c = |_frame: CallingFrame,
                 _args: Vec<WasmValue>,
                 data: *mut std::os::raw::c_void|
         -> Box<
            (dyn std::future::Future<Output = Result<Vec<WasmValue>, HostFuncError>> + Send),
        > {
            let data = unsafe { Box::from_raw(data as *mut Data<i32, &str>) };

            Box::new(async move {
                println!("Hello, world!");
                println!("host_data: {:?}", data);

                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                // Wrap the future with a `Timeout` set to expire in 10 milliseconds.
                let res = tokio::time::timeout(std::time::Duration::from_millis(100), async {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                })
                .await;
                if res.is_err() {
                    println!("did not receive value within 100 ms");
                }
                println!("Rust: Leaving Rust function real_add");
                println!("Hello, world after sleep!");
                Ok(vec![])
            })
        };

        // create a host function
        let async_host_func =
            Function::create_async_func(&func_ty, Box::new(c), Some(Box::new(data)), 0)?;

        // run this function
        let mut executor = Executor::create(None, None)?;

        // create an async execution state
        let async_state = AsyncState::new();

        async_host_func
            .call_async(&async_state, &mut executor, vec![])
            .await?;
    }

    Ok(())
}
