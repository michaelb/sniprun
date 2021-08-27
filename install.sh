#!/usr/bin/env bash


echo "Runnning Sniprun Installer"
local_version=v$(cat Cargo.toml | grep version | cut -d "\"" -f 2)
name="sniprun"
force_build=$1

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Building..."
    cargo build --release &>/dev/null
    echo "Done"
    return 0
  else
    echo "Could not find cargo in \$PATH: the Rust toolchain is required to build Sniprun"
    return 1
  fi
}
get_latest_release() {
  curl --silent "https://api.github.com/repos/michaelb/sniprun/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' # Pluck JSON value
}

download() {
  # command -v curl >/dev/null &&
  # curl --fail --location "$1" --output target/release/sniprun
  echo "Downloading" $1
  # curl -s https://api.github.com/repos/michaelb/sniprun/releases/$1 | grep "sniprun" | cut -d ":" -f 2,3 | tr -d \" | wget -qi -
  curl -fsSL https://github.com/michaelb/sniprun/releases/download/$1/sniprun --output sniprun
  mkdir -p target/release/
  mv -f sniprun target/release/
}

fetch_prebuilt_binary() {
  mkdir -p target/release

  if (download $1); then
    chmod a+x target/release/sniprun
    echo "Done"
    return 0 
  else
    return 1
  fi
}

arch=$(uname)
if [ $arch != "Linux" ]; then
  echo "Looks you are not running Linux: Mac users have to compile sniprun themselves and thus need the Rust toolchain"
  force_build=1
fi

remote_version=$(get_latest_release)

if [ $force_build ]; then
  echo "Compiling sniprun locally:"
  neovim_version=$(nvim --version | head -n 1 | cut -d . -f 2) # 4 -> neovim 0.4.x
  if [ $neovim_version == "4" ]; then
    echo "Sniprun 0.4.9 is the highest version supported on neovim 0.4.x"
    git reset --hard v0.4.9
  fi
  cargo_build
else

  tag_to_fetch=$remote_version
  neovim_version=$(nvim --version | head -n 1 | cut -d . -f 2) # 4 -> neovim 0.4.x
  if [ $neovim_version == "4" ]; then
    echo "Sniprun 0.4.9 is the highest version supported on neovim 0.4.x"
    git reset --hard v0.4.9
    tag_to_fetch="v0.4.9"
  fi


  #check if release is up to date
  success=1
  if [ $local_version == $remote_version ]; then
    echo "Trying to get a up-to-date precompiled binary"
    fetch_prebuilt_binary $tag_to_fetch
    success=$?
  else
    echo "Release version is not up to date, building from source"
    cargo_build
    success=$?
  fi

  # if nothing succeeded
  if [ $success == 1 ]; then
    echo "Could not build (missing rust/cargo toolchain?). Getting an out-of-date release if available"
    fetch_prebuilt_binary $tag_to_fetch # get an older release
  fi
fi


