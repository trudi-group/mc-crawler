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

## 1. Required tools

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

  - The `mobilecoinofficial/rust-mbedtls` crate, which this project indirectly depends on, does not currently support gcc 11 (see [this issue](https://github.com/mobilecoinofficial/rust-mbedtls/issues/6)). Release builds, therefore, fail if the latest gcc is used for compilation.
  
    This can be fixed without downgrading the system-wide gcc by compiling the project with an older version of gcc,
       e.g. `gcc-10`. One possibility of doing so is via the CMake environment variables `CC` and `CXX` 
       like below (in the project directory):
    ```
    export CC=/usr/bin/gcc-10 CXX=/usr/bin/g++-10
    ``` 

## 2. (Optional but recommended) Environment Variables
Some of the crates used in this library need the Intel SGX environment variables
`SGX_MODE` and `IAS_MODE`.

You can set them in your terminal like below or pass them when [building the binary](#build).

`export SGX_MODE=SW IAS_MODE=DEV`

Having set the environment variables, the SGX variables do not need to be passed whenever
a call to a cargo subcommand is made.

Continue to the [section on running the crawler](#run).

## 3. Crawling the Network
### Build
`SGX_MODE=SW IAS_MODE=DEV cargo build --release`

    - The environment variables are only necessary if you skipped step 2.
    - The initial compilation will take several minutes due to some of the dependencies used in this project.

### Run

`SGX_MODE=SW IAS_MODE=DEV cargo run --release [-- --output output_directory --debug]`

    - The environment variables are only necessary if you skipped step 2.
    - The default output directory is set to "crawl_data".
    - Debug level messages are suppressed by default.
      Passing --debug results in more verbose terminal output during the crawl.
