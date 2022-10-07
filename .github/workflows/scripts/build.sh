#!/usr/bin/env bash

# Install target component
rustup target add $TARGET

# Build
cargo build --target $TARGET --release