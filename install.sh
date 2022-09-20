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
printf "Welcome to the glue installer. The current version of glue will be installed\n\n"

# Install Rust
NEW_CARGO=false
if ! command -v cargo --version &> /dev/null
then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | RUSTUP_INIT_SKIP_PATH_CHECK=true sh -s -- -y
  NEW_CARGO=true
fi

# Build glue
cargo build --release

# Create Glue Home
mkdir $HOME/.glue
mkdir $HOME/.glue/bin
export GLUE_HOME=$HOME/.glue

# Move executable to Glue Home
cp target/release/glue $GLUE_HOME/bin
chmod +x $GLUE_HOME/bin/glue

if [[ ":$PATH:" == *"$GLUE_HOME/bin"* ]]; then
  printf "$GLUE_HOME/bin is correctly set to your PATH already.\n"
else
  if [[ "$SHELL" == *"zsh" ]]; then
    echo "export PATH=$PATH:$GLUE_HOME/bin" >> $HOME/.zshrc
  fi

  if [ -n "$BASH_VERSION" ]; then
    echo 'export PATH=$PATH:$HOME/.glue/bin' >> $HOME/.bashrc
  fi

  printf "$GLUE_HOME/bin Added to your PATH.\n"
fi

# if Rust was not previously installed, uninstall it
if $NEW_CARGO; then
  rustup self uninstall -y
fi

$HOME/.glue/bin/glue --version

printf "\nInstall completed. Restart your terminal window or open a new one to execute the glue command\n"