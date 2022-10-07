#!/usr/bin/env bash

# Install target component
$HOME/cargo/.bin/rastup target add $TARGET

# Build
cargo build --target $TARGET --release