//! Run a WASI command with separate input, output, and error files.
//!
//! On Unix, run:
//! `cargo run --example wasi_stdio -- guest.wasm input.txt output.txt errors.txt`
//! The output and error files are created or truncated.

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{collections::HashMap, env, fs::File, os::unix::io::AsRawFd};
    use wasmedge_sdk::{params, vm::SyncInst, wasi::WasiModule, Module, Store, Vm};

    let args: Vec<_> = env::args().collect();
    if args.len() != 5 {
        return Err("usage: wasi_stdio guest.wasm input.txt output.txt errors.txt".into());
    }
    let input = File::open(&args[2])?;
    let output = File::create(&args[3])?;
    let errors = File::create(&args[4])?;
    // SAFETY: These files stay open until after the VM and WASI module are dropped.
    let mut wasi = unsafe {
        WasiModule::create_with_fds(
            Some(vec![&args[1]]),
            None,
            None,
            input.as_raw_fd(),
            output.as_raw_fd(),
            errors.as_raw_fd(),
        )
    }?;
    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi.name().into(), wasi.as_mut());
    let mut vm = Vm::new(Store::new(None, instances)?);
    vm.register_module(None, Module::from_file(None, &args[1])?)?;
    vm.run_func(None, "_start", params!())?;
    Ok(())
}

#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("this example uses Unix file descriptors".into())
}
