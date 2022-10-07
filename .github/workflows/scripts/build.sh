#!/usr/bin/env bash

# Install target component
rastup target add $TARGET

# Build
cargo build --target $TARGET --release