#!/usr/bin/env bash

# Install target component
rustup target add $TARGET

if [[ "$OS" == "macos" ] || [ "$OS" == "linux" ]]; then
  if [[ "$OS" == "linux" ]]; then
    sudo apt-get install libxcb-composite0-dev -y
  fi

  if [[ "$TARGET" == "aarch64-unknown-linux-gnu" ]]; then
    sudo apt-get install gcc-aarch64-linux-gnu -y
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER="aarch64-linux-gnu-gcc"
  elif [[ "$TARGET" == "armv7-unknown-linux-gnueabihf" ]]; then
    sudo apt-get install pkg-config gcc-arm-linux-gnueabihf -y
    export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER="arm-linux-gnueabihf-gcc"
  else
    if [[ "$OS" == "linux" ]]; then
      sudo apt install musl-tools -y
    fi
  fi

  cargo build --release --all --target $target --features=static-link-openssl
else
  cargo build --target $TARGET --release
fi