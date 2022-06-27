# Docker script to compile the MobileCoin crawler mc-crawler
# Artifacts are placed in /mc-crawler s.t. compiled binaries can be found in /mc-crawler/target/release and sources in /mc-crawler.
# Image built using the ./build-docker.sh script

FROM rustlang/rust:nightly-bullseye-slim AS chef

# Use cargo-chef to build and cache dependencies
RUN cargo install cargo-chef
WORKDIR mc-crawler

# The toolchain we must use to compile.
# This must be in-sync with rust-toolchain.toml
RUN rustup toolchain install nightly-2021-03-25
RUN rustup override set nightly-2021-03-25

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    clang-3.9 \
    gcc \
    g++ \
    python3 \
    protobuf-compiler\
    libssl-dev \
    libcurl4-openssl-dev \
    libssl1.1

# Set SGX environment variables needed by mobilecoin libraries
ENV IAS_MODE=DEV \
    SGX_MODE=SW

# Determine the project's dependencies (=requirements.txt)
COPY --from=planner /mc-crawler/recipe.json recipe.json
# Build and cache dependencies for faster incremental builds.
RUN cargo chef cook --release --recipe-path recipe.json

# Now build
COPY . .
RUN cargo build --release --locked
