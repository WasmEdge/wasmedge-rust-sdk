// rustup target add wasm32-wasi
// rustc async_hello.rs --target=wasm32-wasi -o async_hello.wasm
fn main() {
    for arg in std::env::args() {
        println!("[wasm-app] arg: {}", arg);
    }

    let val = std::env::var("ENV").expect("failed to get environment variable: ENV");
    println!("[wasm-app] ENV={val}");

    for _ in 0..10 {
        println!("[async hello] say hello");
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    println!("[async hello] Done!");
}
