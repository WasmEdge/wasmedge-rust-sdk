# Upgrade to 0.14.0
Due to the WasmEdge Rust SDK breaking changes, this document shows the guideline for programming with WasmEdge Rust SDK to upgrade from the 0.13.2 to the 0.14.0 version.

## Run a wasm with wasi
Before the version `0.14.0`.
* WASI is a special module and requires a unique approach to set it up.
```rust
use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    params, VmBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let wasm_file = std::path::PathBuf::from(&args[1]);

    // enable the `wasi` option
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    // create a vm
    let mut vm = VmBuilder::new().with_config(config).build()?;

    // set the envs and args for the wasi module
    let args = vec!["arg1", "arg2"];
    let envs = vec!["ENV1=VAL1", "ENV2=VAL2", "ENV3=VAL3"];
    // the preopened directory is the current directory. You have to guarantee
    // the write permission if you wanna write something in this directory.
    let preopens = vec![(".:./target")];
    let wasi_module = vm.wasi_module_mut().expect("Not found wasi module");
    wasi_module.initialize(Some(args), Some(envs), Some(preopens));
    assert_eq!(wasi_module.exit_code(), 0);

    // load wasm module and run the wasm function named `print_env`
    vm.run_func_from_file(wasm_file, "print_env", params!())?;

    Ok(())
}
```
After `0.14.0`.
* WASI is just like any other normal module. It is created with its own `WasiModule::create`, which inserts into a map, and then creates a `Store` and `VM` from that map. 

```rust
use std::collections::HashMap;

use wasmedge_sdk::{params, wasi::WasiModule, Module, Store, Vm};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let wasm_file = std::path::PathBuf::from(&args[1]);

    // set the envs and args for the wasi module
    let args = vec!["arg1", "arg2"];
    let envs = vec!["ENV1=VAL1", "ENV2=VAL2", "ENV3=VAL3"];
    // the preopened directory is the current directory. You have to guarantee
    // the write permission if you wanna write something in this directory.
    let preopens = vec![(".:.")];
    let mut wasi_module = WasiModule::create(Some(args), Some(envs), Some(preopens)).unwrap();
    let mut instances = HashMap::new();
    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
    // create a vm
    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let module = Module::from_file(None, wasm_file).unwrap();
    vm.register_module(None, module).unwrap();
    // load wasm module and run the wasm function named `print_env`
    vm.run_func(None, "print_env", params!()).unwrap();

    Ok(())
}
```

## Define a ImportObject with host data
The incorrect use of host data is the main reason for this SDK change.
Before the version `0.14.0`.
```rust
use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder, HostRegistrationConfigOptions},
    error::HostFuncError,
    host_function, params, Caller, ImportObjectBuilder, NeverType, ValType, VmBuilder, WasmVal,
    WasmValue,
};

#[host_function]
fn my_add(
    _caller: Caller,
    input: Vec<WasmValue>,
    data: &mut Circle,
) -> Result<Vec<WasmValue>, HostFuncError> {
    println!("radius of circle: {}", data.radius);

    // check the number of inputs
    if input.len() != 2 {
        return Err(HostFuncError::User(1));
    }

    // parse the first input of WebAssembly value type into Rust built-in value type
    let a = if input[0].ty() == ValType::I32 {
        input[0].to_i32()
    } else {
        return Err(HostFuncError::User(2));
    };

    // parse the second input of WebAssembly value type into Rust built-in value type
    let b = if input[1].ty() == ValType::I32 {
        input[1].to_i32()
    } else {
        return Err(HostFuncError::User(3));
    };

    let c = a + b;

    Ok(vec![WasmValue::from_i32(c)])
}

// define host data
#[derive(Clone, Debug)]
struct Circle {
    radius: i32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let circle = Circle { radius: 10 };

    // create an import module
    let import = ImportObjectBuilder::new()
        .with_func::<(i32, i32), i32, Circle>("add", my_add, Some(Box::new(circle)))? // circle is leak!
        .build::<NeverType>("extern", None)?;

    // enable the `wasi` option
    let config = ConfigBuilder::new(CommonConfigOptions::default())
        .with_host_registration_config(HostRegistrationConfigOptions::default().wasi(true))
        .build()?;

    // create a new Vm with default config
    let mut vm = VmBuilder::new().with_config(config).build()?;

    vm.register_import_module(&import)?;

    let res = vm.run_func(Some("extern"), "add", params!(15, 51))?;

    println!("add({}, {}) = {}", 15, 51, res[0].to_i32());

    Ok(())
}
```

After `0.14.0`.
* The host function args changed `(_caller: Caller, input: Vec<WasmValue>, data:  &mut Circle)` to `(data:  &mut Circle, _inst:  &mut Instance, _caller:  &mut CallingFrame, _input: Vec<WasmValue>)`
* Host data is bound to `ImportObject` instead of `ImportObject function`

* All host data ImportObjects must first put their `&mut ref` into a `HashMap<String,  &mut dyn SyncInst>` before registering them into the VM. This is because different `ImportObject` have different data types, which results in different types for `importObject<Data>`, and the Rust SDK cannot design a safe container to store these varying types. Additionally, during VM runtime, it is necessary to ensure that these ImportObjects are not dropped.

```rust
use std::collections::HashMap;

use wasmedge_sdk::{
    error::CoreError, params, vm::SyncInst, wasi::WasiModule, AsInstance, CallingFrame,
    ImportObjectBuilder, Instance, Store, ValType, Vm, WasmVal, WasmValue,
};

// define host data
#[derive(Clone, Debug)]
struct Circle {
    radius: i32,
}

fn get_radius(
    data: &mut Circle,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    _input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    Ok(vec![WasmValue::from_i32(data.radius)])
}

fn inc_radius(
    data: &mut Circle,
    _inst: &mut Instance,
    _caller: &mut CallingFrame,
    input: Vec<WasmValue>,
) -> Result<Vec<WasmValue>, CoreError> {
    // check the number of inputs
    if input.len() != 1 {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    }

    // parse the first input of WebAssembly value type into Rust built-in value type
    let value = if input[0].ty() == ValType::I32 {
        input[0].to_i32()
    } else {
        return Err(CoreError::Execution(
            wasmedge_sdk::error::CoreExecutionError::FuncSigMismatch,
        ));
    };

    data.radius += value;

    Ok(vec![])
}

fn main() {
    let circle = Circle { radius: 10 };

    let mut wasi_module = WasiModule::create(None, None, None).unwrap();

    // create an import module
    let mut import_builder = ImportObjectBuilder::new("extern", circle).unwrap();
    import_builder
        .with_func::<i32, ()>("inc_radius", inc_radius)
        .unwrap();
    import_builder
        .with_func::<(), i32>("get_radius", get_radius)
        .unwrap();
    let mut import_object = import_builder.build();

    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
    instances.insert(import_object.name().unwrap(), &mut import_object);

    // create a new Vm with default config
    let mut vm = Vm::new(Store::new(None, instances).unwrap());

    let res = vm.run_func(Some("extern"), "get_radius", vec![]).unwrap();
    println!("get_radius() = {}", res[0].to_i32());

    let res = vm.run_func(Some("extern"), "inc_radius", params!(5));
    println!("inc_radius(5) = {:?}", res);

    let res = vm.run_func(Some("extern"), "get_radius", vec![]).unwrap();
    println!("get_radius() = {}", res[0].to_i32());
}
```

## More Examples
[second-state/wasmedge-rustsdk-examples (github.com)](https://github.com/second-state/wasmedge-rustsdk-examples) 
These examples show how the usage before SDK 0.14.0 has changed to adapt to the 0.14.0 version.