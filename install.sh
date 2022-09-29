#!/usr/bin/env bash

echo '           $$                      '
echo '           $$ |                     '
echo '  $$$$$$   $$ |$$    $$  $$$$$$   '
echo ' $$  __$$  $$ |$$ |  $$ |$$  __$$  '
echo ' $$ /  $$ |$$ |$$ |  $$ |$$$$$$$$ | '
echo ' $$ |  $$ |$$ |$$ |  $$ |$$   ____| '
echo '  $$$$$$$ |$$ | $$$$$$  | $$$$$$$   '
echo '   ____$$ | __|  ______/  \_______| '
echo ' $$    $$ |                         '
echo '  $$$$$$  |                         '
echo '   ______/                          '
printf "\n\n"

# Install Rust
NEW_CARGO=false
if ! command -v cargo --version &> /dev/null
then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | RUSTUP_INIT_SKIP_PATH_CHECK=true sh -s -- -y
  NEW_CARGO=true
fi

# Build glue
cargo install --path .

# if Rust was not previously installed, uninstall it
if $NEW_CARGO; then
  rustup self uninstall -y
fi

printf "\nInstall completed. Restart your terminal window or open a new one to execute the glue command\n"