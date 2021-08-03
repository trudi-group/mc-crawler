# mc-crawler - A [MobileCoin](https://github.com/mobilecoinfoundation/mobilecoin) Network Crawler

This binary crawls the MobileCoin network and writes the results as a JSON.

The crawler communicates with the validator nodes using RPCs provided by the [mc-consensus-api](https://github.com/mobilecoinfoundation/mobilecoin/tree/master/consensus/api), and asks each new node for the last consensus message it broadcast to the other validators.
The response of the gRPC contains, among other information, the queried node's quorum set which in turn contains other validators that the crawler may have not yet seen.

The crawler, therefore, only finds validators (no watcher nodes), and will not find nodes that are not included in any validator's quorum set.

The JSON contains the following data about every found node:

    - Hostname
    - Port
    - Quorum Set
    - Public Key
    - Connectivity status
    - IP-based Geolocation data, i.e. country and ISP

## 1. Install required tools

   - [Rust](https://www.rust-lang.org)
        - Install: https://www.rust-lang.org/tools/install
   - Rust's Nightly Compiler
        - `rustup toolchain install nightly-2021-03-25`
   - In the project directory:
        - `rustup override set nightly-2021-03-25`    
        - The output of `rustup toolchain list` should now be similar to this:
        ```
        ...
        nightly-2021-03-25-x86_64-unknown-linux-gnu (override)
        ...
        ```
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
SGX_MODE=SW IAS_MODE=DEV cargo build --release
```
    - The environment variables are only necessary if you skipped step 2.
    - The initial compilation will take several minutes due to some of the dependencies used in this project.
    - NB: If you have gcc 11 installed, the release build will likely fail. This is because of an error in 
   [mobilecoinofficial/rust-mbedtls](https://github.com/mobilecoinofficial/rust-mbedtls/issues/6) which this project indirectly depends on.

    This can be fixed without downgrading the system-wide gcc by compiling the project with an older version of gcc, e.g. `gcc-10`.
    One possibility of doing so is via the CMake environment variables `CC` and `CXX` like below:

    ```
    export CC=/usr/bin/gcc-10 CXX=/usr/bin/g++-10
    ```
   Other methods of setting a different compiler can be found [here](https://gitlab.kitware.com/cmake/community/-/wikis/FAQ#how-do-i-use-a-different-compiler).

2. Run
```
cargo run --release -- [--output output_directory --debug]
```
    - The default output directory is set to `crawl_data`.
    - Debug level messages are suppressed by default.
      Passing --debug results in more verbose terminal output during the crawl.
