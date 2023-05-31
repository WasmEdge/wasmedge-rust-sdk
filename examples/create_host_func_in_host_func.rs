#[cfg(not(feature = "async"))]
use wasmedge_sdk::{
    error::HostFuncError, host_function, params, Caller, Executor, Func, ImportObjectBuilder,
    NeverType, ValType, VmBuilder, WasmVal, WasmValue,
};

#[cfg(not(feature = "async"))]
#[host_function]
fn func<T>(
    _caller: Caller,
    _input: Vec<WasmValue>,
    _data: Option<&mut T>,
) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("Entering host function: func");

    // spawn a new thread to create another host function
    let handler = std::thread::spawn(|| {
        #[host_function]
        fn real_add<T>(
            _frame: Caller,
            input: Vec<WasmValue>,
            _data: Option<&mut T>,
        ) -> Result<Vec<WasmValue>, HostFuncError> {
            println!("Rust: Entering Rust function real_add");

            if input.len() != 2 {
                return Err(HostFuncError::User(1));
            }

            let a = if input[0].ty() == ValType::I32 {
                input[0].to_i32()
            } else {
                return Err(HostFuncError::User(2));
            };

            let b = if input[1].ty() == ValType::I32 {
                input[1].to_i32()
            } else {
                return Err(HostFuncError::User(3));
            };

            let c = a + b;

            println!("Rust: Leaving Rust function real_add");
            Ok(vec![WasmValue::from_i32(c)])
        }

        // create a host function
        let result = Func::wrap::<(i32, i32), i32, NeverType>(real_add, None);
        assert!(result.is_ok());
        let func = result.unwrap();

        // create an executor
        let executor = Executor::new(None, None).unwrap();

        // call the host function
        let result = func.run(&executor, params!(2, 3));
        assert!(result.is_ok());
        let returns = result.unwrap();
        assert_eq!(returns[0].to_i32(), 5);
    });
    handler.join().unwrap();

    println!("Leaving host function: func");
    Ok(vec![])
}

#[cfg_attr(test, test)]
fn main() -> anyhow::Result<()> {
    #[cfg(not(feature = "async"))]
    {
        // create an import module
        let import = ImportObjectBuilder::new()
            .with_func::<(), (), NeverType>("outer-func", func, None)?
            .build("extern")?;

        let _ = VmBuilder::new()
            .build()?
            .register_import_module(import)?
            .run_func(Some("extern"), "outer-func", params!())?;
    }

    Ok(())
}
