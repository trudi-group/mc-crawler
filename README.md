## 1. Install / check required tools

   - [Rust](https://www.rust-lang.org)
     - Web app: Check: `$ rustc -V` => `rustc 1.43.1 (8d69840ab 2020-05-04)`
     - Console app: Check: `$ rustc -V` => `rustc 1.53.0 (07e0e2ec2 2021-03-24)`
     - Install: https://www.rust-lang.org/tools/install
   - [cargo-make](https://sagiegurari.github.io/cargo-make/)
     - Check: `$ cargo make -V` => `cargo-make 0.30.7`
     - Install: `$ cargo install cargo-make`
   - cargo install trunk
   - cargo install wasm-bindgen-cli

## 2. Running the Console App 
    1. cargo run --release -p mobcoin-crawler-console

## 3. Running the Web App
    1. `cd web-app/`
    1. Build with `trunk serve`
    2. Open [localhost:8000](http://localhost:8000)
