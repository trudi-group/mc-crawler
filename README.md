# mc-crawler - A [MobileCoin](https://github.com/mobilecoinfoundation/mobilecoin) Network Crawler
This binary crawls the MobileCoin network and stores the results (as a JSON).
The JSON contains the following data about every found node:

    - Hostname
    - Port
    - Quorum Set
    - Public Key
    - Whether or not the node was online
    - Geolocation data, e.g. country and ISP

## 1. Install required tools

   - [Rust](https://www.rust-lang.org)
        - Install: https://www.rust-lang.org/tools/install
   - Rust's Nightly Compiler
        - `rustup toolchain install nightly-2021-03-25`
        - `rustup override set nightly-2021-03-25`    

## 2. Environment Variables 
Some of the crates used in this library need the Intel SGX environment variables
`SGX_MODE` and `IAS_MODE`.
You can (optionally) set them in your terminal like below or pass them when building the binary.
```
export SGX_MODE=SW
export IAS_MODE=DEV

```

## 3. Crawling the Network
1. Build
```
SGX_MODE=SW IAS_MODE=DEV cargo build --release`
```
    - The environment variables are only necessary if you skipped step 2.
2. Run
```
cargo run -- [--output output_directory --debug]`
```
    - The output directory is `crawl_data` by default.
    - Debug level messages are supressed by default. Passing `--debug` results in more verbose terminal output during the crawl.
