#!/usr/bin/env bash

set -o errexit

# version="v0.2.0"

echo "Runnning Sniprun Installer"
local_version=vv$(cat Cargo.toml | grep version | cut -d "\"" -f 2)
name="sniprun"
force_build=$1

cargo_build() {
  if command -v cargo >/dev/null; then
    echo "Building..."
    cargo build --release &>/dev/null
    echo "Done"
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
  rm -rf download_dir
  mkdir -p download_dir
  cd download_dir
  # curl -s https://api.github.com/repos/michaelb/sniprun/releases/$1 | grep "sniprun" | cut -d ":" -f 2,3 | tr -d \" | wget -qi -
  wget -q https://github.com/michaelb/sniprun/releases/download/v0.4.9/sniprun
  mv -f sniprun ../target/release/
  cd ..
}

fetch_prebuilt_binary() {
  mkdir -p target/release

  if (download $1); then
    chmod a+x target/release/sniprun
    echo "Done"
    return
  else
    return 1
  fi
}

arch=$(uname)
if [ $arch != "Linux" ]; then
  echo "Warning: Doesn't look like you are on Linux. Sniprun is not tested on Mac and will not work on windows"
fi

remote_version=v$(get_latest_release)

if [ $force_build ]; then
  echo "Always compiling the latest commit (most bleeding-edge option)"
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
  if [ success == 1 ]; then
    echo "Could not build (missing rust/cargo toolchain?). Getting an out-of-date release if available"
    fetch_prebuilt_binary $tag_to_fetch # get an older release
  fi
fi


