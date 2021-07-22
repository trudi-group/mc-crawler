This binary crawls the MobileCoin network and stores the results (as a JSON).
The JSON contains the following data about every found node:
    - Hostname
    - IP Address
    - Port
    - Quorum Set
    - Whether or not the node was online

## 1. Install / check required tools

   - [Rust](https://www.rust-lang.org)
     - Web app: Check: `$ rustc -V` => `rustc 1.43.1 (8d69840ab 2020-05-04)`
     - Install: https://www.rust-lang.org/tools/install

## 2. Environment Variables 
Some of the crates used in this library need the Intel SGX environment variables
`SGX_MODE` and `IAS_MODE`. 

    `export SGX_MODE=SW`
    `export IAS_MODE=DEV`

## 3. Running the binary
    1. `cargo run --release`
    2. `SGX_MODE=SW IAS_MODE=DEV cargo run mobcoin-crawler-console`
        Alternatively, you can export the variables (see mc-crawler-terminal).

