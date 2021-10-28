# mc-crawler - A [MobileCoin](https://github.com/mobilecoinfoundation/mobilecoin) Network Crawler

[![Rust](https://camo.githubusercontent.com/5782bcc58a7786e9a7d00e2cf45937db8a2598232d9524ec9dcd149c7218671b/68747470733a2f2f696d672e736869656c64732e696f2f62616467652f527573742d50726f6772616d6d696e672532304c616e67756167652d626c61636b3f7374796c653d666c6174266c6f676f3d72757374)](www.rust-lang.org)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
[![CI](https://github.com/wiberlin/mc-crawler/actions/workflows/test.yml/badge.svg)](https://github.com/wiberlin/mc-crawler/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/wiberlin/mc-crawler/branch/main/graph/badge.svg?token=GxUhfuKEoA)](https://codecov.io/gh/wiberlin/mc-crawler)
[![dependency status](https://deps.rs/repo/github/wiberlin/mc-crawler/status.svg)](https://deps.rs/repo/github/wiberlin/mc-crawler)

This binary crawls the MobileCoin network and optionally provides 2 JSONs in [stellarbeat.io](https://stellarbeat.io) format.

The crawler communicates with the validator nodes using RPCs provided by the [mc-consensus-api](https://github.com/mobilecoinfoundation/mobilecoin/tree/master/consensus/api), and asks each new node for the last consensus message it broadcast to the other validators.
The response of the gRPC contains, among other information, the queried node's quorum set which in turn contains other validators that the crawler may have not yet seen.

The crawler, therefore, only finds validators (no watcher nodes), and will not find nodes that are not included in any validator's quorum set.

The Nodes-JSON contains the following data about every found node:

    - Hostname
    - Port
    - Quorum Set
    - Public Key
    - Connectivity status
    - (When available) IP-based Geolocation data, i.e. country and ISP

The Crawl Report contains the same data as the Nodes-JSON in addition to metadata about the crawl such as the duration and a timestamp.

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
        N.B. After cloning the repository, you only need to do this once.

   - Protobuf compiler `protoc` which can be built from source or installed using a package manager or , e.g.
   
        ``` apt install -y protobuf-compiler ``` on Ubuntu

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

```
export SGX_MODE=SW IAS_MODE=DEV
```

Having set the environment variables, the SGX variables do not need to be passed whenever
a call to a cargo subcommand is made.

Continue to the [section on running the crawler](#run).

## 3. Crawling the Network
### Build
`SGX_MODE=SW IAS_MODE=DEV cargo build --release`

    - The environment variables are only necessary if you skipped step 2.
    - The initial compilation will take several minutes due to some of the dependencies used in this project.

### Run

`SGX_MODE=SW IAS_MODE=DEV cargo run --release [-- --output output_directory --debug --fbas --complete]`

    - The environment variables are only necessary if you skipped step 2.
    - The default output directory is set to "crawl_data".
    - The crawler optionally writes a JSON with the FBAS discovered during the crawl when "fbas" is passed.
    - The crawler optionally writes a JSON with additional data about the crawl when "complete" is passed.
    - Debug level messages are suppressed by default.
      Passing --debug results in more verbose terminal output during the crawl.

## 4. Analysing the crawl data using the fbas_analyzer
The results presented in the paper can all be reproduced using the data obtained from the crawler and the [fbas_analyzer](https://github.com/wiberlin/fbas_analyzer).

Refer to its documentation for installation instructions before proceeding.

Below are some example commands: (see `target/release/fbas_analyzer -h` for more analysis options)

### Find all minimal quorums, minimal blocking sets and minimal splitting sets and output metrics about the sizes of the node sets. 
`target/release/fbas_analyzer -adp mobilecoin_nodes_completed_manually_2021-08-02.json`

### Find the same sets as above, but merge by organisations
`target/release/fbas_analyzer -adp mobilecoin_nodes_completed_manually_2021-08-02.json --merge-by-org mobilecoin_organisations_2021-08-02_created_manually.json`

### Find the same sets as above, but merge by ISPs
`target/release/fbas_analyzer -adp mobilecoin_nodes_completed_manually_2021-08-02.json --merge-by-isp`

### Find the same sets as above, but merge by countries and output lists of node lists (instead of metrics)
`target/release/fbas_analyzer -ap mobilecoin_nodes_completed_manually_2021-08-02.json --merge-by-country`
