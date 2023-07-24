// rustup target add wasm32-wasi
// rustc hello.rs --target=wasm32-wasi -o hello.wasm
fn main() {
    for arg in std::env::args() {
        println!("[hello] arg: {}", arg);
    }

    let val = std::env::var("ENV").expect("failed to get environment variable: ENV");
    println!("[hello] ENV={val}");

    println!("[hello] Done!");
}
