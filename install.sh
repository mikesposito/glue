#!/usr/bin/env bash

echo "Welcome to the glue installer. The current version of glue will be installed"

# Install Rust
if ! command -v cargo --version &> /dev/null
then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | RUSTUP_INIT_SKIP_PATH_CHECK=true sh -s -- -y
fi

# Build glue
cargo build --release

# Move to Glue home
mkdir $HOME/.glue
mkdir $HOME/.glue/bin

cp target/release/glue $HOME/.glue/bin

if [[ ":$PATH:" == *"$HOME/.glue/bin"* ]]; then
  echo "Your path is correctly set already."
else
  if [[ "$SHELL" == *"zsh" ]]; then
    echo "export PATH=$PATH:$HOME/.glue/bin" >> $HOME/.zshrc
  fi

  if [ -n "$BASH_VERSION" ]; then
    echo 'export PATH=$PATH:$HOME/.glue/bin' >> $HOME/.bashrc
  fi

  echo "$HOME/.glue/bin Added to your path."
fi

echo "Install completed. Restart your terminal window or open a new one to execute the glue command"
$HOME/.glue/bin/glue --version