#![cfg(unix)]

use std::{
    collections::HashMap,
    io::{Read, Write},
    os::unix::{io::AsRawFd, net::UnixStream},
    time::Duration,
};
use wasmedge_sdk::{params, vm::SyncInst, wasi::WasiModule, wat2wasm, Module, Store, Vm};

struct Streams {
    guest: [UnixStream; 3],
    host: [UnixStream; 3],
}

impl Streams {
    fn new() -> Self {
        let pairs: Vec<_> = (0..3)
            .map(|_| {
                let (guest, host) = UnixStream::pair().unwrap();
                for stream in [&guest, &host] {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_secs(2)))
                        .unwrap();
                }
                (guest, host)
            })
            .collect();
        let (guest, host): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
        Self {
            guest: guest.try_into().unwrap(),
            host: host.try_into().unwrap(),
        }
    }

    fn read_output(&mut self, fd: usize) -> [u8; 4] {
        let mut output = [0; 4];
        self.host[fd].read_exact(&mut output).unwrap();
        output
    }
}

fn echo(wasi: &mut WasiModule, expected_config: (i32, i32, i32)) {
    let wasm = wat2wasm(
        br#"(module
            (import "wasi_snapshot_preview1" "fd_read"
                (func $read (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_write"
                (func $write (param i32 i32 i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "args_sizes_get"
                (func $args (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "environ_sizes_get"
                (func $envs (param i32 i32) (result i32)))
            (import "wasi_snapshot_preview1" "fd_prestat_get"
                (func $prestat (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "argc") (result i32)
                (drop (call $args (i32.const 96) (i32.const 100)))
                (i32.load (i32.const 96)))
            (func (export "envc") (result i32)
                (drop (call $envs (i32.const 96) (i32.const 100)))
                (i32.load (i32.const 96)))
            (func (export "preopen_len") (result i32)
                (if (result i32)
                    (i32.eqz (call $prestat (i32.const 3) (i32.const 96)))
                    (then (i32.load (i32.const 100)))
                    (else (i32.const -1))))
            (data (i32.const 64) "ERR!")
            (func (export "echo") (result i32) (local $errno i32)
                (i32.store (i32.const 0) (i32.const 32))
                (i32.store (i32.const 4) (i32.const 4))
                (local.set $errno
                    (i32.or
                        (call $read (i32.const 0) (i32.const 0) (i32.const 1) (i32.const 8))
                        (call $write (i32.const 1) (i32.const 0) (i32.const 1) (i32.const 8))))
                (i32.store (i32.const 0) (i32.const 64))
                (i32.or (local.get $errno)
                    (call $write (i32.const 2) (i32.const 0) (i32.const 1) (i32.const 8))))
        )"#,
    )
    .unwrap();
    let mut instances: HashMap<String, &mut dyn SyncInst> = HashMap::new();
    instances.insert(wasi.name().into(), wasi.as_mut());
    let mut vm = Vm::new(Store::new(None, instances).unwrap());
    vm.register_module(None, Module::from_bytes(None, wasm).unwrap())
        .unwrap();
    for (name, expected) in [
        ("argc", expected_config.0),
        ("envc", expected_config.1),
        ("preopen_len", expected_config.2),
    ] {
        assert_eq!(
            vm.run_func(None, name, params!()).unwrap()[0].to_i32(),
            expected
        );
    }
    assert_eq!(vm.run_func(None, "echo", params!()).unwrap()[0].to_i32(), 0);
}

#[test]
fn wasi_guests_have_separate_standard_streams() {
    let mut a = Streams::new();
    let mut b = Streams::new();
    // SAFETY: Both sets of stream owners outlive their WASI modules and VM calls.
    let mut wasi_a = unsafe {
        WasiModule::create_with_fds(
            Some(vec!["guest-a", "argument"]),
            Some(vec!["KEY=VALUE"]),
            Some(vec!["/fixture:."]),
            a.guest[0].as_raw_fd(),
            a.guest[1].as_raw_fd(),
            a.guest[2].as_raw_fd(),
        )
    }
    .unwrap();
    let mut wasi_b = unsafe {
        WasiModule::create_with_fds(
            None,
            None,
            None,
            b.guest[0].as_raw_fd(),
            b.guest[1].as_raw_fd(),
            b.guest[2].as_raw_fd(),
        )
    }
    .unwrap();
    a.host[0].write_all(b"AAAA").unwrap();
    b.host[0].write_all(b"BBBB").unwrap();
    std::thread::scope(|scope| {
        scope.spawn(|| echo(&mut wasi_a, (2, 1, 7)));
        scope.spawn(|| echo(&mut wasi_b, (1, 0, -1)));
    });
    assert_eq!(a.read_output(1), *b"AAAA");
    assert_eq!(a.read_output(2), *b"ERR!");
    assert_eq!(b.read_output(1), *b"BBBB");
    assert_eq!(b.read_output(2), *b"ERR!");

    drop(wasi_a);
    drop(wasi_b);
    for fd in 0..3 {
        a.guest[fd].write_all(b"open").unwrap();
        b.guest[fd].write_all(b"open").unwrap();
        assert_eq!(a.read_output(fd), *b"open");
        assert_eq!(b.read_output(fd), *b"open");
    }
}
